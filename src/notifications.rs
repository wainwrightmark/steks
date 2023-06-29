use bevy::prelude::*;
use capacitor_bindings::local_notifications::*;

use crate::{async_event_writer::AsyncEventWriter, level::ChangeLevelEvent, logging::*};

const DAILY_CHALLENGE_CLICK_ACTION_ID: &'static str = "DailyChallengeClick";
const DAILY_CHALLENGE_ACTION_TYPE_ID: &'static str = "DailyChallenge";

pub struct NotificationPlugin;

impl Plugin for NotificationPlugin {
    fn build(&self, app: &mut bevy::prelude::App) {
        app.add_startup_system(setup);
    }
}

fn setup(writer: AsyncEventWriter<ChangeLevelEvent>) {
    bevy::tasks::IoTaskPool::get()
        .spawn(async move {
            setup_notifications_async(writer).await;
        })
        .detach();
}

async fn setup_notifications_async(writer: AsyncEventWriter<ChangeLevelEvent>) {
    let schedule_options = LocalNotificationSchema::builder()
        .title("Steks daily challenge")
        .body("Beat your friends in the Steks daily challenge")
        .summary_text("Beat your friends in the Steks daily challenge")
        .id(-1125158781) //Very Random number
        .action_type_id(DAILY_CHALLENGE_ACTION_TYPE_ID)
        .small_icon("notification_icon")
        .large_icon("notification_icon")
        .icon_color("#86AEEA")
        .schedule(ScheduleOn::builder().hour(8).build())
        .auto_cancel(true)
        .build();

    let on_action = move |action: ActionPerformed| {
        if action.action_id == DAILY_CHALLENGE_ACTION_TYPE_ID || action.action_id == "tap" {
            bevy::log::info!("Clicked Action");

            writer
                .send_blocking(ChangeLevelEvent::StartChallenge)
                .expect("Channel closed prematurely");
        }
    };

    #[cfg(any(feature = "ios", feature = "android"))]
    {
        bevy::log::info!("Registering Action Types");

        crate::logging::do_or_report_error_async(|| {
            let action_type_options = RegisterActionTypesOptions {
                types: vec![ActionType {
                    id: DAILY_CHALLENGE_ACTION_TYPE_ID.to_string(),
                    actions: vec![Action {
                        id: DAILY_CHALLENGE_CLICK_ACTION_ID.to_string(),
                        title: "Play Now".to_string(),
                    }],
                }],
            };
            LocalNotifications::register_action_types(action_type_options)
        })
        .await;
    }

    schedule_notification(schedule_options, on_action).await;
}

async fn schedule_notification<F: Fn(ActionPerformed) + 'static>(
    schedule_options: impl Into<ScheduleOptions>,
    on_action: F,
) {
    bevy::log::info!("Scheduling local notification...");
    let schedule_result = LocalNotifications::schedule(schedule_options).await;

    match schedule_result {
        Ok(sr) => {
            bevy::log::info!("Notification Scheduled {:?}", sr.notifications);
        }
        Err(err) => {
            LoggableEvent::try_log_error_message_async2(err.to_string()).await;
        }
    }

    bevy::log::info!("Registering Action Listener");
    let listener_result = LocalNotifications::add_action_performed_listener(on_action).await;
    match listener_result {
        Ok(lr) => {
            lr.leak();
        }
        Err(err) => {
            LoggableEvent::try_log_error_message_async2(err.to_string()).await;
        }
    }
    bevy::log::info!("Action Listener Registered");
}
