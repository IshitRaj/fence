use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Resource {
    Filesystem,
    Process,
    Network,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Operation {
    Read,
    Write,
    Delete,
    Execute,
    Connect,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Target {
    Path(PathBuf),

    Process {
        command: String,
        args: Vec<String>,
        cwd: PathBuf,
    },

    Network {
        host: String,
        port: u16,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct FenceRequest {
    pub resource: Resource,
    pub operation: Operation,
    pub target: Target,
}

impl FenceRequest {
    pub fn filesystem(operation: Operation, path: impl Into<PathBuf>) -> Self {
        Self {
            resource: Resource::Filesystem,
            operation,
            target: Target::Path(path.into()),
        }
    }

    pub fn process(command: impl Into<String>, args: Vec<String>, cwd: impl Into<PathBuf>) -> Self {
        Self {
            resource: Resource::Process,
            operation: Operation::Execute,
            target: Target::Process {
                command: command.into(),
                args,
                cwd: cwd.into(),
            },
        }
    }

    pub fn network(host: impl Into<String>, port: u16) -> Self {
        Self {
            resource: Resource::Network,
            operation: Operation::Connect,
            target: Target::Network {
                host: host.into(),
                port,
            },
        }
    }
}
