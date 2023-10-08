use crate::{prelude::*};
use steks_base::prelude::*;

use steks_base::ui::*;
use strum::Display;



pub struct ButtonPlugin;

impl Plugin for ButtonPlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(First, icon_button_system);
        app.add_systems(First, text_button_system);
    }
}




pub fn icon_button_bundle(disabled: bool) -> ButtonBundle {
    ButtonBundle {
        style: Style {
            width: Val::Px(ICON_BUTTON_WIDTH),
            height: Val::Px(ICON_BUTTON_HEIGHT),
            margin: UiRect::all(Val::Auto),
            justify_content: JustifyContent::Center,
            align_items: AlignItems::Center,
            flex_grow: 0.0,
            flex_shrink: 0.0,

            ..Default::default()
        },
        background_color: ButtonType::Icon.background_color(&Interaction::None, disabled),
        ..default()
    }
}

fn icon_button_system(
    mut interaction_query: Query<
        (&Interaction, &mut BackgroundColor, &IconButtonComponent),
        (Changed<Interaction>, With<Button>),
    >,
    mut change_level_events: EventWriter<ChangeLevelEvent>,

    mut global_ui_state: ResMut<GlobalUiState>,
    mut settings: ResMut<GameSettings>,
    dragged: Query<(), With<BeingDragged>>,
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
                NextLevel => change_level_events.send(ChangeLevelEvent::Next),

                MinimizeSplash => {
                    *global_ui_state = GlobalUiState::MenuClosed(GameUIState::Minimized);
                }

                GooglePlay | Apple | Steam | None => {}

                EnableSnow => settings.snow_enabled = true,

                _ => {}
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

    mut global_ui_state: ResMut<GlobalUiState>,
    mut settings: ResMut<GameSettings>,

    //current_level: Res<CurrentLevel>,
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

                TextButton::Begging => change_level_events.send(ChangeLevelEvent::Begging),

                TextButton::SetArrows(arrows) => settings.show_arrows = arrows,
                TextButton::SetTouchOutlines(outlines) => settings.show_touch_outlines = outlines,
                TextButton::SetRotationSensitivity(rs) => settings.rotation_sensitivity = rs,
                TextButton::SetHighContrast(high_contrast) => {
                    settings.high_contrast = high_contrast
                }

                TextButton::SetFireworks(fireworks) => settings.fireworks_enabled = fireworks,
                TextButton::SetSnow(snow) => settings.snow_enabled = snow,

                TextButton::GetTheGame => {
                    #[cfg(target_arch = "wasm32")]
                    {
                        info!("Exit App");
                        crate::wasm::google_ads_exit_app();
                        // crate::wasm::facebook_ads_exit_app();
                    }
                }

                _ => {}
            }

            if button.button_action.closes_menu() {
                global_ui_state.minimize();
            }
        }
    }
}
