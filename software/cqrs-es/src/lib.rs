extern crate failure;
extern crate serde;

use failure::Fail;
use serde::de::DeserializeOwned;
use serde::Serialize;
use std::fmt::Debug;
use std::hash::Hash;

pub trait Aggregate: Default {
    type Id: AggregateId<Self>;
    type Event: Event<Self>;
    type Command: Command<Self>;

    // `/` で区切ることで階層化することを想定
    fn type_name() -> &'static str;
}

pub trait AggregateId<A: Aggregate>:
    Debug + Copy + Clone + Eq + PartialEq + Hash + ToString
{
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

    fn insert(&mut self, id: A::Id, event: A::Event) -> Result<(), Self::Error>;

    // FIXME: Eventsではなく&Eventsを返すことで、不要なメモリコピーを抑制できる気がする
    fn read(&self, id: A::Id) -> Result<Self::Events, Self::Error>;

    fn replay_aggregate(&self, id: A::Id) -> Result<A, Self::Error> {
        let mut aggregate = A::default();
        let events = self.read(id)?;
        events.into_iter().for_each(|e| e.apply_to(&mut aggregate));
        Ok(aggregate)
    }

    fn execute_command<C: Command<A>>(
        &mut self,
        id: A::Id,
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
    fn project(&mut self, id: A::Id, event: &A::Event);
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
