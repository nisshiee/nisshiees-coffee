use std::fmt::{Error as FmtError, Formatter};
use std::marker::PhantomData;

use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::store::Version;
use crate::{Aggregate, Event};

use super::serde::*;

#[cfg(test)]
mod tests;

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

    fn expecting(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
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
