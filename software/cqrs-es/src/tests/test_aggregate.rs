use crate::*;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TestAggregate(pub u64);

impl Default for TestAggregate {
    fn default() -> Self {
        TestAggregate(0)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum TestEvent {
    Increased,
}

impl Event<TestAggregate> for TestEvent {
    fn apply_to(self, aggregate: &mut TestAggregate) {
        aggregate.0 = aggregate.0 + 1
    }
}

pub enum TestCommand {
    Increase,
}

#[derive(Fail, Debug, Eq, PartialEq)]
#[fail(display = "unknown")]
pub struct TestCommandError {}

impl CommandError for TestCommandError {}

impl Command<TestAggregate> for TestCommand {
    type Events = Option<TestEvent>;
    type Error = TestCommandError;

    fn execute_on(self, _aggregate: &TestAggregate) -> Result<Self::Events, Self::Error> {
        Ok(Some(TestEvent::Increased))
    }
}

impl Aggregate for TestAggregate {
    type Event = TestEvent;
    type Command = TestCommand;

    fn type_name() -> &'static str {
        "test"
    }
}
