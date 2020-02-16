use failure::Fail;

use crate::Aggregate;

#[cfg(test)]
mod tests;

pub trait CommandError: Fail {}

pub trait Command<A: Aggregate> {
    type Events: IntoIterator<Item = A::Event>;
    type Error: CommandError;

    fn execute_on(self, aggregate: &A) -> Result<Self::Events, Self::Error>;
}
