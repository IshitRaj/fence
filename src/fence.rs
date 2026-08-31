use crate::approval::{ApprovalDecision, ApprovalHandler};
use crate::engine::{Decision, FenceRequest, Operation};
use crate::policy::path::resolve_runtime_path;
use crate::policy::{ParseError, Policy, parse};
use std::sync::Arc;

/// Errors produced while performing a policy-controlled operation.
#[derive(Debug)]
pub enum FenceOperationError {
    Denied,
    Ask(FenceRequest),
    Io(std::io::Error),
}

pub struct Fence {
    policy: Policy,
    root: std::path::PathBuf,
    approval_handler: Option<Arc<dyn ApprovalHandler>>,
}

impl std::fmt::Display for FenceOperationError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FenceOperationError::Denied => write!(f, "request denied by policy"),
            FenceOperationError::Ask(request) => write!(
                f,
                "policy marks `{request}` as ask, but no approval handler is configured. \
                 Call `.with_approval_handler(...)` on this Fence, or handle `FenceOperationError::Ask` yourself."
            ),
            FenceOperationError::Io(err) => write!(f, "io error: {err}"),
        }
    }
}

impl std::error::Error for FenceOperationError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FenceOperationError::Io(err) => Some(err),
            _ => None,
        }
    }
}

impl Fence {
    /// Loads a Fence policy from a `.fence` file.
    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self, FenceError> {
        let path = path.as_ref();

        if path.extension().and_then(|ext| ext.to_str()) != Some("fence") {
            return Err(FenceError::InvalidPolicyFile);
        }

        let contents = std::fs::read_to_string(path).map_err(FenceError::Io)?;
        let policy = parse(&contents).map_err(FenceError::Parse)?;

        let root = path
            .parent()
            .unwrap_or_else(|| std::path::Path::new("."))
            .canonicalize()
            .map_err(FenceError::Io)?;

        Ok(Self {
            policy,
            root,
            approval_handler: None,
        })
    }

    /// Registers the application callback used to resolve `ask` rules.
    pub fn with_approval_handler(mut self, handler: impl ApprovalHandler + 'static) -> Self {
        self.approval_handler = Some(Arc::new(handler));
        self
    }

    /// Evaluates a request against the loaded policy.
    pub fn check(&self, request: &FenceRequest) -> Decision {
        self.policy.evaluate(request, &self.root)
    }

    /// Authorizes a request using the policy and, when required, the approval handler.
    ///
    /// `Allow` permits the operation immediately. `Deny` rejects it immediately.
    /// For `Ask`, the configured approval handler is consulted. If no handler is
    /// configured, the request is returned as `FenceOperationError::Ask`.
    fn authorize(&self, request: &FenceRequest) -> Result<(), FenceOperationError> {
        match self.check(request) {
            Decision::Allow => Ok(()),
            Decision::Deny => Err(FenceOperationError::Denied),
            Decision::Ask => match &self.approval_handler {
                Some(handler) => match handler.approve(request) {
                    ApprovalDecision::Approved => Ok(()),
                    ApprovalDecision::Denied => Err(FenceOperationError::Denied),
                },
                None => Err(FenceOperationError::Ask(request.clone())),
            },
        }
    }

    /// Reads a file after policy authorization.
    pub fn read(&self, path: impl AsRef<std::path::Path>) -> Result<Vec<u8>, FenceOperationError> {
        let request_path =
            resolve_runtime_path(path.as_ref(), &self.root).map_err(FenceOperationError::Io)?;
        let request = FenceRequest::filesystem(Operation::Read, request_path.clone());
        self.authorize(&request)?;
        std::fs::read(&request_path).map_err(FenceOperationError::Io)
    }

    /// Writes a file after policy authorization.
    pub fn write(
        &self,
        path: impl AsRef<std::path::Path>,
        content: impl AsRef<[u8]>,
    ) -> Result<(), FenceOperationError> {
        let request_path =
            resolve_runtime_path(path.as_ref(), &self.root).map_err(FenceOperationError::Io)?;
        let request = FenceRequest::filesystem(Operation::Write, request_path.clone());
        self.authorize(&request)?;
        std::fs::write(&request_path, content).map_err(FenceOperationError::Io)
    }

    /// Deletes a file after policy authorization.
    pub fn delete(&self, path: impl AsRef<std::path::Path>) -> Result<(), FenceOperationError> {
        let request_path =
            resolve_runtime_path(path.as_ref(), &self.root).map_err(FenceOperationError::Io)?;
        let request = FenceRequest::filesystem(Operation::Delete, request_path.clone());
        self.authorize(&request)?;
        std::fs::remove_file(&request_path).map_err(FenceOperationError::Io)
    }

    /// Executes a process after policy authorization.
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
        let request = FenceRequest::process(&command, args.clone(), request_cwd.clone());
        self.authorize(&request)?;
        std::process::Command::new(&command)
            .args(&args)
            .current_dir(&request_cwd)
            .output()
            .map_err(FenceOperationError::Io)
    }

    /// Opens a TCP connection after policy authorization.
    pub fn connect(
        &self,
        host: impl Into<String>,
        port: u16,
    ) -> Result<std::net::TcpStream, FenceOperationError> {
        let host = host.into();
        let request = FenceRequest::network(&host, port);
        self.authorize(&request)?;
        std::net::TcpStream::connect((host.as_str(), port)).map_err(FenceOperationError::Io)
    }
}

/// Errors produced while loading a Fence policy.
#[derive(Debug)]
pub enum FenceError {
    InvalidPolicyFile,
    Io(std::io::Error),
    Parse(ParseError),
}

impl std::fmt::Display for FenceError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            FenceError::InvalidPolicyFile => {
                write!(f, "policy file must have a `.fence` extension")
            }
            FenceError::Io(err) => write!(f, "failed to read policy file: {err}"),
            FenceError::Parse(err) => write!(f, "invalid policy file: {err}"),
        }
    }
}

impl std::error::Error for FenceError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            FenceError::Io(err) => Some(err),
            FenceError::Parse(err) => Some(err),
            FenceError::InvalidPolicyFile => None,
        }
    }
}
