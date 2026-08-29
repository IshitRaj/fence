use std::path::{Component, Path, PathBuf};

/// Replace a leading `~` with $HOME. Leaves everything else untouched.
pub fn expand_home(path: impl AsRef<Path>) -> std::io::Result<PathBuf> {
    let path = path.as_ref();

    let home = std::env::var_os("HOME").ok_or_else(|| {
        std::io::Error::new(
            std::io::ErrorKind::NotFound,
            "HOME environment variable not set",
        )
    })?;

    if path == Path::new("~") {
        return Ok(PathBuf::from(home)); // bare "~"
    }

    if let Ok(stripped) = path.strip_prefix("~") {
        return Ok(PathBuf::from(home).join(stripped)); // "~/foo" -> "$HOME/foo"
    }

    Ok(path.to_path_buf()) // no leading "~", nothing to do
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
                normalized.pop(); // ".." -> remove the last thing we pushed
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

/// Real, on-disk-style path -> absolute, fully normalized.
/// expand "~" -> make absolute against cwd if relative -> resolve "."/"..".
pub fn normalize_runtime_path(path: impl AsRef<Path>) -> std::io::Result<PathBuf> {
    let expanded = expand_home(path)?;

    let absolute = if expanded.is_absolute() {
        expanded
    } else {
        std::env::current_dir()?.join(expanded)
    };

    Ok(normalize_path(absolute))
}

/// Same job as normalize_path, but for GLOB PATTERN strings, not Path.
/// "*"/"**" aren't real path syntax -- std::path would just treat them as
/// ordinary Component::Normal segments, so this walks the string manually.
pub fn normalize_pattern(pattern: &str) -> std::io::Result<String> {
    let expanded = expand_home(pattern)?;
    let expanded = expanded.to_string_lossy();

    let mut result = Vec::new();

    for component in expanded.split('/') {
        match component {
            "" | "." => {} // "" comes from things like "/a//b"; drop with "."

            ".." => {
                // Pop the previous segment, UNLESS it's "**" -- you can't
                // cancel a variable-depth wildcard with a literal "..".
                if let Some(last) = result.last()
                    && *last != "**"
                {
                    result.pop();
                }
            }

            "*" | "**" => {
                result.push(component); // wildcards pass through as-is
            }

            component => {
                result.push(component);
            }
        }
    }

    Ok(format!("/{}", result.join("/"))) // re-anchored as absolute
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
    fn expands_home_in_pattern() {
        let result = normalize_pattern("~/projects/**").unwrap();

        let home = std::env::var("HOME").unwrap();

        assert_eq!(result, format!("{home}/projects/**"));
    }

    #[test]
    fn normalizes_pattern_parent_directory() {
        let result = normalize_pattern("/home/user/projects/../other/**").unwrap();

        assert_eq!(result, "/home/user/other/**");
    }
}
