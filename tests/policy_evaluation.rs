#[cfg(test)]
mod tests {
    use fence::engine::{Decision, FenceRequest, Operation};
    use fence::policy::Policy;
    use fence::policy::model::{
        FilesystemPolicy, FilesystemRules, NetworkPolicy, PathPattern, ProcessPolicy,
    };

    fn test_policy() -> Policy {
        Policy {
            filesystem: FilesystemPolicy {
                allow: FilesystemRules {
                    read: vec![PathPattern("/home/user/project/file.txt".into())],
                    ..Default::default()
                },
                ..Default::default()
            },
            process: ProcessPolicy::default(),
            network: NetworkPolicy::default(),
        }
    }

    #[test]
    fn allowed_read_returns_allow() {
        let policy = test_policy();

        let request = FenceRequest::filesystem(Operation::Read, "/home/user/project/file.txt");

        assert_eq!(policy.evaluate(&request), Decision::Allow);
    }

    #[test]
    fn unknown_read_returns_deny() {
        let policy = test_policy();

        let request = FenceRequest::filesystem(Operation::Read, "/etc/passwd");

        assert_eq!(policy.evaluate(&request), Decision::Deny);
    }

    #[test]
    fn wildcard_pattern_allows_nested_path() {
        let home = std::env::var("HOME").unwrap();

        let policy = Policy {
            filesystem: FilesystemPolicy {
                allow: FilesystemRules {
                    read: vec![PathPattern(format!("{home}/projects/**"))],
                    ..Default::default()
                },
                ..Default::default()
            },
            process: ProcessPolicy::default(),
            network: NetworkPolicy::default(),
        };

        let request = FenceRequest::filesystem(
            Operation::Read,
            format!("{home}/projects/myapp/src/main.rs"),
        );

        assert_eq!(policy.evaluate(&request), Decision::Allow);
    }

    #[test]
    fn allow_write_returns_allow() {
        let policy = Policy {
            filesystem: FilesystemPolicy {
                allow: FilesystemRules {
                    write: vec![PathPattern("/tmp/**".into())],
                    ..Default::default()
                },
                ..Default::default()
            },
            process: ProcessPolicy::default(),
            network: NetworkPolicy::default(),
        };

        let request = FenceRequest::filesystem(Operation::Write, "/tmp/test.txt");

        assert_eq!(policy.evaluate(&request), Decision::Allow);
    }

    #[test]
    fn deny_overrides_allow_for_read() {
        let policy = Policy {
            filesystem: FilesystemPolicy {
                allow: FilesystemRules {
                    read: vec![PathPattern("/tmp/**".into())],
                    ..Default::default()
                },
                deny: FilesystemRules {
                    read: vec![PathPattern("/tmp/secret/**".into())],
                    ..Default::default()
                },
                ..Default::default()
            },
            process: ProcessPolicy::default(),
            network: NetworkPolicy::default(),
        };

        let request = FenceRequest::filesystem(Operation::Read, "/tmp/secret/password.txt");

        assert_eq!(policy.evaluate(&request), Decision::Deny);
    }

    #[test]
    fn ask_overrides_allow_for_read() {
        let policy = Policy {
            filesystem: FilesystemPolicy {
                allow: FilesystemRules {
                    read: vec![PathPattern("/tmp/**".into())],
                    ..Default::default()
                },
                ask: FilesystemRules {
                    read: vec![PathPattern("/tmp/important/**".into())],
                    ..Default::default()
                },
                ..Default::default()
            },
            process: ProcessPolicy::default(),
            network: NetworkPolicy::default(),
        };

        let request = FenceRequest::filesystem(Operation::Read, "/tmp/important/file.txt");

        assert_eq!(policy.evaluate(&request), Decision::Ask);
    }

    #[test]
    fn allow_delete_returns_allow() {
        let policy = Policy {
            filesystem: FilesystemPolicy {
                allow: FilesystemRules {
                    delete: vec![PathPattern("/tmp/**".into())],
                    ..Default::default()
                },
                ..Default::default()
            },
            process: ProcessPolicy::default(),
            network: NetworkPolicy::default(),
        };

        let request = FenceRequest::filesystem(Operation::Delete, "/tmp/test.txt");

        assert_eq!(policy.evaluate(&request), Decision::Allow);
    }

    #[test]
    fn unmatched_operation_fails_closed_to_deny() {
        // No rules configured anywhere for this policy at all.
        let policy = Policy {
            filesystem: FilesystemPolicy::default(),
            process: ProcessPolicy::default(),
            network: NetworkPolicy::default(),
        };

        let request = FenceRequest::filesystem(Operation::Write, "/anything/at/all.txt");

        assert_eq!(policy.evaluate(&request), Decision::Deny);
    }
}
