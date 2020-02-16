use serde::{Deserialize, Serialize};

/// EventSourcingの基盤となる、AggregateやEventの「バージョン」を表す
///
/// # Examples
///
/// ```
/// use cqrs_es::store::version::Version;
/// let current = Version::default();
/// let next = current.next();
/// assert!(next.is_next_of(&current));
/// ```
#[derive(Debug, Copy, Clone, Ord, PartialOrd, Eq, PartialEq, Default, Serialize, Deserialize)]
pub struct Version(pub u64);

impl Into<u64> for Version {
    fn into(self) -> u64 {
        self.0
    }
}

impl Version {
    pub fn is_next_of(&self, other: &Version) -> bool {
        self.0 == other.0 + 1
    }

    pub fn next(&self) -> Version {
        Version(self.0 + 1)
    }
}
