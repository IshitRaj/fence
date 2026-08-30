use std::path::{Component, Path, PathBuf};

/// Replace a leading `~` with `$HOME`.
pub fn expand_home(path: impl AsRef<Path>) -> std::io::Result<PathBuf> {
    let path = path.as_ref();

    let home = std::env::var_os("HOME").ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "HOME environment variable not set",
        )
    })?;

    if path == Path::new("~") {
        return Ok(PathBuf::from(home));
    }

    let path_str = path.to_string_lossy();

    if let Some(stripped) = path_str.strip_prefix("~/") {
        return Ok(PathBuf::from(home).join(stripped));
    }

    Ok(path.to_path_buf())
}

/// Lexically resolve "." and "..", pure component math, no disk access,
/// so it works even for paths that don't exist yet.
pub fn normalize_path(path: impl AsRef<Path>) -> PathBuf {
    let path = path.as_ref();
    let mut normalized = PathBuf::new();

    for component in path.components() {
        match component {
            Component::CurDir => {} // "." -> drop

            Component::ParentDir => {
                if normalized.file_name().is_some() {
                    normalized.pop();
                }
            }

            Component::RootDir => {
                normalized.push(Path::new("/"));
            }

            Component::Normal(part) => {
                normalized.push(part); // ordinary segment
            }

            Component::Prefix(prefix) => {
                normalized.push(prefix.as_os_str()); // Windows drive letters, e.g. "C:"
            }
        }
    }

    normalized
}

/// Resolve a runtime path against a base directory.
///
/// Absolute paths remain absolute.
/// Relative paths are resolved against `base`.
/// `~` is resolved against `$HOME`.
pub fn resolve_runtime_path(
    path: impl AsRef<Path>,
    base: impl AsRef<Path>,
) -> std::io::Result<PathBuf> {
    let expanded = expand_home(path)?;

    let absolute = if expanded.is_absolute() {
        expanded
    } else {
        base.as_ref().join(expanded)
    };

    Ok(normalize_path(absolute))
}

/// Resolve a runtime path against the current working directory.
pub fn normalize_runtime_path(path: impl AsRef<Path>) -> std::io::Result<PathBuf> {
    let cwd = std::env::current_dir()?;
    resolve_runtime_path(path, cwd)
}

/// Normalize a glob pattern against a base directory.
///
/// Relative patterns are resolved against `base`.
/// Absolute patterns remain absolute.
/// `~` is resolved against `$HOME`.
pub fn normalize_pattern(pattern: &str, base: impl AsRef<Path>) -> std::io::Result<String> {
    let expanded = expand_home(pattern)?;

    let absolute = if expanded.is_absolute() {
        expanded
    } else {
        base.as_ref().join(expanded)
    };

    let expanded = absolute.to_string_lossy();

    let mut result = Vec::new();

    for component in expanded.split('/') {
        match component {
            "" | "." => {}

            ".." => {
                if let Some(last) = result.last()
                    && *last != "**"
                {
                    result.pop();
                }
            }

            "*" | "**" => {
                result.push(component);
            }

            component => {
                result.push(component);
            }
        }
    }

    Ok(format!("/{}", result.join("/")))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn removes_current_directory() {
        let result = normalize_path("/home/user/./projects/app");

        assert_eq!(result, PathBuf::from("/home/user/projects/app"));
    }

    #[test]
    fn resolves_parent_directory() {
        let result = normalize_path("/home/user/projects/app/../secret");

        assert_eq!(result, PathBuf::from("/home/user/projects/secret"));
    }

    #[test]
    fn resolves_multiple_parent_directories() {
        let result = normalize_path("/home/user/projects/app/../../etc/passwd");

        assert_eq!(result, PathBuf::from("/home/user/etc/passwd"));
    }

    #[test]
    fn traversal_cannot_escape_root() {
        let result = normalize_path("/../../etc/passwd");

        assert_eq!(result, PathBuf::from("/etc/passwd"));
    }

    #[test]
    fn expands_home() {
        let result = expand_home("~/projects/file.txt").unwrap();
        let home = std::env::var("HOME").unwrap();

        assert_eq!(result, PathBuf::from(home).join("projects/file.txt"));
    }

    #[test]
    fn relative_path_uses_base() {
        let result = resolve_runtime_path("./playground/test.txt", "/home/user/project").unwrap();

        assert_eq!(
            result,
            PathBuf::from("/home/user/project/playground/test.txt")
        );
    }

    #[test]
    fn absolute_path_ignores_base() {
        let result = resolve_runtime_path("/playground/test.txt", "/home/user/project").unwrap();

        assert_eq!(result, PathBuf::from("/playground/test.txt"));
    }

    #[test]
    fn relative_pattern_uses_base() {
        let result = normalize_pattern("./playground/**", "/home/user/project").unwrap();

        assert_eq!(result, "/home/user/project/playground/**");
    }

    #[test]
    fn absolute_pattern_ignores_base() {
        let result = normalize_pattern("/playground/**", "/home/user/project").unwrap();

        assert_eq!(result, "/playground/**");
    }

    #[test]
    fn pattern_parent_directory_is_normalized() {
        let result = normalize_pattern("/home/user/projects/../other/**", "/ignored").unwrap();

        assert_eq!(result, "/home/user/other/**");
    }
}
