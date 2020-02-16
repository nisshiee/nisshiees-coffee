use crate::store::Version;
use crate::tests::test_aggregate::{TestAggregate, TestEvent};

use super::VersionedEvent;

#[test]
fn versioned_event_serde() {
    let before = VersionedEvent::<TestAggregate> {
        version: Version(123),
        event: TestEvent::Increased,
    };
    let json = serde_json::to_string(&before).unwrap();
    println!("{}", json);
    let after: VersionedEvent<TestAggregate> = serde_json::from_str(&json).unwrap();
    assert_eq!(before.version, after.version);
    assert_eq!(before.event, after.event);
}
