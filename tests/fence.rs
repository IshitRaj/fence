use fence::Fence;
use fence::engine::{Decision, FenceRequest, Operation};

use std::fs;
use std::time::{SystemTime, UNIX_EPOCH};

fn temp_policy_path() -> std::path::PathBuf {
    let id: u128 = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();

    std::env::temp_dir().join(format!("fence-test-{id}.fence"))
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
