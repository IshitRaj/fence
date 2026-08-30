use crate::engine::FenceRequest;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalDecision {
    Approved,
    Denied,
}

pub trait ApprovalHandler: Send + Sync {
    fn approve(&self, request: &FenceRequest) -> ApprovalDecision;
}

// lets you pass a plain closure instead of implementing the trait
impl<F> ApprovalHandler for F
where
    F: Fn(&FenceRequest) -> ApprovalDecision + Send + Sync,
{
    fn approve(&self, request: &FenceRequest) -> ApprovalDecision {
        self(request)
    }
}

impl std::fmt::Display for FenceRequest {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?} on {:?}", self.operation, self.target)
    }
}
