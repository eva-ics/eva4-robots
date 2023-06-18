use eva_common::prelude::*;
use serde::Serialize;
use std::time::Duration;
use uuid::Uuid;

#[derive(Serialize)]
pub struct Sequence<'a> {
    pub u: Uuid,
    pub seq: Vec<SequenceEntry<'a>>,
    pub on_abort: Option<SequenceActionEntry<'a>>,
    #[serde(serialize_with = "eva_common::tools::serialize_duration_as_micros")]
    pub timeout: Duration,
}

impl<'a> Sequence<'a> {
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
            .push(SequenceEntry::Delay(delay.as_micros() as u64));
    }
    #[inline]
    pub fn push_action(&mut self, action: SequenceAction<'a>) {
        self.seq
            .push(SequenceEntry::Actions(SequenceActionEntry::Single(action)));
    }
    #[inline]
    pub fn push_actions_multi(&mut self, actions: Vec<SequenceAction<'a>>) {
        self.seq
            .push(SequenceEntry::Actions(SequenceActionEntry::Multi(actions)));
    }
    pub fn max_expected_duration(&self) -> Duration {
        let mut duration: Duration = Duration::from_secs(0);
        for s in &self.seq {
            match s {
                SequenceEntry::Delay(d) => duration += Duration::from_micros(*d),
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
    pub fn set_on_abort(&mut self, action: SequenceAction<'a>) {
        self.on_abort = Some(SequenceActionEntry::Single(action));
    }
    #[inline]
    pub fn set_on_abort_multi(&mut self, actions: Vec<SequenceAction<'a>>) {
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

#[derive(Serialize)]
#[serde(untagged)]
pub enum SequenceEntry<'a> {
    Delay(u64),
    Actions(SequenceActionEntry<'a>),
}

#[derive(Serialize)]
#[serde(untagged)]
pub enum SequenceActionEntry<'a> {
    Single(SequenceAction<'a>),
    Multi(Vec<SequenceAction<'a>>),
}

#[derive(Serialize)]
pub struct SequenceAction<'a> {
    #[serde(rename = "i")]
    pub oid: &'a OID,
    pub params: Option<eva_common::actions::Params>,
    #[serde(serialize_with = "eva_common::tools::serialize_duration_as_micros")]
    pub wait: Duration,
}

impl<'a> SequenceAction<'a> {
    pub fn new_unit(oid: &'a OID, value: Value, wait: Duration) -> Self {
        Self {
            oid,
            params: Some(eva_common::actions::Params::Unit(
                eva_common::actions::UnitParams { value },
            )),
            wait,
        }
    }
}
