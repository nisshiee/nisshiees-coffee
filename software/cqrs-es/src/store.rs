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
    use crate::store::*;
    use crate::tests::test_aggregate::*;
    use crate::*;

    #[derive(Fail, Debug, Clone, Eq, PartialEq)]
    #[fail(display = "unexpected")]
    pub struct MockStorageError {}

    impl EventStorageError for MockStorageError {}

    mock_trait_no_default!(
        MockEventStorage,
        read(Id<TestAggregate>) -> Result<Option<TestEvent>, MockStorageError>,
        insert(Id<TestAggregate>, TestEvent) -> Result<(), MockStorageError>
    );

    impl EventStorage<TestAggregate> for MockEventStorage {
        type Events = Option<TestEvent>;
        type Error = MockStorageError;

        mock_method!(read(&self, id: Id<TestAggregate>) -> Result<Self::Events, Self::Error>);
        mock_method!(insert(&mut self, id: Id<TestAggregate>, event: TestEvent) -> Result<(), Self::Error>);
    }

    #[test]
    fn replay_aggregate() {
        let storage = MockEventStorage::new(Ok(None), Ok(()));
        let id = Id::new();
        let got = storage.replay_aggregate(id);
        assert!(got.is_ok());
        assert!(storage.read.has_calls_exactly(vec![id]));
    }

    #[test]
    fn execute_command() {
        let mut storage = MockEventStorage::new(Ok(None), Ok(()));
        let id = Id::new();
        let cmd = TestCommand::Increase;
        let got = storage.execute_command(id, cmd);
        assert!(got.is_ok());
        assert!(storage.read.has_calls_exactly(vec![id]));
        assert!(storage
            .insert
            .has_calls_exactly(vec![(id, TestEvent::Increased)]));
    }
}
