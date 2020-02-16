use serde::de::MapAccess;
use serde::Deserialize;

use super::Version;

pub const SERIALIZE_KEY_VERSION: &str = "version";
pub const SERIALIZE_KEY_AGGREGATE: &str = "aggregate";
pub const SERIALIZE_KEY_EVENT: &str = "event";

pub fn visit_versioned<'de, B, V, F, M>(
    mut map: M,
    base_key: &'static str,
    f: F,
) -> Result<V, M::Error>
where
    B: Deserialize<'de>,
    F: (FnOnce(Version, B) -> V),
    M: MapAccess<'de>,
{
    let mut version = None;
    let mut base = None;

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
