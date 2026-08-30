use crate::engine::{Decision, FenceRequest, Operation};
use crate::policy::{ParseError, Policy, parse};

#[derive(Debug)]
pub enum FenceOperationError {
    Denied,
    Ask,
    Io(std::io::Error),
}

pub struct Fence {
    policy: Policy,
}

impl Fence {
    pub fn load(path: impl AsRef<std::path::Path>) -> Result<Self, FenceError> {
        let contents = std::fs::read_to_string(path).map_err(FenceError::Io)?;
        let policy = parse(&contents).map_err(FenceError::Parse)?;

        Ok(Self { policy })
    }

    pub fn check(&self, request: &FenceRequest) -> Decision {
        self.policy.evaluate(request)
    }

    pub fn read(&self, path: impl AsRef<std::path::Path>) -> Result<Vec<u8>, FenceOperationError> {
        let request = FenceRequest::filesystem(Operation::Read, path.as_ref());

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
        let request = FenceRequest::filesystem(Operation::Write, path.as_ref());

        match self.check(&request) {
            Decision::Allow => std::fs::write(path, content).map_err(FenceOperationError::Io),
            Decision::Ask => Err(FenceOperationError::Ask),
            Decision::Deny => Err(FenceOperationError::Denied),
        }
    }

    pub fn delete(&self, path: impl AsRef<std::path::Path>) -> Result<(), FenceOperationError> {
        let request = FenceRequest::filesystem(Operation::Delete, path.as_ref());

        match self.check(&request) {
            Decision::Allow => std::fs::remove_file(path).map_err(FenceOperationError::Io),
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
