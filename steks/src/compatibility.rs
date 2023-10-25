use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub struct SubmitScoreData {
    pub leaderboard_id: String,
    pub total_score_amount: i32,
}

#[cfg(any(feature = "android", feature = "ios", feature = "web"))]
impl Into<capacitor_bindings::game_connect::SubmitScoreOptions> for SubmitScoreData {
    fn into(self) -> capacitor_bindings::game_connect::SubmitScoreOptions {
        capacitor_bindings::game_connect::SubmitScoreOptions {
            leaderboard_id: self.leaderboard_id,
            total_score_amount: self.total_score_amount,
        }
    }
}

pub const DEVICE_ID: std::sync::OnceLock<DeviceIdentifier> = std::sync::OnceLock::new(); // DeviceIdentifier::EMPTY;

#[derive(Clone, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(transparent)]
pub struct DeviceIdentifier {
    pub identifier: String,
}

impl DeviceIdentifier {
    pub const EMPTY: Self = Self {
        identifier: String::new(),
    };

    pub fn unknown()-> Self{
        Self { identifier: "unknown".to_string() }
    }

    pub fn steam()-> Self{
        Self { identifier: "Steam".to_string() }
    }
}

impl DeviceIdentifier {
    pub fn new(identifier: String) -> Self {
        Self { identifier }
    }
}

#[cfg(any(feature = "android", feature = "ios", feature = "web"))]
impl Into<capacitor_bindings::device::DeviceId> for DeviceIdentifier {
    fn into(self) -> capacitor_bindings::device::DeviceId {
        capacitor_bindings::device::DeviceId {
            identifier: self.identifier,
        }
    }
}

#[cfg(any(feature = "android", feature = "ios", feature = "web"))]
impl From<capacitor_bindings::device::DeviceId> for DeviceIdentifier {
    fn from(value: capacitor_bindings::device::DeviceId) -> Self {
        Self {
            identifier: value.identifier,
        }
    }
}


#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DeviceInformation {
    pub name: Option<String>,
    pub model: String,
    pub platform: Platform,
    pub os: OperatingSystem,
    pub os_version: String,
    pub manufacturer: String,
    pub is_virtual: bool,
    pub web_view_version: Option<String>,
}

#[cfg(any(feature = "android", feature = "ios", feature = "web"))]
impl From<capacitor_bindings::device::DeviceInfo> for DeviceInformation {
    fn from(d: capacitor_bindings::device::DeviceInfo) -> Self {
        Self {
            name: d.name,
            model: d.model,
            platform: d.platform.into(),
            os: d.operating_system.into(),
            os_version: d.os_version,
            manufacturer: d.manufacturer,
            is_virtual: d.is_virtual,
            web_view_version: d.web_view_version,
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone, Copy, Serialize, Deserialize)]
pub enum Platform {
    IOs,
    Android,
    Web,
    Other,
}

impl Platform {
    pub const CURRENT: Platform = {
        if cfg!(feature = "android") {
            Platform::Android
        } else if cfg!(feature = "ios") {
            Platform::IOs
        } else if cfg!(feature = "web") {
            Platform::Web
        } else {
            Platform::Other
        }
    };
}

#[cfg(any(feature = "android", feature = "ios", feature = "web"))]
impl From<capacitor_bindings::device::Platform> for Platform {
    fn from(value: capacitor_bindings::device::Platform) -> Self {
        match value {
            capacitor_bindings::device::Platform::IOs => Self::IOs,
            capacitor_bindings::device::Platform::Android => Self::Android,
            capacitor_bindings::device::Platform::Web => Self::Web,
        }
    }
}

#[derive(Copy, Clone, Default, Debug, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum OperatingSystem {
    IOs,
    Android,
    Windows,
    Mac,
    #[default]
    Unknown,
}

#[cfg(any(feature = "android", feature = "ios", feature = "web"))]
impl From<capacitor_bindings::device::OperatingSystem> for OperatingSystem {
    fn from(value: capacitor_bindings::device::OperatingSystem) -> Self {
        match value {
            capacitor_bindings::device::OperatingSystem::IOs => Self::IOs,
            capacitor_bindings::device::OperatingSystem::Android => Self::Android,
            capacitor_bindings::device::OperatingSystem::Windows => Self::Windows,
            capacitor_bindings::device::OperatingSystem::Mac => Self::Mac,
            capacitor_bindings::device::OperatingSystem::Unknown => Self::Unknown,
        }
    }
}

#[skip_serializing_none]
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct LogAppInfo {
    build: String,
    version: String,
}

#[cfg(any(feature = "android", feature = "ios", feature = "web"))]
impl From<capacitor_bindings::app:: AppInfo> for LogAppInfo {
    fn from(value: capacitor_bindings::app::AppInfo) -> Self {
        Self {
            build: value.build,
            version: value.version,
        }
    }
}
