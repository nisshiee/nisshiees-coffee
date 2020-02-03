use crate::{Aggregate, Command, CommandError, Event, Id};
use failure::Fail;

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct Version(u64);

impl Version {
    pub fn is_next_of(&self, other: &Version) -> bool {
        self.0 == other.0 + 1
    }
}

#[derive(Debug, Clone, Default)]
pub struct VersionedAggregate<A: Aggregate> {
    version: Version,
    aggregate: A,
}

#[derive(Debug, Clone)]
pub struct VersionedEvent<A: Aggregate> {
    version: Version,
    event: A::Event,
}

pub trait EventStorageError: Fail {}

#[derive(Fail, Debug, Clone, Eq, PartialEq)]
pub enum ReplayAggregateError<E: EventStorageError> {
    #[fail(display = "Storage error: {}", _0)]
    EventStorage(#[fail(cause)] E),
    #[fail(display = "Version inconsistent")]
    VersionInconsistent,
}

impl<E: EventStorageError> From<E> for ReplayAggregateError<E> {
    fn from(e: E) -> Self {
        ReplayAggregateError::EventStorage(e)
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

pub trait EventStorage<A: Aggregate> {
    type Events: IntoIterator<Item = VersionedEvent<A>>;
    type Error: EventStorageError;

    fn insert(&mut self, id: Id<A>, event: A::Event) -> Result<(), Self::Error>;

    fn read(&self, id: Id<A>) -> Result<Self::Events, Self::Error>;

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

        let events = command.execute_on(&aggregate.aggregate);
        let events = events.map_err(|e| ExecuteCommandError::Command(e))?;

        events
            .into_iter()
            .try_for_each(|e| self.insert(id, e))
            .map_err(|e| ExecuteCommandError::Insert(e))
    }
}

#[cfg(test)]
mod tests {
    use crate::store::ExecuteCommandError;
    use crate::store::*;
    use crate::tests::test_aggregate::*;
    use crate::*;

    #[derive(Fail, Debug, Clone, Eq, PartialEq)]
    #[fail(display = "unexpected")]
    pub struct MockStorageError {}

    impl EventStorageError for MockStorageError {}

    #[rustfmt::skip]
    mock_trait_no_default!(
        MockEventStorage1,
        read(Id<TestAggregate>) -> Result<Vec<VersionedEvent<TestAggregate>>, MockStorageError>
    );

    impl EventStorage<TestAggregate> for MockEventStorage1 {
        type Events = Vec<VersionedEvent<TestAggregate>>;
        type Error = MockStorageError;

        fn insert(&mut self, id: Id<TestAggregate>, event: TestEvent) -> Result<(), Self::Error> {
            unimplemented!()
        }

        mock_method!(read(&self, id: Id<TestAggregate>) -> Result<Self::Events, Self::Error>);
    }

    #[test]
    fn replay_aggregate() {
        let read = Ok(vec![
            VersionedEvent {
                version: Version(1),
                event: TestEvent::Increased,
            },
            VersionedEvent {
                version: Version(2),
                event: TestEvent::Increased,
            },
        ]);
        let storage = MockEventStorage1::new(read);

        let id = Id::new();
        let got = storage.replay_aggregate(id);

        assert!(got.is_ok());
        assert!(storage.read.has_calls_exactly(vec![id]));
    }

    #[test]
    fn replay_aggregate_event_storage_error() {
        let storage = MockEventStorage1::new(Err(MockStorageError {}));
        let id = Id::new();
        if let Err(ReplayAggregateError::EventStorage(_)) = storage.replay_aggregate(id) {
        } else {
            panic!();
        }
    }

    #[test]
    fn replay_aggregate_version_inconsistent() {
        let read = Ok(vec![VersionedEvent {
            version: Version(99),
            event: TestEvent::Increased,
        }]);
        let storage = MockEventStorage1::new(read);

        let id = Id::new();
        if let Err(ReplayAggregateError::VersionInconsistent) = storage.replay_aggregate(id) {
        } else {
            panic!();
        }
    }

    #[rustfmt::skip]
    mock_trait_no_default!(
        MockEventStorage2,
        replay_aggregate(Id<TestAggregate>) -> Result<VersionedAggregate<TestAggregate>, ReplayAggregateError<MockStorageError>>,
        insert(Id<TestAggregate>, TestEvent) -> Result<(), MockStorageError>
    );

    impl EventStorage<TestAggregate> for MockEventStorage2 {
        type Events = Vec<VersionedEvent<TestAggregate>>;
        type Error = MockStorageError;

        fn read(&self, id: Id<TestAggregate>) -> Result<Self::Events, Self::Error> {
            unimplemented!()
        }
        mock_method!(replay_aggregate(&self, id: Id<TestAggregate>) -> Result<VersionedAggregate<TestAggregate>, ReplayAggregateError<Self::Error>>);
        mock_method!(insert(&mut self, id: Id<TestAggregate>, event: TestEvent) -> Result<(), Self::Error>);
    }

    #[test]
    fn execute_command() {
        let mut storage = MockEventStorage2::new(
            Ok(VersionedAggregate {
                version: Version(1),
                aggregate: TestAggregate(1),
            }),
            Ok(()),
        );
        let id = Id::new();
        let cmd = TestCommand::Increase;

        let got = storage.execute_command(id, cmd);
        assert_eq!(got, Ok(()))
    }

    #[test]
    fn execute_command_replay_aggregate_error() {
        let replay_aggregate_error = ReplayAggregateError::VersionInconsistent;
        let mut storage = MockEventStorage2::new(Err(replay_aggregate_error.clone()), Ok(()));
        let id = Id::new();
        let cmd = TestCommand::Increase;

        if let Err(ExecuteCommandError::ReplayAggregate(e)) = storage.execute_command(id, cmd) {
            assert_eq!(e, replay_aggregate_error);
        } else {
            panic!();
        };
    }

    #[test]
    fn execute_command_command_error() {
        let command_error = TestCommandError::Invalid;
        let mut storage = MockEventStorage2::new(
            Ok(VersionedAggregate {
                version: Version(1),
                aggregate: TestAggregate(1),
            }),
            Ok(()),
        );
        let id = Id::new();
        let cmd = TestCommand::Invalid;

        if let Err(ExecuteCommandError::Command(e)) = storage.execute_command(id, cmd) {
            assert_eq!(e, command_error);
        } else {
            panic!();
        };
    }

    #[test]
    fn execute_command_insert_error() {
        let storage_error = MockStorageError {};
        let mut storage = MockEventStorage2::new(
            Ok(VersionedAggregate {
                version: Version(1),
                aggregate: TestAggregate(1),
            }),
            Err(storage_error.clone()),
        );
        let id = Id::new();
        let cmd = TestCommand::Increase;

        if let Err(ExecuteCommandError::Insert(e)) = storage.execute_command(id, cmd) {
            assert_eq!(e, storage_error);
        }
    }
}
