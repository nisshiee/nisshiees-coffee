extern crate failure;
extern crate uuid;

extern crate cqrs_es;
extern crate eventstorage_file;

use cqrs_es::store::*;
use cqrs_es::*;

mod common;
use common::*;
use eventstorage_file::FileEventStorage;

#[test]
fn it_works() {
    let ctx = TestContext::new();
    let mut storage = FileEventStorage::<TestAggregate, TestEvent>::new(ctx.dir()).unwrap();

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
