use eva_common::prelude::*;
use serde::{Deserialize, Deserializer, Serialize};
use std::time::Duration;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct Sequence {
    #[serde(alias = "uuid", deserialize_with = "deserialize_uuid")]
    pub u: Uuid,
    pub seq: Vec<SequenceEntry>,
    pub on_abort: Option<SequenceActionEntry>,
    #[serde(
        deserialize_with = "eva_common::tools::de_float_as_duration",
        serialize_with = "eva_common::tools::serialize_duration_as_f64"
    )]
    pub timeout: Duration,
}

pub fn deserialize_uuid<'de, D>(deserializer: D) -> Result<Uuid, D::Error>
where
    D: Deserializer<'de>,
{
    let val: Value = Deserialize::deserialize(deserializer)?;
    Uuid::deserialize(val).map_err(serde::de::Error::custom)
}

impl Sequence {
    #[inline]
    pub fn new(timeout: Duration) -> Self {
        Self {
            u: Uuid::new_v4(),
            seq: <_>::default(),
            on_abort: <_>::default(),
            timeout,
        }
    }
    #[inline]
    pub fn uuid(&self) -> Uuid {
        self.u
    }
    #[inline]
    pub fn timeout(&self) -> Duration {
        self.timeout
    }
    #[inline]
    pub fn push_delay(&mut self, delay: Duration) {
        self.seq.push(SequenceEntry::Delay(delay.as_secs_f64()));
    }
    #[inline]
    pub fn push_action(&mut self, action: SequenceAction) {
        self.seq
            .push(SequenceEntry::Actions(SequenceActionEntry::Single(action)));
    }
    #[inline]
    pub fn push_actions_multi(&mut self, actions: Vec<SequenceAction>) {
        self.seq
            .push(SequenceEntry::Actions(SequenceActionEntry::Multi(actions)));
    }
    pub fn max_expected_duration(&self) -> Duration {
        let mut duration: Duration = Duration::from_secs(0);
        for s in &self.seq {
            match s {
                SequenceEntry::Delay(d) => duration += Duration::from_secs_f64(*d),
                SequenceEntry::Actions(a) => match a {
                    SequenceActionEntry::Single(action) => duration += action.wait,
                    SequenceActionEntry::Multi(actions) => {
                        duration += actions.iter().map(|a| a.wait).max().unwrap_or_default();
                    }
                },
            }
        }
        duration
    }
    #[inline]
    pub fn set_on_abort(&mut self, action: SequenceAction) {
        self.on_abort = Some(SequenceActionEntry::Single(action));
    }
    #[inline]
    pub fn set_on_abort_multi(&mut self, actions: Vec<SequenceAction>) {
        self.on_abort = Some(SequenceActionEntry::Multi(actions));
    }
    pub fn abort_timeout(&self) -> Duration {
        if let Some(ref on_abort) = self.on_abort {
            match on_abort {
                SequenceActionEntry::Single(a) => a.wait,
                SequenceActionEntry::Multi(actions) => {
                    actions.iter().map(|a| a.wait).max().unwrap_or_default()
                }
            }
        } else {
            Duration::from_secs(0)
        }
    }
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum SequenceEntry {
    Delay(f64),
    Actions(SequenceActionEntry),
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum SequenceActionEntry {
    Single(SequenceAction),
    Multi(Vec<SequenceAction>),
}

#[derive(Serialize, Deserialize)]
pub struct SequenceAction {
    #[serde(rename = "i")]
    pub oid: OID,
    pub params: Option<eva_common::actions::Params>,
    #[serde(
        deserialize_with = "eva_common::tools::de_float_as_duration",
        serialize_with = "eva_common::tools::serialize_duration_as_f64"
    )]
    pub wait: Duration,
}

impl SequenceAction {
    pub fn new_unit(oid: OID, status: ItemStatus, value: Option<Value>, wait: Duration) -> Self {
        Self {
            oid,
            params: Some(eva_common::actions::Params::Unit(
                eva_common::actions::UnitParams {
                    status,
                    value: value.into(),
                },
            )),
            wait,
        }
    }
}
