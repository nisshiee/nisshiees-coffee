use crate::{Aggregate, Command, CommandError, Event, Id};
use failure::Fail;

pub trait EventStorageError: Fail {}

pub trait EventStorage<A: Aggregate> {
    type Events: IntoIterator<Item = A::Event>;
    type Error: EventStorageError;

    fn insert(&mut self, id: Id<A>, event: A::Event) -> Result<(), Self::Error>;

    // FIXME: Eventsではなく&Eventsを返すことで、不要なメモリコピーを抑制できる気がする
    fn read(&self, id: Id<A>) -> Result<Self::Events, Self::Error>;

    fn replay_aggregate(&self, id: Id<A>) -> Result<A, Self::Error> {
        let mut aggregate = A::default();
        let events = self.read(id)?;
        events.into_iter().for_each(|e| e.apply_to(&mut aggregate));
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
        let aggregate = self.replay_aggregate(id);
        let aggregate = aggregate.map_err(|e| ExecuteCommandError::Storage(e))?;

        let events = command.execute_on(&aggregate);
        let events = events.map_err(|e| ExecuteCommandError::Command(e))?;

        let ret = events.into_iter().try_for_each(|e| self.insert(id, e));
        ret.map_err(|e| ExecuteCommandError::Storage(e))
    }
}

#[derive(Fail, Debug)]
pub enum ExecuteCommandError<S: EventStorageError, C: CommandError> {
    #[fail(display = "Storage error: {}", _0)]
    Storage(S),
    #[fail(display = "Command error: {}", _0)]
    Command(C),
}

#[cfg(test)]
mod tests {
    use crate::tests::test_aggregate::*;
    use crate::tests::onmemory_storage::*;
    use crate::*;

    #[test]
    fn event() {
        let event = TestEvent::Increased;
        let mut aggregate = TestAggregate(0);
        event.apply_to(&mut aggregate);
        assert_eq!(aggregate.0, 1);
    }
}
