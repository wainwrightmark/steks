use bevy::{log, prelude::*};
//use capacitor_bindings::{app::AppInfo, device::*};
use crate::prelude::*;
use crate::{
    game_level::LevelLogData,
    global_ui::{spawn_and_run, GlobalUiState},
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use steks_base::shape_component::{handle_change_level_events, ChangeLevelEvent, CurrentLevel};
use strum::EnumDiscriminants;

pub struct LogWatchPlugin;

impl Plugin for LogWatchPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(
            First,
            watch_level_changes.before(handle_change_level_events::<GlobalUiState>),
        );
    }
}

#[allow(unused_variables)]
fn watch_level_changes(
    mut events: EventReader<ChangeLevelEvent>,
    current_level: Res<CurrentLevel>,
) {
    for ev in events.into_iter() {
        let event = match ev {
            ChangeLevelEvent::Next => "Next Level".to_string(),
            ChangeLevelEvent::ChooseCampaignLevel { index, .. } => {
                format!("Go to campaign level {index}")
            }
            ChangeLevelEvent::ChooseTutorialLevel { index, .. } => {
                format!("Go to tutorial level {index}")
            }
            ChangeLevelEvent::ResetLevel => "Reset Level".to_string(),
            ChangeLevelEvent::StartInfinite => "Start Infinite".to_string(),
            ChangeLevelEvent::StartChallenge => "Start Challenge".to_string(),
            ChangeLevelEvent::Load(_) => "Load Game".to_string(),
            ChangeLevelEvent::Credits => "Start Credits".to_string(),
            ChangeLevelEvent::Begging => "Go to begging".to_string(),
            ChangeLevelEvent::Custom { .. } => "Go to custom level".to_string(),
        };
        let loggable_event = LoggableEvent::ChangeLevel {
            level_from: current_level.level.clone().into(),
            event,
        };

        loggable_event.try_log1();
    }
}

#[must_use]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, EnumDiscriminants, Event)]
#[serde(tag = "type")]
pub enum LoggableEvent {
    NewUser {
        ref_param: Option<String>,
        referrer: Option<String>,
        gclid: Option<String>,
        language: Option<String>,
        device: Option<DeviceInformation>,
        app: Option<LogAppInfo>,
        platform: Platform,
    },
    ApplicationStart {
        ref_param: Option<String>,
        referrer: Option<String>,
        gclid: Option<String>,
    },
    ChangeLevel {
        level_from: LevelLogData,
        event: String,
    },
    ClickShare,
    ShareOn {
        platform: String,
    },
    Warn {
        message: String,
    },
    Error {
        message: String,
    },

    Internal {
        message: String,
    },

    GoAppStore {
        store: String,
        level: String,
        max_demo_level: u8,
    },

    PermissionsRequested{
        given: String
    },

    FollowNewsLink,

    NotificationClick,

    ActedInTutorial
}

#[cfg(any(feature = "android", feature = "ios", feature = "web"))]
impl From<capacitor_bindings::error::Error> for LoggableEvent {
    fn from(value: capacitor_bindings::error::Error) -> Self {
        Self::Error {
            message: value.to_string(),
        }
    }
}

// cSpell:ignore xaat

/// This token can only be used to ingest data into our bucket
const API_TOKEN: &str = "xaat-32948a48-2fd1-4ebb-bc4f-263d83c3eac9";

#[derive(Debug, Clone, Serialize)]
pub struct EventLog {
    pub device_id: DeviceIdentifier,
    #[serde(skip_serializing_if = "is_false")]
    pub resent: bool,
    pub event: LoggableEvent,
    #[serde(skip_serializing_if = "is_info_or_lower")]
    pub severity: Severity,
}

fn is_false(b: &bool) -> bool {
    !b
}

fn is_info_or_lower(severity: &Severity) -> bool {
    severity != &Severity::Warn && severity != &Severity::Error
}

#[derive(Debug, Clone, Serialize, PartialEq, Eq)]
pub enum Severity {
    Info,
    Warn,
    Error,
}

// impl EventLog {
//     pub fn new_resent(device_id: DeviceIdentifier, event: LoggableEvent) -> Self {
//         let severity = event.get_severity();
//         Self {
//             device_id,
//             resent: true,
//             event,
//             severity,
//         }
//     }
// }

impl LogAppInfo {
    pub async fn try_get_async() -> Option<LogAppInfo> {
        #[cfg(any(feature = "android", feature = "ios"))]
        {
            capacitor_bindings::app::App::get_info()
                .await
                .ok()
                .map(|x| x.into())
            // crate::capacitor_bindings::get_or_log_error_async()
            //     .await
            //     .map(|x| x.into())
        }
        #[cfg(not(any(feature = "android", feature = "ios")))]
        {
            None
        }
    }
}

#[cfg(any(feature = "android", feature = "ios", feature = "web"))]
pub fn do_or_report_error(
    future: impl std::future::Future<Output = Result<(), capacitor_bindings::error::Error>> + 'static,
) {
    spawn_and_run(do_or_report_error_async(future))
}

#[cfg(any(feature = "android", feature = "ios", feature = "web"))]
pub async fn do_or_report_error_async(
    future: impl std::future::Future<Output = Result<(), capacitor_bindings::error::Error>>,
) {
    let result = future.await;

    match result {
        Ok(_) => {}
        Err(err) => {
            log::error!("{err:?}");
            LoggableEvent::try_log_error_message_async2(err.to_string()).await;
        }
    }
}

pub fn try_log_error_message(message: String) {
    spawn_and_run(LoggableEvent::try_log_error_message_async2(message));
}

impl LoggableEvent {
    pub async fn try_log_error_message_async2(message: String) {
        const MESSAGES_TO_IGNORE: &[&str] = &[
            "Js Exception: Notifications not enabled on this device",
            "Js Exception: Notifications not supported in this browser.",
            "Js Exception: Player is not authenticated",
        ];

        if MESSAGES_TO_IGNORE.contains(&message.as_str()) {
            return;
        }

        Self::try_get_device_id_and_log_async(Self::Error { message }).await
    }

    pub async fn try_log_async1(self, device_id: DeviceIdentifier) {
        Self::try_log_async(self, device_id).await
    }

    /// Either logs the message or sends it to be retried later
    pub async fn try_log_async(data: impl Into<Self>, device_id: DeviceIdentifier) {
        //let user = Dispatch::<UserState>::new().get();
        let event = data.into();
        let severity = event.get_severity();

        let message = EventLog {
            event,
            device_id: device_id.into(),
            resent: false,
            severity,
        };

        log::debug!("logged {message:?}");
        message.send_log_async().await;
    }

    pub async fn try_get_device_id_and_log_async(data: impl Into<Self>) {
        let device_id: DeviceIdentifier;
        #[cfg(any(feature = "android", feature = "ios", feature = "web"))]
        {
            match capacitor_bindings::device::Device::get_id().await {
                Ok(id) => device_id = id.into(),
                Err(err) => {
                    log::error!("{err:?}");
                    return;
                }
            }
        }
        #[cfg(not(any(feature = "android", feature = "ios", feature = "web")))]
        {
            #[cfg(feature="steam")]
            {
                device_id = DeviceIdentifier::steam();
            }
            #[cfg(not(feature="steam"))]
            {
                device_id = DeviceIdentifier::unknown();
            }
        }

        Self::try_log_async(data, device_id).await
    }

    pub fn try_log1(self) {
        Self::try_log(self)
    }

    fn try_log(data: impl Into<Self> + 'static) {
        spawn_and_run(Self::try_get_device_id_and_log_async(data));
    }

    pub fn get_severity(&self) -> Severity {
        match self {
            LoggableEvent::Warn { .. } => Severity::Warn,
            LoggableEvent::Error { .. } => Severity::Error,
            _ => Severity::Info,
        }
    }
}

impl EventLog {
    pub async fn send_log_async(self) {
        Self::log_async(self).await
    }

    async fn try_log<T: Serialize>(data: &T) -> Result<(), reqwest::Error> {
        if !cfg!(debug_assertions) {
            let client = reqwest::Client::new();
            let res = client
                .post("https://api.axiom.co/v1/datasets/steks_usage/ingest")
                // .header("Authorization", format!("Bearer {API_TOKEN}"))
                .bearer_auth(API_TOKEN)
                .header("Content-Type", "application/json")
                .json(&[data])
                .send()
                .await?;

            res.error_for_status().map(|_| ())
        } else {
            Ok(())
        }
    }

    async fn log_async(data: Self) {
        let r = Self::try_log(&data).await;
        if let Err(err) = r {
            log::error!("Failed to log: {}", err);
            //Dispatch::<FailedLogsState>::new().apply(LogFailedMessage(data.event));
        } else {
            let discriminant: LoggableEvent = data.event;
            log::debug!("Log {discriminant:?} sent successfully",);
        }
    }
}
