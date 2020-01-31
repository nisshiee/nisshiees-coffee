extern crate failure;
extern crate uuid;

pub mod store;

use failure::Fail;
use failure::_core::fmt::{Error, Formatter};
use failure::_core::marker::PhantomData;
use std::fmt::Debug;
use std::hash::{Hash, Hasher};
use uuid::Uuid;

pub trait Aggregate: Default {
    type Event: Event<Self>;
    type Command: Command<Self>;

    // `/` で区切ることで階層化することを想定
    fn type_name() -> &'static str;
}

#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq)]
pub struct Version(u64);

#[derive(Debug)]
pub struct VersionedAggregate<A: Aggregate> {
    version: Version,
    aggregate: A,
}

pub struct Id<A: Aggregate> {
    id: Uuid,
    phantom: PhantomData<A>,
}

impl<A: Aggregate> PartialEq for Id<A> {
    fn eq(&self, other: &Self) -> bool {
        self.id == other.id
    }
}
impl<A: Aggregate> Eq for Id<A> {}
impl<A: Aggregate> Hash for Id<A> {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.id.hash(state)
    }
}


impl<A: Aggregate> Debug for Id<A> {
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), Error> {
        write!(f, "{:?}", self.id)
    }
}

impl<A: Aggregate> Clone for Id<A> {
    fn clone(&self) -> Self {
        Id {
            id: self.id,
            phantom: PhantomData,
        }
    }
}
impl<A: Aggregate> Copy for Id<A> {}

impl<A: Aggregate> ToString for Id<A> {
    fn to_string(&self) -> String {
        self.id.to_string()
    }
}

pub trait Event<A: Aggregate>: Debug + Clone {
    fn apply_to(self, aggregate: &mut A);
}

pub trait CommandError: Fail {}

pub trait Command<A: Aggregate> {
    type Events: IntoIterator<Item = A::Event>;
    type Error: CommandError;

    fn execute_on(self, aggregate: &A) -> Result<Self::Events, Self::Error>;
}

pub trait Projector<A: Aggregate>: Debug {
    fn project(&mut self, id: Id<A>, event: &A::Event);
}

#[cfg(test)]
mod tests {
    pub mod test_aggregate;
    pub mod onmemory_storage;

    use crate::*;
    use test_aggregate::*;

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
}
