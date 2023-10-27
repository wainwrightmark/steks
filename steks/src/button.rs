use std::sync::Arc;

use crate::prelude::*;

pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, text_button_system)
            .add_systems(First, icon_button_system);
    }
}

fn icon_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &IconButtonComponent),
        (Changed<Interaction>, With<Button>),
    >,
    mut change_level_events: EventWriter<ChangeLevelEvent>,
    mut share_events: EventWriter<ShareEvent>,
    mut global_ui_state: ResMut<GlobalUiState>,
    mut settings: ResMut<GameSettings>,
    current_level: Res<CurrentLevel>,
    personal_bests: Res<PersonalBests>,
    dragged: Query<(), With<BeingDragged>>,
    mut news: ResMut<NewsResource>,
    leaderboard_data_event_writer: AsyncEventWriter<LeaderboardDataEvent>,
) {
    if !dragged.is_empty() {
        return;
    }

    for (interaction, mut bg_color, button) in interaction_query.iter_mut() {
        if button.disabled || button.button_action == IconButton::None {
            continue;
        }

        use IconButton::*;
        *bg_color = button
            .button_type
            .background_color(interaction, button.disabled);

        if interaction == &Interaction::Pressed {
            match button.button_action {
                OpenMenu => global_ui_state.open_menu(),
                Share => share_events.send(ShareEvent::CurrentShapes),
                SharePB => share_events.send(ShareEvent::PersonalBest),
                NextLevel => change_level_events.send(ChangeLevelEvent::Next),
                OpenNews => {
                    news.is_read = true;
                    *global_ui_state = GlobalUiState::News;
                }
                MinimizeSplash => {
                    *global_ui_state = GlobalUiState::MenuClosed(GameUIState::Minimized);
                }
                RestoreSplash => {
                    *global_ui_state = GlobalUiState::MenuClosed(GameUIState::Splash);
                }
                NextLevelsPage => global_ui_state.as_mut().next_levels_page(),
                PlayPB => {
                    match global_ui_state.as_ref() {
                        GlobalUiState::MenuOpen(MenuPage::PBs { level }) => {
                            if let Some(pb) = personal_bests.get_from_level_index(*level as usize) {
                                if let Some(campaign_level) = CAMPAIGN_LEVELS.get(*level as usize) {
                                    let stage = campaign_level.stages.len();

                                    let saved_data = Some(Arc::new(pb.image_blob.clone()));
                                    let event = ChangeLevelEvent::ChooseCampaignLevel {
                                        index: *level,
                                        stage,
                                        saved_data,
                                    };

                                    change_level_events.send(event);
                                }
                            }
                        }
                        _ => {
                            //do nothing
                        }
                    }
                }

                PreviousLevelsPage => global_ui_state.as_mut().previous_levels_page(),
                RefreshWR => {
                    if let LevelCompletion::Complete { score_info } = current_level.completion {
                        crate::leaderboard::refresh_wr_data(
                            score_info.hash,
                            leaderboard_data_event_writer.clone(),
                        )
                    }
                }

                FollowNewsLink => match news.latest.as_ref() {
                    Some(_news_item) => {
                        #[cfg(target_arch = "wasm32")]
                        {
                            let link = match Platform::CURRENT {
                                Platform::IOs => _news_item.ios_link.as_str(),
                                Platform::Android => _news_item.android_link.as_str(),
                                Platform::Other | Platform::Web => _news_item.default_link.as_str(),
                            };
                            crate::logging::LoggableEvent::FollowNewsLink.try_log1();
                            crate::wasm::open_link(link);
                        }
                    }
                    Option::None => {
                        warn!("There is no news");
                    }
                },

                GooglePlay => {
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[cfg(feature = "web")]
                        {
                            //spellchecker:disable-next-line
                            crate::wasm::gtag_convert("AW-11332063513/iagJCMOoxu8YEJmixpsq");
                        }

                        let level = current_level.level.get_log_name();
                        crate::logging::LoggableEvent::GoAppStore {
                            store: "Google".to_string(),
                            level,
                            max_demo_level: *MAX_DEMO_LEVEL,
                        }
                        .try_log1();
                        crate::wasm::open_link(
                            "https://play.google.com/store/apps/details?id=com.steksgame.app",
                        );
                    }
                }
                Apple => {
                    #[cfg(target_arch = "wasm32")]
                    {
                        #[cfg(feature = "web")]
                        {
                            //spellchecker:disable-next-line
                            crate::wasm::gtag_convert("AW-11332063513/iagJCMOoxu8YEJmixpsq");
                        }

                        let level = current_level.level.get_log_name();
                        crate::logging::LoggableEvent::GoAppStore {
                            store: "Apple".to_string(),
                            level,
                            max_demo_level: *MAX_DEMO_LEVEL,
                        }
                        .try_log1();
                        crate::wasm::open_link("https://apps.apple.com/us/app/steks/id6461480511");
                    }
                }

                Steam | None => {}

                ViewPB => {
                    *global_ui_state =
                        GlobalUiState::MenuClosed(GameUIState::Preview(PreviewImage::PB));
                }
                ViewRecord => {
                    *global_ui_state =
                        GlobalUiState::MenuClosed(GameUIState::Preview(PreviewImage::WR));
                }

                ShowLeaderboard => {
                    crate::leaderboard::try_show_leaderboard(&current_level);
                }
                EnableSnow => settings.snow_enabled = true,
            }
        }
    }
}

fn text_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &TextButtonComponent),
        (Changed<Interaction>, With<Button>),
    >,
    mut change_level_events: EventWriter<ChangeLevelEvent>,
    mut share_events: EventWriter<ShareEvent>,
    mut import_events: EventWriter<ImportEvent>,

    mut global_ui_state: ResMut<GlobalUiState>,
    mut settings: ResMut<GameSettings>,
    mut news: ResMut<NewsResource>,

    current_level: Res<CurrentLevel>,
    achievements: Res<Achievements>,
    completion: Res<CampaignCompletion>,

    dragged: Query<(), With<BeingDragged>>,
) {
    if !dragged.is_empty() {
        return;
    }

    for (interaction, mut bg_color, button) in interaction_query.iter_mut() {
        if button.disabled {
            continue;
        }

        //info!("{interaction:?} {button:?} {menu_state:?}");
        *bg_color = button
            .button_type
            .background_color(interaction, button.disabled);

        if interaction == &Interaction::Pressed {
            match button.button_action {
                TextButton::Resume => {
                    *global_ui_state = GlobalUiState::MenuClosed(GameUIState::Minimized)
                }
                TextButton::News => {
                    news.is_read = true;
                    *global_ui_state = GlobalUiState::News;
                }
                TextButton::GoFullscreen => {
                    #[cfg(target_arch = "wasm32")]
                    {
                        crate::wasm::request_fullscreen();
                    }
                }
                TextButton::ClipboardImport => import_events.send(ImportEvent),
                TextButton::Tutorial => change_level_events
                    .send(ChangeLevelEvent::ChooseTutorialLevel { index: 0, stage: 0 }),
                TextButton::Infinite => change_level_events.send(ChangeLevelEvent::StartInfinite),
                TextButton::Begging => change_level_events.send(ChangeLevelEvent::Begging),
                TextButton::DailyChallenge => {
                    change_level_events.send(ChangeLevelEvent::StartChallenge)
                }

                TextButton::Share => share_events.send(ShareEvent::CurrentShapes),
                TextButton::GotoLevel { level } => {
                    change_level_events.send(ChangeLevelEvent::ChooseCampaignLevel {
                        index: level,
                        stage: 0,
                        saved_data: None,
                    })
                }
                TextButton::ChooseLevel => global_ui_state
                    .as_mut()
                    .toggle_levels(current_level.as_ref()),

                TextButton::ViewPBs => global_ui_state
                    .as_mut()
                    .toggle_view_pbs(current_level.as_ref(), &completion),
                TextButton::MinimizeApp => {
                    spawn_and_run(minimize_app_async());
                }
                TextButton::Credits => change_level_events.send(ChangeLevelEvent::Credits),
                TextButton::OpenSettings => global_ui_state.as_mut().open_settings(),
                TextButton::OpenAccessibility => global_ui_state.as_mut().open_accessibility(),
                TextButton::BackToMenu => global_ui_state.as_mut().open_menu(),
                TextButton::SetArrows(arrows) => settings.show_arrows = arrows,
                TextButton::SetTouchOutlines(outlines) => settings.show_touch_outlines = outlines,
                TextButton::SetRotationSensitivity(rs) => settings.rotation_sensitivity = rs,
                TextButton::SetHighContrast(high_contrast) => {
                    settings.high_contrast = high_contrast
                }

                // TextButton::SyncAchievements => {},
                TextButton::ShowAchievements => {
                    achievements.resync();
                    show_achievements();
                }
                TextButton::InfiniteLeaderboard => {
                    try_show_leaderboard_only(INFINITE_LEADERBOARD.to_string());
                }

                TextButton::SetFireworks(fireworks) => settings.fireworks_enabled = fireworks,
                TextButton::SetSnow(snow) => settings.snow_enabled = snow,

                TextButton::GetTheGame => {}
            }

            if button.button_action.closes_menu() {
                global_ui_state.minimize();
            }
        }
    }
}

async fn minimize_app_async() {
    #[cfg(feature = "android")]
    {
        do_or_report_error(capacitor_bindings::app::App::minimize_app());
    }
}
