extern crate failure;
extern crate serde;
extern crate uuid;

#[cfg(test)]
#[macro_use]
extern crate failure_derive;

use failure::Fail;
use failure::_core::marker::PhantomData;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use std::fmt::Debug;
use std::hash::Hash;
use uuid::Uuid;

pub trait Aggregate: Default + Clone {
    type Event: Event<Self>;
    type Command: Command<Self>;

    // `/` で区切ることで階層化することを想定
    fn type_name() -> &'static str;
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Serialize, Deserialize)]
pub struct Version(u64);

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VersionedAggregate<A: Aggregate> {
    version: Version,
    aggregate: A,
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub struct Id<A: Aggregate> {
    id: Uuid,
    phantom: PhantomData<A>,
}

impl<A: Aggregate> Copy for Id<A> {}

impl<A: Aggregate> ToString for Id<A> {
    fn to_string(&self) -> String {
        self.id.to_string()
    }
}

pub trait Event<A: Aggregate>: Debug + Clone + Serialize + DeserializeOwned {
    fn apply_to(self, aggregate: &mut A);
}

pub trait CommandError: Fail {}

pub trait Command<A: Aggregate> {
    type Events: IntoIterator<Item = A::Event>;
    type Error: CommandError;

    fn execute_on(self, aggregate: &A) -> Result<Self::Events, Self::Error>;
}

pub trait EventStorageError: Fail {}

pub trait EventStorage<A: Aggregate> {
    type Events: IntoIterator<Item = A::Event>;
    type Error: EventStorageError;

    fn insert(&mut self, id: Id<A>, event: A::Event) -> Result<(), Self::Error>;

    // FIXME: Eventsではなく&Eventsを返すことで、不要なメモリコピーを抑制できる気がする
    fn read(&self, id: Id<A>) -> Result<Self::Events, Self::Error>;

    fn replay_aggregate(&self, id: Id<A>) -> Result<A, Self::Error> {
        let mut aggregate = A::default();
        let events = self.read(id)?;
        events.into_iter().for_each(|e| e.apply_to(&mut aggregate));
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
        let aggregate = self.replay_aggregate(id);
        let aggregate = aggregate.map_err(|e| ExecuteCommandError::Storage(e))?;

        let events = command.execute_on(&aggregate);
        let events = events.map_err(|e| ExecuteCommandError::Command(e))?;

        let ret = events.into_iter().try_for_each(|e| self.insert(id, e));
        ret.map_err(|e| ExecuteCommandError::Storage(e))
    }
}

#[derive(Fail, Debug)]
pub enum ExecuteCommandError<S: EventStorageError, C: CommandError> {
    #[fail(display = "Storage error: {}", _0)]
    Storage(S),
    #[fail(display = "Command error: {}", _0)]
    Command(C),
}

pub trait Projector<A: Aggregate>: Debug {
    fn project(&mut self, id: Id<A>, event: &A::Event);
}

#[cfg(test)]
mod tests {
    use crate::{Aggregate, Command, CommandError, Event};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone)]
    struct TestAggregate(u64);

    impl Default for TestAggregate {
        fn default() -> Self {
            TestAggregate(0)
        }
    }

    #[derive(Debug, Clone, Eq, PartialEq, Serialize, Deserialize)]
    enum TestEvent {
        Increased,
    }

    impl Event<TestAggregate> for TestEvent {
        fn apply_to(self, aggregate: &mut TestAggregate) {
            aggregate.0 = aggregate.0 + 1
        }
    }

    enum TestCommand {
        Increase,
    }

    #[derive(Fail, Debug)]
    #[fail(display = "unknown")]
    struct TestCommandError {}

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

    #[test]
    fn event() {
        let event = TestEvent::Increased;
        let mut aggregate = TestAggregate(0);
        event.apply_to(&mut aggregate);
        assert_eq!(aggregate.0, 1);
    }

    #[test]
    fn command() {
        let command = TestCommand::Increase;
        let aggregate = TestAggregate(0);
        if let Ok(Some(got)) = command.execute_on(&aggregate) {
            assert_eq!(got, TestEvent::Increased);
        } else {
            assert!(false)
        }
    }

    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
