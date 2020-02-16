use std::fmt::{Error as FmtError, Formatter};
use std::marker::PhantomData;

use serde::de::{MapAccess, Visitor};
use serde::ser::SerializeMap;
use serde::{Deserialize, Deserializer, Serialize, Serializer};

use crate::store::version::Version;
use crate::Aggregate;

use super::serde::*;

#[cfg(test)]
mod tests;

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

    fn expecting(&self, formatter: &mut Formatter) -> Result<(), FmtError> {
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
