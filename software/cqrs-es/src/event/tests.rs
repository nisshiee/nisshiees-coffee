use crate::tests::test_aggregate::*;
use crate::Event;

#[test]
fn apply_to() {
    let event = TestEvent::Increased;
    let mut aggregate = TestAggregate(0);
    event.apply_to(&mut aggregate);
    assert_eq!(aggregate, TestAggregate(1));
}
