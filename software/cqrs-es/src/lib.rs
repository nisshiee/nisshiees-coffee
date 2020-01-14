extern crate failure;

use std::fmt::Debug;
use failure::Fail;

pub trait Aggregate: Default {
    type Id: AggregateId<Self>;
    type Event: Event<Self>;
    type Command: Command<Self>;
}

pub trait AggregateId<A: Aggregate>: Debug + Copy + Clone + Eq + PartialEq {}

pub trait Event<A: Aggregate> {
    fn apply_to(self, aggregate: &mut A);
}

pub trait Command<A: Aggregate> {
    type Events: IntoIterator<Item = A::Event>;
    type Error: Fail;

    fn execute_on(self, aggregate: &A) -> Result<Self::Events, Self::Error>;
}

pub trait EventStorage<A: Aggregate> {
    type Events: IntoIterator<Item = A::Event>;
    type Error: Fail;

    fn insert(&self, id: A::Id, event: A::Event) -> Result<(), Self::Error>;
    fn read(&self, id: A::Id) -> Result<Self::Events, Self::Error>;
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
