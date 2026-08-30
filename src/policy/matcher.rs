use super::{
    model::{HostPattern, PathPattern},
    path::{normalize_pattern, normalize_runtime_path},
};

use std::path::Path;

impl PathPattern {
    /// Public entry point: does `path` satisfy this pattern?
    pub fn matches(&self, path: &Path) -> bool {
        // Normalize the pattern string (handles "~", ".", "..", keeps "*"/"**")
        let Ok(pattern) = normalize_pattern(&self.0) else {
            return false; // couldn't normalize -> fail closed, no match
        };

        // Normalize the actual path the same way, so both sides line up
        let Ok(path) = normalize_runtime_path(path) else {
            return false;
        };

        match_path(&pattern, &path)
    }
}

impl HostPattern {
    pub fn matches(&self, host: &str) -> bool {
        host_matches(&self.0, host)
    }
}

/// Split both sides on '/' and hand off to the recursive matcher.
fn match_path(pattern: &str, path: &Path) -> bool {
    let path = path.to_string_lossy(); // Path -> Cow<str> (lossy only bites on invalid UTF-8)

    let pattern_parts: Vec<&str> = pattern.split('/').collect();
    let path_parts: Vec<&str> = path.split('/').collect();

    match_parts(&pattern_parts, &path_parts)
}

/// Recursive matcher.
fn match_parts(pattern: &[&str], path: &[&str]) -> bool {
    // Both fully consumed at once means match
    if pattern.is_empty() {
        return path.is_empty();
    }

    if pattern[0] == "**" {
        // "**" can swallow ZERO components
        if match_parts(&pattern[1..], path) {
            return true;
        }
        // or ONE MORE component, then try again from the same "**".
        if !path.is_empty() {
            return match_parts(pattern, &path[1..]);
        }
        return false;
    }

    // Any other pattern component needs a real path component to check against
    if path.is_empty() {
        return false;
    }

    if component_matches(pattern[0], path[0]) {
        return match_parts(&pattern[1..], &path[1..]); // consume one from each side, recurse
    }

    false
}

/// Single-segment comparison: "*" matches anything, everything else is literal.
fn component_matches(pattern: &str, value: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    pattern == value
}

pub fn host_matches(pattern: &str, host: &str) -> bool {
    if pattern == "*" {
        return true;
    }

    if let Some(suffix) = pattern.strip_prefix("*.") {
        return host.ends_with(&format!(".{suffix}"));
    }

    pattern == host
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[test]
    fn exact_path_matches() {
        let pattern = PathPattern("/home/user/project/file.txt".into());

        assert!(pattern.matches(Path::new("/home/user/project/file.txt")));
    }

    #[test]
    fn different_path_does_not_match() {
        let pattern = PathPattern("/home/user/project/file.txt".into());

        assert!(!pattern.matches(Path::new("/home/user/project/other.txt")));
    }

    #[test]
    fn star_matches_single_component() {
        let pattern = PathPattern("/home/user/projects/*".into());

        assert!(pattern.matches(Path::new("/home/user/projects/app")));
        assert!(!pattern.matches(Path::new("/home/user/projects/app/src")));
    }

    #[test]
    fn double_star_matches_nested_paths() {
        let pattern = PathPattern("/home/user/projects/**".into());

        assert!(pattern.matches(Path::new("/home/user/projects/app")));
        assert!(pattern.matches(Path::new("/home/user/projects/app/src/main.rs")));
    }

    #[test]
    fn double_star_does_not_match_other_directory() {
        let pattern = PathPattern("/home/user/projects/**".into());

        assert!(!pattern.matches(Path::new("/home/user/documents/file.txt")));
    }

    #[test]
    fn home_pattern_matches_runtime_path() {
        let pattern = PathPattern("~/projects/**".into());

        let home = std::env::var("HOME").unwrap();

        let path = std::path::PathBuf::from(&home)
            .join("projects")
            .join("myapp")
            .join("src")
            .join("main.rs");

        assert!(pattern.matches(&path));
    }

    #[test]
    fn traversal_is_normalized_before_matching() {
        let pattern = PathPattern("~/projects/**".into());

        let home = std::env::var("HOME").unwrap();

        let path = std::path::PathBuf::from(&home)
            .join("projects")
            .join("app")
            .join("..")
            .join("secret.txt");

        assert!(pattern.matches(&path));
    }
}
