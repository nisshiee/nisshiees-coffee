use std::fmt::Debug;

use crate::Aggregate;

#[cfg(test)]
mod tests;

pub trait Event<A: Aggregate>: Debug + Clone {
    fn apply_to(self, aggregate: &mut A);
}
