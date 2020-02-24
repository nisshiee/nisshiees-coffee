use failure::Fail;

use crate::{Aggregate, Command, CommandError, Event, Id};

pub mod version;
pub use version::Version;

pub mod versioned_aggregate;
pub use versioned_aggregate::VersionedAggregate;

pub mod versioned_event;
pub use versioned_event::VersionedEvent;

mod serde;

#[cfg(test)]
mod tests;

pub trait EventStorage<A: Aggregate> {
    type Events: IntoIterator<Item = VersionedEvent<A>>;
    type Error: EventStorageError;

    fn insert(&mut self, id: Id<A>, event: VersionedEvent<A>) -> Result<(), Self::Error>;

    fn read(&self, id: Id<A>) -> Result<Self::Events, Self::Error>;

    fn project(&self, id: Id<A>, event: VersionedEvent<A>, snapshot: VersionedAggregate<A>) -> Result<(), Self::Error> {
        // default noop
    }

    fn replay_aggregate(
        &self,
        id: Id<A>,
    ) -> Result<VersionedAggregate<A>, ReplayAggregateError<Self::Error>> {
        let mut aggregate = VersionedAggregate::<A>::default();
        let events = self.read(id)?;
        events.into_iter().try_for_each(|e| {
            if e.version.is_next_of(&aggregate.version) {
                e.event.apply_to(&mut aggregate.aggregate);
                aggregate.version = e.version;
                Ok(())
            } else {
                Err(ReplayAggregateError::<Self::Error>::VersionInconsistent)
            }
        })?;
        Ok(aggregate)
    }

    fn execute_command<C: Command<A>>(
        &mut self,
        id: Id<A>,
        command: C,
    ) -> Result<(), ExecuteCommandError<Self::Error, C::Error>>
    where
        A: Aggregate<Command = C>,
    {
        let aggregate = self.replay_aggregate(id)?;
        let mut next_version = aggregate.version.next();

        let events = command.execute_on(&aggregate.aggregate);
        let events = events.map_err(|e| ExecuteCommandError::Command(e))?;
        let events = events.into_iter().map(|e| {
            let version = next_version;
            next_version = version.next();

            VersionedEvent { version, event: e }
        });

        events
            .into_iter()
            .try_for_each(|e| self.insert(id, e))
            .map_err(|e| ExecuteCommandError::Insert(e))
    }
}

pub trait EventStorageError: Fail {}

#[derive(Fail, Debug, Eq, PartialEq)]
pub enum ReplayAggregateError<E: EventStorageError> {
    #[fail(display = "Storage error: {}", _0)]
    Read(#[fail(cause)] E),
    #[fail(display = "Version inconsistent")]
    VersionInconsistent,
}

impl<E: EventStorageError> From<E> for ReplayAggregateError<E> {
    fn from(e: E) -> Self {
        ReplayAggregateError::Read(e)
    }
}

#[derive(Fail, Debug, Eq, PartialEq)]
pub enum ExecuteCommandError<E: EventStorageError, C: CommandError> {
    #[fail(display = "ReplayAggregate error: {}", _0)]
    ReplayAggregate(#[fail(cause)] ReplayAggregateError<E>),
    #[fail(display = "Command error: {}", _0)]
    Command(#[fail(cause)] C),
    #[fail(display = "Insert error: {}", _0)]
    Insert(#[fail(cause)] E),
}

impl<E: EventStorageError, C: CommandError> From<ReplayAggregateError<E>>
    for ExecuteCommandError<E, C>
{
    fn from(e: ReplayAggregateError<E>) -> Self {
        ExecuteCommandError::ReplayAggregate(e)
    }
}
