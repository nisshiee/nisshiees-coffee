use std::fmt::{Debug, Error as FmtError, Formatter};
use std::hash::{Hash, Hasher};
use std::marker::PhantomData;

use uuid::Uuid;

use crate::Aggregate;

pub struct Id<A: Aggregate> {
    id: Uuid,
    phantom: PhantomData<A>,
}

impl<A: Aggregate> Id<A> {
    pub fn new() -> Id<A> {
        Id {
            id: Uuid::new_v4(),
            phantom: PhantomData,
        }
    }
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
    fn fmt(&self, f: &mut Formatter<'_>) -> Result<(), FmtError> {
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
