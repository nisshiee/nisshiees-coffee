use crate::*;
use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

/// EventSourcingの基盤となる、AggregateやEventの「バージョン」を表す
///
/// # Examples
///
/// ```
/// use cqrs_es::version::Version;
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

const SERIALIZE_KEY_VERSION: &str = "version";
const SERIALIZE_KEY_AGGREGATE: &str = "aggregate";
const SERIALIZE_KEY_EVENT: &str = "event";

/// バージョン付けされたAggregate
#[derive(Debug, Clone, Default)]
pub struct VersionedAggregate<A: Aggregate> {
    pub version: Version,
    pub aggregate: A,
}

impl<A> Serialize for VersionedAggregate<A>
where
    A: Aggregate + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry(SERIALIZE_KEY_VERSION, &self.version)?;
        map.serialize_entry(SERIALIZE_KEY_AGGREGATE, &self.aggregate)?;
        map.end()
    }
}

struct VersionedAggregateVisitor<A> {
    phantom: PhantomData<A>,
}

impl<'de, A> VersionedAggregateVisitor<A>
where
    A: Aggregate + Deserialize<'de>,
{
    fn new() -> VersionedAggregateVisitor<A> {
        VersionedAggregateVisitor {
            phantom: PhantomData,
        }
    }
}

impl<'de, A> Visitor<'de> for VersionedAggregateVisitor<A>
where
    A: Aggregate + Deserialize<'de>,
{
    type Value = VersionedAggregate<A>;

    fn expecting(&self, formatter: &mut Formatter) -> Result<(), Error> {
        formatter.write_str("struct VersionedAggregate")
    }

    fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        visit_versioned(map, SERIALIZE_KEY_AGGREGATE, |version, aggregate| {
            VersionedAggregate { version, aggregate }
        })
    }
}

impl<'de, A> Deserialize<'de> for VersionedAggregate<A>
where
    A: Aggregate + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(VersionedAggregateVisitor::new())
    }
}

/// バージョン付けされたEvent
#[derive(Debug, Clone)]
pub struct VersionedEvent<A: Aggregate> {
    pub version: Version,
    pub event: A::Event,
}

impl<A, E> Serialize for VersionedEvent<A>
where
    A: Aggregate<Event = E>,
    E: Event<A> + Serialize,
{
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut map = serializer.serialize_map(Some(2))?;
        map.serialize_entry(SERIALIZE_KEY_VERSION, &self.version)?;
        map.serialize_entry(SERIALIZE_KEY_EVENT, &self.event)?;
        map.end()
    }
}

struct VersionedEventVisitor<A, E> {
    phantom: PhantomData<(A, E)>,
}

impl<'de, A, E> VersionedEventVisitor<A, E>
where
    A: Aggregate<Event = E>,
    E: Event<A> + Deserialize<'de>,
{
    fn new() -> VersionedEventVisitor<A, E> {
        VersionedEventVisitor {
            phantom: PhantomData,
        }
    }
}

impl<'de, A, E> Visitor<'de> for VersionedEventVisitor<A, E>
where
    A: Aggregate<Event = E>,
    E: Event<A> + Deserialize<'de>,
{
    type Value = VersionedEvent<A>;

    fn expecting(&self, formatter: &mut Formatter) -> Result<(), Error> {
        formatter.write_str("struct VersionedEvent")
    }

    fn visit_map<M>(self, map: M) -> Result<Self::Value, M::Error>
    where
        M: MapAccess<'de>,
    {
        visit_versioned(map, SERIALIZE_KEY_EVENT, |version, event| VersionedEvent {
            version,
            event,
        })
    }
}

impl<'de, A, E> Deserialize<'de> for VersionedEvent<A>
where
    A: Aggregate<Event = E>,
    E: Event<A> + Deserialize<'de>,
{
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_map(VersionedEventVisitor::new())
    }
}

fn visit_versioned<'de, B, V, F, M>(mut map: M, base_key: &'static str, f: F) -> Result<V, M::Error>
where
    B: Deserialize<'de>,
    F: (FnOnce(Version, B) -> V),
    M: MapAccess<'de>,
{
    let mut version = None;
    let mut base = None;

    // TODO: &strじゃむり・・・？
    while let Some(key) = map.next_key::<String>()? {
        if key == SERIALIZE_KEY_VERSION {
            if version.is_some() {
                return Err(serde::de::Error::duplicate_field(SERIALIZE_KEY_VERSION));
            }
            version = Some(map.next_value()?);
        } else if key == base_key {
            if base.is_some() {
                return Err(serde::de::Error::duplicate_field(base_key));
            }
            base = Some(map.next_value()?);
        }
    }

    let version = version.ok_or_else(|| serde::de::Error::missing_field(SERIALIZE_KEY_VERSION))?;
    let base = base.ok_or_else(|| serde::de::Error::missing_field(base_key))?;

    Ok(f(version, base))
}

#[cfg(test)]
mod tests {
    use crate::tests::test_aggregate::*;
    use crate::version::*;

    #[test]
    fn versioned_aggregate_serde() {
        let before = VersionedAggregate {
            version: Version(123),
            aggregate: TestAggregate(456),
        };
        let json = serde_json::to_string(&before).unwrap();
        println!("{}", json);
        let after: VersionedAggregate<TestAggregate> = serde_json::from_str(&json).unwrap();
        assert_eq!(before.version, after.version);
        assert_eq!(before.aggregate, after.aggregate);
    }

    #[test]
    fn versioned_event_serde() {
        let before = VersionedEvent::<TestAggregate> {
            version: Version(123),
            event: TestEvent::Increased,
        };
        let json = serde_json::to_string(&before).unwrap();
        println!("{}", json);
        let after: VersionedEvent<TestAggregate> = serde_json::from_str(&json).unwrap();
        assert_eq!(before.version, after.version);
        assert_eq!(before.event, after.event);
    }
}
