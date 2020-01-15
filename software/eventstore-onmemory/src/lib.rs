extern crate failure;
#[macro_use] extern crate failure_derive;

use cqrs_es::*;
use std::collections::HashMap;

pub struct OnMemoryEventStorage<A: Aggregate> {
    events: HashMap<A::Id, Vec<A::Event>>,
}

#[derive(Fail, Debug)]
pub enum OnMemoryEventStorageError {
    #[fail(display = "Unexpected error")]
    Unexpected,
}

impl <A: Aggregate> EventStorage<A> for OnMemoryEventStorage<A> {
    type Events = Vec<A::Event>;
    type Error = OnMemoryEventStorageError;

    fn insert(&mut self, id: A::Id, event: A::Event) -> Result<(), Self::Error> {
        let seq = self.events.entry(id).or_insert(Vec::new());
        seq.push(event);
        Ok(())
    }

    fn read(&self, id: A::Id) -> Result<Self::Events, Self::Error> {
        let seq = self.events.get(&id).unwrap_or(&Vec::new()).to_vec();
        Ok(seq)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        assert_eq!(2 + 2, 4);
    }
}
