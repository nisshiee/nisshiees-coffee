use crate::*;

/// # Examples
///
/// ```
/// use cqrs_es::version::Version;
/// let current = Version::default();
/// let next = current.next();
/// assert!(next.is_next_of(&current));
/// ```
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default)]
pub struct Version(pub u64);

impl Version {
    pub fn is_next_of(&self, other: &Version) -> bool {
        self.0 == other.0 + 1
    }

    pub fn next(&self) -> Version {
        Version(self.0 + 1)
    }
}

#[derive(Debug, Clone, Default)]
pub struct VersionedAggregate<A: Aggregate> {
    pub version: Version,
    pub aggregate: A,
}

#[derive(Debug, Clone)]
pub struct VersionedEvent<A: Aggregate> {
    pub version: Version,
    pub event: A::Event,
}
