use crate::tests::test_aggregate::*;
use crate::Command;

#[test]
fn execute_on() {
    let command = TestCommand::Increase;
    let aggregate = TestAggregate(0);
    let got = command.execute_on(&aggregate);
    assert_eq!(got, Ok(Some(TestEvent::Increased)));
}
