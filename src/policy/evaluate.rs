use super::model::Policy;
use crate::engine::{Decision, FenceRequest, Operation, Resource, Target};

impl Policy {
    fn evaluate_path(
        path: &std::path::Path,
        deny: &[super::model::PathPattern],
        ask: &[super::model::PathPattern],
        allow: &[super::model::PathPattern],
    ) -> Decision {
        if deny.iter().any(|pattern| pattern.matches(path)) {
            return Decision::Deny;
        }

        if ask.iter().any(|pattern| pattern.matches(path)) {
            return Decision::Ask;
        }

        if allow.iter().any(|pattern| pattern.matches(path)) {
            return Decision::Allow;
        }

        Decision::Deny
    }

    pub fn evaluate(&self, request: &FenceRequest) -> Decision {
        match (&request.resource, &request.operation, &request.target) {
            (Resource::Filesystem, Operation::Read, Target::Path(path)) => Self::evaluate_path(
                path,
                &self.filesystem.deny.read,
                &self.filesystem.ask.read,
                &self.filesystem.allow.read,
            ),

            (Resource::Filesystem, Operation::Write, Target::Path(path)) => Self::evaluate_path(
                path,
                &self.filesystem.deny.write,
                &self.filesystem.ask.write,
                &self.filesystem.allow.write,
            ),

            (Resource::Filesystem, Operation::Delete, Target::Path(path)) => Self::evaluate_path(
                path,
                &self.filesystem.deny.delete,
                &self.filesystem.ask.delete,
                &self.filesystem.allow.delete,
            ),

            _ => Decision::Deny,
        }
    }
}
