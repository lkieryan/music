use bitcode::{Decode, Encode};
use serde::{Deserialize, Deserializer, Serialize, Serializer};
#[cfg(feature = "ts-rs")]
use ts_rs::TS;

use crate::errors::MusicError;

#[derive(Serialize, Deserialize, Debug, Default, PartialEq, Eq, Clone, Copy, Encode, Decode)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
#[serde(rename_all = "UPPERCASE")]
pub enum PlayerState {
    Playing,
    Paused,
    #[default]
    Stopped,
    Loading,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum PlayerEvents {
    Play,
    Pause,
    Ended,
    Loading,
    TimeUpdate(f64),

    #[serde(
        deserialize_with = "deserialize_moosync_error",
        serialize_with = "serialize_moosync_error"
    )]
    Error(MusicError),
}

impl Clone for PlayerEvents {
    fn clone(&self) -> Self {
        match self {
            PlayerEvents::Play => PlayerEvents::Play,
            PlayerEvents::Pause => PlayerEvents::Pause,
            PlayerEvents::Ended => PlayerEvents::Ended,
            PlayerEvents::Loading => PlayerEvents::Loading,
            PlayerEvents::TimeUpdate(time) => PlayerEvents::TimeUpdate(*time),
            PlayerEvents::Error(error) => PlayerEvents::Error(error.to_string().clone().into()),
        }
    }
}

fn serialize_moosync_error<S>(error: &MusicError, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    serializer.serialize_str(&error.to_string())
}

fn deserialize_moosync_error<'de, D>(deserializer: D) -> Result<MusicError, D::Error>
where
    D: Deserializer<'de>,
{
    let error_str: String = Deserialize::deserialize(deserializer)?;
    Ok(MusicError::String(error_str))
}

#[derive(Debug, Default, Copy, Clone, Encode, Decode)]
pub enum VolumeMode {
    #[default]
    Normal,
    PersistSeparate,
    PersistClamp,
}

#[derive(Debug, Default, PartialEq, Eq, Copy, Clone, Encode, Decode, Serialize, Deserialize)]
#[cfg_attr(feature = "ts-rs", derive(TS), ts(export, export_to = "bindings.d.ts"))]
pub enum PlayerMode {
    #[default]
    Sequential,
    Single,
    Shuffle,
    ListLoop,
}
