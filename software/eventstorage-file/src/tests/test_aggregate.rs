use cqrs_es::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct TestAggregate(pub u64);

impl Default for TestAggregate {
    fn default() -> Self {
        TestAggregate(0)
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Hash, Serialize, Deserialize)]
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
    Invalid,
}

#[derive(Fail, Debug, Eq, PartialEq)]
pub enum TestCommandError {
    #[fail(display = "Invalid")]
    Invalid,
}

impl CommandError for TestCommandError {}

impl Command<TestAggregate> for TestCommand {
    type Events = Option<TestEvent>;
    type Error = TestCommandError;

    fn execute_on(self, _aggregate: &TestAggregate) -> Result<Self::Events, Self::Error> {
        match self {
            TestCommand::Increase => Ok(Some(TestEvent::Increased)),
            TestCommand::Invalid => Err(TestCommandError::Invalid),
        }
    }
}

impl Aggregate for TestAggregate {
    type Event = TestEvent;
    type Command = TestCommand;

    fn type_name() -> &'static str {
        "test"
    }
}
