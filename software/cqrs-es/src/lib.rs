extern crate failure;

use failure::Fail;
use std::fmt::Debug;
use std::hash::Hash;

pub trait Aggregate: Default {
    type Id: AggregateId<Self>;
    type Event: Event<Self>;
    type Command: Command<Self>;
}

pub trait AggregateId<A: Aggregate>: Debug + Copy + Clone + Eq + PartialEq + Hash {}

pub trait Event<A: Aggregate>: Debug + Clone {
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

    fn insert(&mut self, id: A::Id, event: A::Event) -> Result<(), Self::Error>;

    // FIXME: Eventsではなく&Eventsを返すことで、不要なメモリコピーを抑制できる気がする
    fn read(&self, id: A::Id) -> Result<Self::Events, Self::Error>;

    fn replay_aggregate(&self, id: A::Id) -> Result<A, Self::Error> {
        let mut aggregate = A::default();
        let events = self.read(id)?;
        events.into_iter().for_each(|e| e.apply_to(&mut aggregate));
        Ok(aggregate)
    }
}

pub trait Projector<A: Aggregate>: Debug {
    fn project(&mut self, id: A::Id, event: &A::Event);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
