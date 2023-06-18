use eva_common::prelude::*;
use serde::{Deserialize, Serialize};
use std::time::Duration;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
pub struct SequenceOwned {
    #[serde(alias = "uuid", deserialize_with = "crate::deserialize_uuid")]
    pub u: Uuid,
    pub seq: Vec<SequenceEntryOwned>,
    pub on_abort: Option<SequenceActionEntryOwned>,
    #[serde(
        deserialize_with = "eva_common::tools::deserialize_duration_from_micros",
        serialize_with = "eva_common::tools::serialize_duration_as_micros"
    )]
    pub timeout: Duration,
}

impl SequenceOwned {
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
    #[allow(clippy::cast_possible_truncation)]
    pub fn push_delay(&mut self, delay: Duration) {
        self.seq
            .push(SequenceEntryOwned::Delay(delay.as_micros() as u64));
    }
    #[inline]
    pub fn push_action(&mut self, action: SequenceActionOwned) {
        self.seq.push(SequenceEntryOwned::Actions(
            SequenceActionEntryOwned::Single(action),
        ));
    }
    #[inline]
    pub fn push_actions_multi(&mut self, actions: Vec<SequenceActionOwned>) {
        self.seq.push(SequenceEntryOwned::Actions(
            SequenceActionEntryOwned::Multi(actions),
        ));
    }
    pub fn max_expected_duration(&self) -> Duration {
        let mut duration: Duration = Duration::from_secs(0);
        for s in &self.seq {
            match s {
                SequenceEntryOwned::Delay(d) => duration += Duration::from_micros(*d),
                SequenceEntryOwned::Actions(a) => match a {
                    SequenceActionEntryOwned::Single(action) => duration += action.wait,
                    SequenceActionEntryOwned::Multi(actions) => {
                        duration += actions.iter().map(|a| a.wait).max().unwrap_or_default();
                    }
                },
            }
        }
        duration
    }
    #[inline]
    pub fn set_on_abort(&mut self, action: SequenceActionOwned) {
        self.on_abort = Some(SequenceActionEntryOwned::Single(action));
    }
    #[inline]
    pub fn set_on_abort_multi(&mut self, actions: Vec<SequenceActionOwned>) {
        self.on_abort = Some(SequenceActionEntryOwned::Multi(actions));
    }
    pub fn abort_timeout(&self) -> Duration {
        if let Some(ref on_abort) = self.on_abort {
            match on_abort {
                SequenceActionEntryOwned::Single(a) => a.wait,
                SequenceActionEntryOwned::Multi(actions) => {
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
pub enum SequenceEntryOwned {
    Delay(u64),
    Actions(SequenceActionEntryOwned),
}

#[derive(Serialize, Deserialize)]
#[serde(untagged)]
pub enum SequenceActionEntryOwned {
    Single(SequenceActionOwned),
    Multi(Vec<SequenceActionOwned>),
}

#[derive(Serialize, Deserialize)]
pub struct SequenceActionOwned {
    #[serde(rename = "i")]
    pub oid: OID,
    pub params: Option<eva_common::actions::Params>,
    #[serde(
        deserialize_with = "eva_common::tools::deserialize_duration_from_micros",
        serialize_with = "eva_common::tools::serialize_duration_as_micros"
    )]
    pub wait: Duration,
}

impl SequenceActionOwned {
    pub fn new_unit(oid: OID, value: Value, wait: Duration) -> Self {
        Self {
            oid,
            params: Some(eva_common::actions::Params::Unit(
                eva_common::actions::UnitParams { value },
            )),
            wait,
        }
    }
}
