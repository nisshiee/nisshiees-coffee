use crate::version::*;
use crate::*;
use failure::Fail;

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

pub trait EventStorage<A: Aggregate> {
    type Events: IntoIterator<Item = VersionedEvent<A>>;
    type Error: EventStorageError;

    fn insert(&mut self, id: Id<A>, event: VersionedEvent<A>) -> Result<(), Self::Error>;

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

#[cfg(test)]
mod tests {
    use crate::store::*;
    use crate::tests::test_aggregate::*;

    use simulacrum::*;

    #[derive(Fail, Debug, Eq, PartialEq)]
    #[fail(display = "unexpected")]
    pub struct MockStorageError {}

    impl EventStorageError for MockStorageError {}

    create_mock_struct! {
        struct MockEventStorage1: {
            expect_read("read") Id<TestAggregate> => Result<Vec<VersionedEvent<TestAggregate>>, MockStorageError>;
        }
    }

    impl EventStorage<TestAggregate> for MockEventStorage1 {
        type Events = Vec<VersionedEvent<TestAggregate>>;
        type Error = MockStorageError;

        fn insert(
            &mut self,
            _id: Id<TestAggregate>,
            _event: VersionedEvent<TestAggregate>,
        ) -> Result<(), Self::Error> {
            unimplemented!()
        }

        fn read(&self, id: Id<TestAggregate>) -> Result<Self::Events, Self::Error> {
            was_called!(self, "read", (id: Id<TestAggregate>) -> Result<Self::Events, Self::Error>)
        }
    }

    #[test]
    fn replay_aggregate() {
        let mut storage = MockEventStorage1::new();
        storage.expect_read().called_once().returning(|_| {
            Ok(vec![
                VersionedEvent {
                    version: Version(1),
                    event: TestEvent::Increased,
                },
                VersionedEvent {
                    version: Version(2),
                    event: TestEvent::Increased,
                },
            ])
        });

        let id = Id::new();
        let got = storage.replay_aggregate(id);

        assert!(got.is_ok());
    }

    #[test]
    fn replay_aggregate_event_storage_error() {
        let mut storage = MockEventStorage1::new();
        storage
            .expect_read()
            .called_once()
            .returning(|_| Err(MockStorageError {}));

        let id = Id::new();
        if let Err(ReplayAggregateError::Read(_)) = storage.replay_aggregate(id) {
        } else {
            panic!();
        }
    }

    #[test]
    fn replay_aggregate_version_inconsistent() {
        let mut storage = MockEventStorage1::new();
        storage.expect_read().called_once().returning(|_| {
            Ok(vec![VersionedEvent {
                version: Version(99),
                event: TestEvent::Increased,
            }])
        });

        let id = Id::new();
        if let Err(ReplayAggregateError::VersionInconsistent) = storage.replay_aggregate(id) {
        } else {
            panic!();
        }
    }

    create_mock_struct! {
        struct MockEventStorage22: {
            expect_replay_aggregate("replay_aggregate") Id<TestAggregate> => Result<VersionedAggregate<TestAggregate>, ReplayAggregateError<MockStorageError>>;
            expect_insert("insert") (Id<TestAggregate>, VersionedEvent<TestAggregate>) => Result<(), MockStorageError>;
        }
    }

    impl EventStorage<TestAggregate> for MockEventStorage22 {
        type Events = Vec<VersionedEvent<TestAggregate>>;
        type Error = MockStorageError;

        fn insert(
            &mut self,
            id: Id<TestAggregate>,
            event: VersionedEvent<TestAggregate>,
        ) -> Result<(), Self::Error> {
            was_called!(self, "insert", (id: Id<TestAggregate>, event: VersionedEvent<TestAggregate>) -> Result<(), MockStorageError>)
        }

        fn read(&self, _id: Id<TestAggregate>) -> Result<Self::Events, Self::Error> {
            unimplemented!()
        }

        fn replay_aggregate(
            &self,
            id: Id<TestAggregate>,
        ) -> Result<VersionedAggregate<TestAggregate>, ReplayAggregateError<MockStorageError>>
        {
            was_called!(
                self,
                "replay_aggregate",
                (id: Id<TestAggregate>) ->
                    Result<VersionedAggregate<TestAggregate>, ReplayAggregateError<MockStorageError>>
            )
        }
    }

    #[test]
    fn execute_command() {
        let mut storage = MockEventStorage22::new();
        storage
            .expect_replay_aggregate()
            .called_once()
            .returning(|_| {
                Ok(VersionedAggregate {
                    version: Version(1),
                    aggregate: TestAggregate(1),
                })
            });
        let version_check =
            passes::<(Id<TestAggregate>, VersionedEvent<TestAggregate>), _>(|event| {
                event.1.version.is_next_of(&Version(1))
            });
        storage
            .expect_insert()
            .called_once()
            .with(version_check)
            .returning(|_| Ok(()));

        let id = Id::new();
        let cmd = TestCommand::Increase;

        let got = storage.execute_command(id, cmd);
        assert_eq!(got, Ok(()))
    }

    #[test]
    fn execute_command_replay_aggregate_error() {
        let mut storage = MockEventStorage22::new();
        storage
            .expect_replay_aggregate()
            .called_once()
            .returning(|_| Err(ReplayAggregateError::VersionInconsistent));

        let id = Id::new();
        let cmd = TestCommand::Increase;

        if let Err(ExecuteCommandError::ReplayAggregate(e)) = storage.execute_command(id, cmd) {
            assert_eq!(e, ReplayAggregateError::VersionInconsistent);
        } else {
            panic!();
        };
    }

    #[test]
    fn execute_command_command_error() {
        let mut storage = MockEventStorage22::new();
        storage
            .expect_replay_aggregate()
            .called_once()
            .returning(|_| {
                Ok(VersionedAggregate {
                    version: Version(1),
                    aggregate: TestAggregate(1),
                })
            });

        let id = Id::new();
        let cmd = TestCommand::Invalid;

        if let Err(ExecuteCommandError::Command(e)) = storage.execute_command(id, cmd) {
            assert_eq!(e, TestCommandError::Invalid);
        } else {
            panic!();
        };
    }

    #[test]
    fn execute_command_insert_error() {
        let mut storage = MockEventStorage22::new();
        storage
            .expect_replay_aggregate()
            .called_once()
            .returning(|_| {
                Ok(VersionedAggregate {
                    version: Version(1),
                    aggregate: TestAggregate(1),
                })
            });
        storage
            .expect_insert()
            .called_once()
            .returning(|_| Err(MockStorageError {}));

        let id = Id::new();
        let cmd = TestCommand::Increase;

        if let Err(ExecuteCommandError::Insert(e)) = storage.execute_command(id, cmd) {
            assert_eq!(e, MockStorageError {});
        } else {
            panic!();
        }
    }
}
