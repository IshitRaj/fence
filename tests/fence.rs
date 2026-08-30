use fence::engine::{Decision, FenceRequest, Operation};
use fence::{Fence, FenceOperationError};

use std::fs;

fn temp_policy_path() -> std::path::PathBuf {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    std::env::temp_dir().join(format!(
        "fence-policy-{}-{}-{}.fence",
        std::process::id(),
        n,
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ))
}

fn unique_path(tag: &str) -> std::path::PathBuf {
    static COUNTER: std::sync::atomic::AtomicU64 = std::sync::atomic::AtomicU64::new(0);
    let n = COUNTER.fetch_add(1, std::sync::atomic::Ordering::Relaxed);
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    std::env::temp_dir().join(format!("fence-{tag}-{}-{n}-{nanos}", std::process::id()))
}

#[test]
fn loads_valid_policy() {
    let path = temp_policy_path();

    fs::write(
        &path,
        r#"
        [filesystem]
        allow read /tmp/**

        [process]
        allow command cargo

        [network]
        allow host api.github.com
        "#,
    )
    .unwrap();

    let result = Fence::load(&path);

    fs::remove_file(&path).unwrap();

    assert!(result.is_ok());
}

#[test]
fn rejects_missing_policy_file() {
    let path = temp_policy_path();

    let result = Fence::load(&path);

    assert!(result.is_err());
}

#[test]
fn rejects_invalid_policy() {
    let path = temp_policy_path();

    fs::write(
        &path,
        r#"
        [filesystem]
        something invalid
        "#,
    )
    .unwrap();

    let result = Fence::load(&path);

    fs::remove_file(&path).unwrap();

    assert!(result.is_err());
}

#[test]
fn reports_missing_file_error() {
    let path = temp_policy_path();

    match Fence::load(&path) {
        Err(fence::FenceError::Io(error)) => {
            assert_eq!(error.kind(), std::io::ErrorKind::NotFound);
        }
        _ => panic!("expected an IO error"),
    }
}

#[test]
fn reports_parse_error_line_and_message() {
    let path = temp_policy_path();

    fs::write(
        &path,
        r#"
        [filesystem]
        allow read /tmp/**
        invalid rule
        "#,
    )
    .unwrap();

    match Fence::load(&path) {
        Err(fence::FenceError::Parse(error)) => {
            assert_eq!(error.line, 4);
            assert_eq!(error.message, "unknown action: invalid");
        }
        _ => panic!("expected a parse error"),
    }

    fs::remove_file(&path).unwrap();
}

#[test]
fn check_allows_allowed_request() {
    let path = temp_policy_path();

    fs::write(
        &path,
        r#"
        [filesystem]
        allow read /tmp/**
        "#,
    )
    .unwrap();

    let fence = Fence::load(&path).unwrap();

    let request = FenceRequest::filesystem(Operation::Read, "/tmp/test.txt");

    assert_eq!(fence.check(&request), Decision::Allow);

    fs::remove_file(&path).unwrap();
}

#[test]
fn check_denies_unmatched_request() {
    let path = temp_policy_path();

    fs::write(
        &path,
        r#"
        [filesystem]
        allow read /tmp/**
        "#,
    )
    .unwrap();

    let fence = Fence::load(&path).unwrap();

    let request = FenceRequest::filesystem(Operation::Read, "/etc/passwd");

    assert_eq!(fence.check(&request), Decision::Deny);

    fs::remove_file(&path).unwrap();
}

#[test]
fn check_returns_ask() {
    let path = temp_policy_path();

    fs::write(
        &path,
        r#"
        [filesystem]
        ask read /tmp/important/**
        "#,
    )
    .unwrap();

    let fence = Fence::load(&path).unwrap();

    let request = FenceRequest::filesystem(Operation::Read, "/tmp/important/data.txt");

    assert_eq!(fence.check(&request), Decision::Ask);

    fs::remove_file(&path).unwrap();
}

#[test]
fn read_allows_allowed_path() {
    let dir = std::env::temp_dir();
    let policy_path = unique_path("read-allow-policy");
    let file_path = unique_path("read-allow-target.txt");

    fs::write(
        &policy_path,
        format!(
            r#"
            [filesystem]
            allow read {}/**
            "#,
            dir.display()
        ),
    )
    .unwrap();

    // Write the target file directly, bypassing Fence, so this test
    // only exercises the read path, not write.
    fs::write(&file_path, b"hello from disk").unwrap();

    let fence = Fence::load(&policy_path).unwrap();
    let contents = fence.read(&file_path).unwrap();

    assert_eq!(contents, b"hello from disk");

    let _ = fs::remove_file(&policy_path);
    let _ = fs::remove_file(&file_path);
}

#[test]
fn read_denies_disallowed_path() {
    let policy_path = unique_path("read-deny-policy");
    let file_path = unique_path("read-deny-target.txt");

    // Policy only allows reads under an unrelated scope, so the
    // target path should fall through to the default/deny behavior.
    fs::write(
        &policy_path,
        r#"
        [filesystem]
        allow read /nonexistent-fence-scope/**
        "#,
    )
    .unwrap();

    fs::write(&file_path, b"should not be readable").unwrap();

    let fence = Fence::load(&policy_path).unwrap();
    let result = fence.read(&file_path);

    assert!(matches!(result, Err(FenceOperationError::Denied)));

    let _ = fs::remove_file(&policy_path);
    let _ = fs::remove_file(&file_path);
}

#[test]
fn read_returns_ask_for_ask_rule() {
    let dir = std::env::temp_dir();
    let policy_path = unique_path("read-ask-policy");
    let file_path = unique_path("read-ask-target.txt");

    fs::write(
        &policy_path,
        format!(
            r#"
            [filesystem]
            ask read {}/**
            "#,
            dir.display()
        ),
    )
    .unwrap();

    fs::write(&file_path, b"needs confirmation").unwrap();

    let fence = Fence::load(&policy_path).unwrap();
    let result = fence.read(&file_path);

    assert!(matches!(result, Err(FenceOperationError::Ask)));

    let _ = fs::remove_file(&policy_path);
    let _ = fs::remove_file(&file_path);
}

#[test]
fn read_propagates_io_error_for_missing_file() {
    let dir = std::env::temp_dir();
    let policy_path = unique_path("read-missing-policy");
    // Deliberately never created.
    let file_path = unique_path("read-missing-target.txt");

    fs::write(
        &policy_path,
        format!(
            r#"
            [filesystem]
            allow read {}/**
            "#,
            dir.display()
        ),
    )
    .unwrap();

    let fence = Fence::load(&policy_path).unwrap();
    let result = fence.read(&file_path);

    assert!(matches!(result, Err(FenceOperationError::Io(_))));

    let _ = fs::remove_file(&policy_path);
}

#[test]
fn write_allows_allowed_path() {
    let policy_path = temp_policy_path();
    let dir = std::env::temp_dir();
    let file_path = dir.join("fence-write-test.txt");

    fs::write(
        &policy_path,
        format!(
            r#"
            [filesystem]
            allow write {}/**
            "#,
            dir.display()
        ),
    )
    .unwrap();

    let fence = Fence::load(&policy_path).unwrap();
    fence.write(&file_path, "hello from fence").unwrap();
    assert_eq!(fs::read_to_string(&file_path).unwrap(), "hello from fence");

    fs::remove_file(policy_path).unwrap();
    fs::remove_file(file_path).unwrap();
}

#[test]
fn write_denies_disallowed_path() {
    let policy_path = temp_policy_path();
    let dir = std::env::temp_dir();
    let file_path = dir.join("fence-write-denied.txt");

    fs::write(
        &policy_path,
        format!(
            r#"
            [filesystem]
            deny write {}/**
            "#,
            dir.display()
        ),
    )
    .unwrap();

    let fence = Fence::load(&policy_path).unwrap();
    let result = fence.write(&file_path, "should not be written");

    assert!(matches!(result, Err(fence::FenceOperationError::Denied)));
    assert!(!file_path.exists());

    fs::remove_file(policy_path).unwrap();
}

#[test]
fn write_returns_ask_for_ask_rule() {
    let policy_path = temp_policy_path();
    let dir = std::env::temp_dir();
    let file_path = dir.join("fence-write-ask.txt");

    fs::write(
        &policy_path,
        format!(
            r#"
            [filesystem]
            ask write {}/**
            "#,
            dir.display()
        ),
    )
    .unwrap();

    let fence = Fence::load(&policy_path).unwrap();

    let result = fence.write(&file_path, "should not be written");

    assert!(matches!(result, Err(fence::FenceOperationError::Ask)));

    assert!(!file_path.exists());

    fs::remove_file(policy_path).unwrap();
}

#[test]
fn write_returns_io_error_when_parent_does_not_exist() {
    let policy_path = temp_policy_path();
    let dir = std::env::temp_dir();

    let missing_dir = dir.join("fence-missing-parent");
    let file_path = missing_dir.join("file.txt");

    fs::write(
        &policy_path,
        format!(
            r#"
            [filesystem]
            allow write {}/**
            "#,
            dir.display()
        ),
    )
    .unwrap();

    let fence = Fence::load(&policy_path).unwrap();

    let result = fence.write(&file_path, "hello");

    assert!(matches!(result, Err(fence::FenceOperationError::Io(_))));

    fs::remove_file(policy_path).unwrap();
}

#[test]
fn delete_allows_allowed_path() {
    let policy_path = temp_policy_path();
    let dir = std::env::temp_dir();
    let file_path = dir.join("fence-delete-test.txt");

    fs::write(&file_path, "delete me").unwrap();

    fs::write(
        &policy_path,
        format!(
            r#"
            [filesystem]
            allow delete {}/**
            "#,
            dir.display()
        ),
    )
    .unwrap();

    let fence = Fence::load(&policy_path).unwrap();
    fence.delete(&file_path).unwrap();

    assert!(!file_path.exists());

    fs::remove_file(policy_path).unwrap();
}

#[test]
fn delete_denies_disallowed_path() {
    let policy_path = temp_policy_path();
    let dir = std::env::temp_dir();
    let file_path = dir.join("fence-delete-denied.txt");

    fs::write(&file_path, "keep me").unwrap();

    fs::write(
        &policy_path,
        format!(
            r#"
            [filesystem]
            deny delete {}/**
            "#,
            dir.display()
        ),
    )
    .unwrap();

    let fence = Fence::load(&policy_path).unwrap();
    let result = fence.delete(&file_path);

    assert!(matches!(result, Err(fence::FenceOperationError::Denied)));
    assert!(file_path.exists());

    fs::remove_file(policy_path).unwrap();
    fs::remove_file(file_path).unwrap();
}

#[test]
fn delete_returns_ask_for_ask_rule() {
    let policy_path = temp_policy_path();
    let dir = std::env::temp_dir();
    let file_path = dir.join("fence-delete-ask.txt");

    fs::write(&file_path, "keep me").unwrap();

    fs::write(
        &policy_path,
        format!(
            r#"
            [filesystem]
            ask delete {}/**
            "#,
            dir.display()
        ),
    )
    .unwrap();

    let fence = Fence::load(&policy_path).unwrap();
    let result = fence.delete(&file_path);

    assert!(matches!(result, Err(fence::FenceOperationError::Ask)));
    assert!(file_path.exists());

    fs::remove_file(policy_path).unwrap();
    fs::remove_file(file_path).unwrap();
}

#[test]
fn delete_returns_io_error_when_file_does_not_exist() {
    let policy_path = temp_policy_path();
    let dir = std::env::temp_dir();
    let file_path = dir.join("fence-delete-missing.txt");

    fs::write(
        &policy_path,
        format!(
            r#"
            [filesystem]
            allow delete {}/**
            "#,
            dir.display()
        ),
    )
    .unwrap();

    let fence = Fence::load(&policy_path).unwrap();
    let result = fence.delete(&file_path);

    assert!(matches!(result, Err(fence::FenceOperationError::Io(_))));

    fs::remove_file(policy_path).unwrap();
}
