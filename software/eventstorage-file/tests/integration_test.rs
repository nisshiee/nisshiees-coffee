extern crate failure;
extern crate simulacrum;
extern crate uuid;

extern crate cqrs_es;
extern crate eventstorage_file;

use simulacrum::*;

use cqrs_es::projector::*;
use cqrs_es::store::*;
use cqrs_es::*;

mod common;
use common::*;
use eventstorage_file::FileEventStorage;

create_mock_struct! {
    struct TestProjector: {
        expect_project("project") (Id<TestAggregate>, *const VersionedEvent<TestAggregate>, *const VersionedAggregate<TestAggregate>);
    }
}

impl Projector<TestAggregate> for TestProjector {
    fn project(
        &mut self,
        id: Id<TestAggregate>,
        event: &VersionedEvent<TestAggregate>,
        aggregate: &VersionedAggregate<TestAggregate>,
    ) {
        was_called!(
            self,
            "project",
            (
                id: Id<TestAggregate>,
                event: *const VersionedEvent<TestAggregate>,
                aggregate: *const VersionedAggregate<TestAggregate>
            )
        );
    }
}

#[test]
fn it_works() {
    let ctx = TestContext::new();
    let mut storage = FileEventStorage::<TestAggregate, TestEvent>::new(ctx.dir()).unwrap();

    let mut projector = TestProjector::new();
    projector.expect_project().called_times(2);
    storage.add_projector(&mut projector);

    let id = Id::<TestAggregate>::new();
    storage
        .insert(
            id,
            VersionedEvent {
                version: Version(1),
                event: TestEvent::Increased,
            },
        )
        .unwrap();
    storage
        .insert(
            id,
            VersionedEvent {
                version: Version(2),
                event: TestEvent::Increased,
            },
        )
        .unwrap();

    let mut events = storage.read(id).unwrap();
    assert_eq!(events.len(), 2);

    let VersionedEvent { version, event } = events.remove(0);
    assert_eq!(version, Version(1));
    assert_eq!(event, TestEvent::Increased);

    let VersionedEvent { version, event } = events.remove(0);
    assert_eq!(version, Version(2));
    assert_eq!(event, TestEvent::Increased);
}
