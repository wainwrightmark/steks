use bevy::{log, tasks::IoTaskPool};
use capacitor_bindings::{
    app::AppInfo,
    device::{Device, DeviceId, DeviceInfo, OperatingSystem, Platform},
};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use strum::EnumDiscriminants;

use crate::level::LevelLogData;

#[must_use]
#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, EnumDiscriminants)]
#[serde(tag = "type")]
pub enum LoggableEvent {
    NewUser {
        ref_param: Option<String>,
        referrer: Option<String>,
        gclid: Option<String>,
        language: Option<String>,
        device: Option<LogDeviceInfo>,
        app: Option<LogAppInfo>,
    },
    ApplicationStart {
        ref_param: Option<String>,
        referrer: Option<String>,
        gclid: Option<String>,
    },
    ChangeLevel {
        level: LevelLogData,
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

    NotificationClick
}

#[derive(PartialEq, Eq, Clone, serde:: Serialize, serde::Deserialize, Debug)]
#[serde(transparent)]
pub struct DeviceIdentifier(pub String);

impl From<DeviceId> for DeviceIdentifier {
    fn from(value: DeviceId) -> Self {
        Self(value.identifier)
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

impl EventLog {
    pub fn new_resent(device_id: DeviceIdentifier, event: LoggableEvent) -> Self {
        let severity = event.get_severity();
        Self {
            device_id,
            resent: true,
            event,
            severity,
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogAppInfo {
    build: String,
    version: String,
}

impl From<AppInfo> for LogAppInfo {
    fn from(value: AppInfo) -> Self {
        Self {
            build: value.build,
            version: value.version,
        }
    }
}

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

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogDeviceInfo {
    pub name: Option<String>,
    pub model: String,
    pub platform: Platform,
    pub os: OperatingSystem,
    pub os_version: String,
    pub manufacturer: String,
    pub is_virtual: bool,
    pub web_view_version: Option<String>,
}

impl From<DeviceInfo> for LogDeviceInfo {
    fn from(d: DeviceInfo) -> Self {
        Self {
            name: d.name,
            model: d.model,
            platform: d.platform,
            os: d.operating_system,
            os_version: d.os_version,
            manufacturer: d.manufacturer,
            is_virtual: d.is_virtual,
            web_view_version: d.web_view_version,
        }
    }
}

pub async fn do_or_report_error_async<
    Fut: std::future::Future<Output = Result<(), capacitor_bindings::helpers::Error>>,
    F: Fn() -> Fut + 'static,
>(
    f: F,
) {
    let r = f().await;

    match r {
        Ok(_) => {}
        Err(err) => {
            log::error!("{err:?}");
            LoggableEvent::try_log_error_message_async2(err.to_string()).await;
        }
    }
}

pub fn try_log_error_message(message: String) {
    IoTaskPool::get().spawn(
    async move { LoggableEvent::try_log_error_message_async2(message).await }).detach();
}

impl LoggableEvent {
    pub async fn try_log_error_message_async(message: String, device_id: DeviceId) {
        log::error!("{}", message);
        if !Self::should_ignore_error(&message) {
            let event = LoggableEvent::Error { message };
            Self::try_log_async(event, device_id).await
        }
    }

    pub fn should_ignore_error(error: &str) -> bool {
        const ERRORS_TO_IGNORE: &[&str] = &[
            "Js Exception: Notifications not supported in this browser.",
            "Js Exception: Browser does not support the vibrate API",
            "Js Exception: Abort due to cancellation of share.",
            "Js Exception: Share canceled",
            "Js Exception: Share API not available in this browser",
        ];
        if ERRORS_TO_IGNORE.contains(&error) {
            return true;
        }

        false
    }

    pub async fn try_log_error_async(err: impl Into<anyhow::Error>, device_id: DeviceId) {
        Self::try_log_error_message_async(err.into().to_string(), device_id).await
    }

    pub async fn try_log_error_message_async2(message: String) {
        Self::try_get_device_id_and_log_async(Self::Error{message: message.into()}).await
    }

    pub async fn try_log_async1(self, device_id: DeviceId) {
        Self::try_log_async(self, device_id).await
    }

    /// Either logs the message or sends it to be retried later
    pub async fn try_log_async(data: impl Into<Self>, device_id: DeviceId) {
        //let user = Dispatch::<UserState>::new().get();
        let event = data.into();
        let severity = event.get_severity();

        let message = EventLog {
            event,
            device_id: device_id.into(),
            resent: false,
            severity,
        };

        log::info!("logged {message:?}");
        message.send_log_async().await;
    }

    async fn try_get_device_id_and_log_async(data: impl Into<Self>) {
        let device_id: DeviceId;

        #[cfg(target_arch = "wasm32")]
        {
            match Device::get_id().await {
                Ok(id) => device_id = id,
                Err(err) => {
                    log::error!("{err:?}");
                    return;
                }
            }
        }
        #[cfg(not(target_arch = "wasm32"))]
        {
            device_id = DeviceId {
                identifier: "unknown".to_string(),
            };
        }

        Self::try_log_async(data, device_id).await
    }

    pub fn try_log1(self) {
        Self::try_log(self)
    }

    fn try_log(data: impl Into<Self> + 'static) {
        IoTaskPool::get()
            .spawn(async move { Self::try_get_device_id_and_log_async(data).await })
            .detach();
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
