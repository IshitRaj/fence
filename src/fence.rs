use crate::engine::{Decision, FenceRequest};
use crate::policy::{ParseError, Policy, parse};

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
}

#[derive(Debug)]
pub enum FenceError {
    Io(std::io::Error),
    Parse(ParseError),
}
