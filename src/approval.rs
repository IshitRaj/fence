use crate::engine::FenceRequest;

/// A decision returned by an approval handler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ApprovalDecision {
    Approved,
    Denied,
}

/// Handles requests that match an `ask` policy rule.
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
