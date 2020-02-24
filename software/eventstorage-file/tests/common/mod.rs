use std::fs;
use std::path::PathBuf;

use failure::Fail;
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use cqrs_es::*;

pub struct TestContext {
    dir: PathBuf,
}

impl TestContext {
    pub fn new() -> TestContext {
        let mut dir = PathBuf::new();
        dir.push("target");
        dir.push("tests");
        dir.push(Uuid::new_v4().to_string());
        TestContext { dir }
    }

    pub fn dir(&self) -> PathBuf {
        self.dir.clone()
    }
}

impl Drop for TestContext {
    fn drop(&mut self) {
        fs::remove_dir_all(self.dir.as_path()).unwrap();
    }
}

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

pub struct TestCommand {}

#[derive(Fail, Debug, Eq, PartialEq)]
#[fail(display = "Invalid")]
pub struct TestCommandError {
    // Invalid,
}

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
