use serde::{Deserialize, Deserializer};
use uuid::Uuid;
mod seq;
mod seq_owned;
use eva_common::value::Value;

pub use seq::{Sequence, SequenceAction, SequenceActionEntry, SequenceEntry};
pub use seq_owned::{
    SequenceActionEntryOwned, SequenceActionOwned, SequenceEntryOwned, SequenceOwned,
};

fn deserialize_uuid<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
where
    D: Deserializer<'de>,
{
    let val: Value = Deserialize::deserialize(deserializer)?;
    Uuid::deserialize(val).map_err(serde::de::Error::custom)
}
