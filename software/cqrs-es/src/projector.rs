use crate::store::{VersionedAggregate, VersionedEvent};
use crate::*;

pub trait Projector<A: Aggregate> {
    fn project(&mut self, id: Id<A>, event: &VersionedEvent<A>, aggregate: &VersionedAggregate<A>);
}
