use simulacrum::*;

use crate::store::*;
use crate::tests::test_aggregate::*;
use crate::Id;

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
    ) -> Result<VersionedAggregate<TestAggregate>, ReplayAggregateError<MockStorageError>> {
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
    let version_check = passes::<(Id<TestAggregate>, VersionedEvent<TestAggregate>), _>(|event| {
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
