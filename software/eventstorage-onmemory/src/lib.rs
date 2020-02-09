extern crate failure;
#[macro_use]
extern crate failure_derive;

use cqrs_es::projector::*;
use cqrs_es::store::*;
use cqrs_es::version::*;
use cqrs_es::*;
use std::collections::HashMap;

pub struct OnMemoryEventStorage<A: Aggregate> {
    events: HashMap<Id<A>, Vec<VersionedEvent<A>>>,
    projectors: Vec<Box<dyn Projector<A>>>,
}

impl<A: Aggregate> OnMemoryEventStorage<A> {
    pub fn new() -> OnMemoryEventStorage<A> {
        OnMemoryEventStorage {
            events: HashMap::new(),
            projectors: Vec::new(),
        }
    }

    pub fn add_projector<P: Projector<A> + 'static>(&mut self, projector: P) {
        let b = Box::new(projector);
        self.projectors.push(b)
    }
}

#[derive(Fail, Debug)]
pub enum OnMemoryEventStorageError {
    #[fail(display = "Unexpected error")]
    Unexpected,
}

impl EventStorageError for OnMemoryEventStorageError {}

impl<A: Aggregate> EventStorage<A> for OnMemoryEventStorage<A> {
    type Events = Vec<VersionedEvent<A>>;
    type Error = OnMemoryEventStorageError;

    fn insert(&mut self, id: Id<A>, event: VersionedEvent<A>) -> Result<(), Self::Error> {
        let seq = self.events.entry(id).or_insert(Vec::new());
        seq.push(event.clone());
        self.projectors
            .iter_mut()
            .for_each(|p| p.project(id, &event));
        Ok(())
    }

    fn read(&self, id: Id<A>) -> Result<Self::Events, Self::Error> {
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
