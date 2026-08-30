use crate::engine::{Decision, FenceRequest, Operation};
use crate::policy::path::resolve_runtime_path;
use crate::policy::{ParseError, Policy, parse};

#[derive(Debug)]
pub enum FenceOperationError {
    Denied,
    Ask,
    Io(std::io::Error),
}

pub struct Fence {
    policy: Policy,
    root: std::path::PathBuf,
}

impl Fence {
    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self, FenceError> {
        let path = path.as_ref();

        let contents = std::fs::read_to_string(path).map_err(FenceError::Io)?;
        let policy = parse(&contents).map_err(FenceError::Parse)?;

        let root = path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .canonicalize()
            .map_err(FenceError::Io)?;

        Ok(Self { policy, root })
    }

    pub fn check(&self, request: &FenceRequest) -> Decision {
        self.policy.evaluate(request, &self.root)
    }

    pub fn read(&self, path: impl AsRef<std::path::Path>) -> Result<Vec<u8>, FenceOperationError> {
        let request_path =
            resolve_runtime_path(path.as_ref(), &self.root).map_err(FenceOperationError::Io)?;

        let request = FenceRequest::filesystem(Operation::Read, request_path);

        match self.check(&request) {
            Decision::Allow => std::fs::read(path).map_err(FenceOperationError::Io),
            Decision::Ask => Err(FenceOperationError::Ask),
            Decision::Deny => Err(FenceOperationError::Denied),
        }
    }

    pub fn write(
        &self,
        path: impl AsRef<std::path::Path>,
        content: impl AsRef<[u8]>,
    ) -> Result<(), FenceOperationError> {
        let request_path =
            resolve_runtime_path(path.as_ref(), &self.root).map_err(FenceOperationError::Io)?;

        let request = FenceRequest::filesystem(Operation::Write, request_path);

        match self.check(&request) {
            Decision::Allow => std::fs::write(path, content).map_err(FenceOperationError::Io),
            Decision::Ask => Err(FenceOperationError::Ask),
            Decision::Deny => Err(FenceOperationError::Denied),
        }
    }

    pub fn delete(&self, path: impl AsRef<std::path::Path>) -> Result<(), FenceOperationError> {
        let request_path =
            resolve_runtime_path(path.as_ref(), &self.root).map_err(FenceOperationError::Io)?;

        let request = FenceRequest::filesystem(Operation::Delete, request_path);

        match self.check(&request) {
            Decision::Allow => std::fs::remove_file(path).map_err(FenceOperationError::Io),
            Decision::Ask => Err(FenceOperationError::Ask),
            Decision::Deny => Err(FenceOperationError::Denied),
        }
    }

    pub fn execute<I, S>(
        &self,
        command: impl Into<String>,
        args: I,
        cwd: impl AsRef<std::path::Path>,
    ) -> Result<std::process::Output, FenceOperationError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let command = command.into();
        let args: Vec<String> = args.into_iter().map(Into::into).collect();

        let request_cwd =
            resolve_runtime_path(cwd.as_ref(), &self.root).map_err(FenceOperationError::Io)?;

        let request = FenceRequest::process(&command, args.clone(), request_cwd);

        match self.check(&request) {
            Decision::Allow => std::process::Command::new(&command)
                .args(&args)
                .current_dir(cwd)
                .output()
                .map_err(FenceOperationError::Io),

            Decision::Ask => Err(FenceOperationError::Ask),
            Decision::Deny => Err(FenceOperationError::Denied),
        }
    }

    pub fn connect(
        &self,
        host: impl Into<String>,
        port: u16,
    ) -> Result<std::net::TcpStream, FenceOperationError> {
        let host = host.into();

        let request = FenceRequest::network(&host, port);

        match self.check(&request) {
            Decision::Allow => {
                std::net::TcpStream::connect((host.as_str(), port)).map_err(FenceOperationError::Io)
            }
            Decision::Ask => Err(FenceOperationError::Ask),
            Decision::Deny => Err(FenceOperationError::Denied),
        }
    }
}

#[derive(Debug)]
pub enum FenceError {
    Io(std::io::Error),
    Parse(ParseError),
}
