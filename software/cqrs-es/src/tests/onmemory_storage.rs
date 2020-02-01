use crate::store::{EventStorage, EventStorageError};
use crate::{Aggregate, Id};
use failure::Fail;
use std::collections::HashMap;

pub struct OnmemoryStorage<A: Aggregate> {
    events: HashMap<Id<A>, Vec<A::Event>>,
}

#[derive(Fail, Debug)]
#[fail(display = "unexpected")]
pub struct OnmemoryStorageError {}

impl EventStorageError for OnmemoryStorageError {}

impl<A: Aggregate> EventStorage<A> for OnmemoryStorage<A> {
    type Events = Vec<A::Event>;
    type Error = OnmemoryStorageError;

    fn insert(&mut self, id: Id<A>, event: A::Event) -> Result<(), Self::Error> {
        let seq = self.events.entry(id).or_insert(Vec::new());
        seq.push(event);
        Ok(())
    }

    fn read(&self, id: Id<A>) -> Result<Self::Events, Self::Error> {
        let seq = self.events.get(&id).unwrap_or(&Vec::new()).to_vec();
        Ok(seq)
    }
}
