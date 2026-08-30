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

    fn evaluate_host(
        host: &str,
        deny: &[super::model::HostPattern],
        ask: &[super::model::HostPattern],
        allow: &[super::model::HostPattern],
    ) -> Decision {
        if deny.iter().any(|pattern| pattern.matches(host)) {
            return Decision::Deny;
        }

        if ask.iter().any(|pattern| pattern.matches(host)) {
            return Decision::Ask;
        }

        if allow.iter().any(|pattern| pattern.matches(host)) {
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

            (
                Resource::Process,
                Operation::Execute,
                Target::Process {
                    command,
                    args: _,
                    cwd,
                },
            ) => {
                if !self.process.scope.iter().any(|scope| scope.matches(cwd)) {
                    return Decision::Deny;
                }

                if self.process.deny.iter().any(|cmd| cmd == command) {
                    return Decision::Deny;
                }

                if self.process.ask.iter().any(|cmd| cmd == command) {
                    return Decision::Ask;
                }

                if self.process.allow.iter().any(|cmd| cmd == command) {
                    return Decision::Allow;
                }

                Decision::Deny
            }

            (Resource::Network, Operation::Connect, Target::Network { host, port: _ }) => {
                Self::evaluate_host(
                    host,
                    &self.network.deny,
                    &self.network.ask,
                    &self.network.allow,
                )
            }

            _ => Decision::Deny,
        }
    }
}
