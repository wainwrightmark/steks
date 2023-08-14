use crate::prelude::*;
use state_hierarchy::{impl_hierarchy_root, prelude::*, transition::speed::ScalarSpeed};
use strum::EnumIs;
pub struct LevelUiPlugin;

impl Plugin for LevelUiPlugin {
    fn build(&self, app: &mut App) {
        app.init_resource::<GameUIState>()
            .register_state_hierarchy::<LevelUiRoot>();
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Resource, EnumIs)]
pub enum GameUIState {
    #[default]
    GameSplash,
    GameMinimized,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LevelUiRoot;

impl HasContext for LevelUiRoot {
    type Context = NC2<MenuState, NC3<GameUIState, CurrentLevel, AssetServer>>;
}

impl ChildrenAspect for LevelUiRoot {
    fn set_children(
        &self,
        _previous: Option<&Self>,
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        if context.0.is_closed() {
            let (top, _) = match context.1 .1.completion {
                LevelCompletion::Complete { score_info: _ } => {
                    if context.1 .0.is_game_minimized() {
                        (Val::Percent(00.), Val::Percent(90.))
                    } else {
                        (Val::Percent(30.), Val::Percent(70.))
                    }
                }

                _ => (Val::Percent(30.), Val::Percent(70.)),
            };

            commands.add_child(
                0,
                MainPanelWrapper.with_transition_to::<StyleTopLens>(top, ScalarSpeed::new(20.0)),
                &context.1,
            );
        }
    }
}

impl_hierarchy_root!(LevelUiRoot);

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MainPanelWrapper;

impl HasContext for MainPanelWrapper {
    type Context = NC3<GameUIState, CurrentLevel, AssetServer>;
}

impl StaticComponentsAspect for MainPanelWrapper {
    type B = NodeBundle;

    fn get_bundle() -> Self::B {
        NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                left: Val::Percent(50.0),
                top: Val::Percent(30.0),
                right: Val::Percent(50.0),
                bottom: Val::Percent(90.0),
                justify_content: JustifyContent::Center,
                flex_direction: FlexDirection::Column,
                ..Default::default()
            },

            z_index: ZIndex::Global(15),
            ..Default::default()
        }
    }
}

impl ChildrenAspect for MainPanelWrapper {
    fn set_children(
        &self,
        _previous: Option<&Self>,
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        commands.add_child(0, MainPanel, context);
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct MainPanel;

impl HasContext for MainPanel {
    type Context = NC3<GameUIState, CurrentLevel, AssetServer>;
}

impl ComponentsAspect for MainPanel {
    fn set_components(
        &self,
        _previous: Option<&Self>,
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ComponentCommands,
        _event: SetComponentsEvent,
    ) {
        let (background, border) = match (context.1.completion, context.0.is_game_splash()) {
            (LevelCompletion::Complete { .. }, true) => (Color::WHITE, Color::BLACK),
            _ => (Color::WHITE.with_a(0.0), Color::BLACK.with_a(0.0)),
        };

        let color_speed = context.1.completion.is_complete().then_some(ScalarSpeed {
            amount_per_second: 1.0,
        });

        let background =
            commands.transition_value::<BackgroundColorLens>(background, background, color_speed);

        let border = commands.transition_value::<BorderColorLens>(border, border, color_speed);

        let visibility = if context.1.level.skip_completion() && context.1.completion.is_complete() {
            Visibility::Hidden
        } else {
            Visibility::Inherited
        };

        let z_index = ZIndex::Global(15);

        let flex_direction: FlexDirection =
            if context.1.completion.is_complete() && context.0.is_game_splash() {
                FlexDirection::Column
            } else {
                FlexDirection::RowReverse
            };

        let bundle = NodeBundle {
            style: Style {
                display: Display::Flex,
                align_items: AlignItems::Center,
                flex_direction,
                margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(0.), Val::Px(0.)),
                justify_content: JustifyContent::Center,
                border: UiRect::all(UI_BORDER_WIDTH),
                ..Default::default()
            },

            background_color: BackgroundColor(background),
            border_color: BorderColor(border),
            visibility,
            z_index,
            ..Default::default()
        };

        commands.insert(bundle);
    }
}

impl ChildrenAspect for MainPanel {
    fn set_children(
        &self,
        _previous: Option<&Self>,
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        if context.1.level.is_begging() {
            commands.add_child(100, BeggingPanel, context);
        } else {
            commands.add_child(0, TextPanel, context);
            commands.add_child(1, ButtonPanel, context);

            let show_store_buttons =
                context.1.completion.is_complete() && context.0.is_game_splash() && IS_DEMO;

            if show_store_buttons {
                commands.add_child(2, StoreButtonPanel, context);
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct TextPanel;

impl HasContext for TextPanel {
    type Context = NC3<GameUIState, CurrentLevel, AssetServer>;
}

impl ChildrenAspect for TextPanel {
    fn set_children(
        &self,
        _previous: Option<&Self>,
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        if context.1.completion.is_incomplete() {
            let initial_color = context.1.text_color();
            let destination_color = if  context.1.text_fade() {
                initial_color.with_a(0.0)
            } else {
                initial_color
            };

            const FADE_SECS: f32 = 20.;
            if let Some(level_number_text) = context.1.get_level_number_text(true) {
                commands.add_child(
                    "level_number",
                    TextNode {
                        text: level_number_text,
                        style: LEVEL_NUMBER_TEXT_STYLE.clone(),
                    }
                    .with_transition_in::<TextColorLens<0>>(
                        initial_color,
                        destination_color,
                        Duration::from_secs_f32(FADE_SECS),
                    ),
                    &context.2,
                );
            }

            if let Some(title_text) = context.1.get_title() {
                commands.add_child(
                    "title",
                    TextNode {
                        text: title_text,
                        style: TITLE_TEXT_STYLE.clone(),
                    }
                    .with_transition_in::<TextColorLens<0>>(
                        initial_color,
                        destination_color,
                        Duration::from_secs_f32(FADE_SECS),
                    ),
                    &context.2,
                );
            }

            if let Some(message) = context.1.get_text(&context.0) {

                //info!("Message {initial_color:?} {destination_color:?}");
                commands.add_child(
                    "message",
                    TextNode {
                        text: message,
                        style: LEVEL_MESSAGE_TEXT_STYLE.clone(),
                    }
                    .with_transition_in::<TextColorLens<0>>(
                        initial_color,
                        destination_color,
                        Duration::from_secs_f32(FADE_SECS),
                    )
                    ,
                    &context.2,
                )
            }
        } else if let Some(message) = context.1.get_text(&context.0) {
            commands.add_child(
                "completion_message",
                TextNode {
                    text: message,
                    style: LEVEL_MESSAGE_TEXT_STYLE.clone(),
                },
                &context.2,
            )
        }
    }
}

impl ComponentsAspect for TextPanel {
    fn set_components(
        &self,
        _previous: Option<&Self>,
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ComponentCommands,
        _event: SetComponentsEvent,
    ) {
        let top_margin = match (context.0.is_game_splash(), context.1.completion) {
            (_, LevelCompletion::Incomplete { .. }) => Val::Px(0.0),
            (true, LevelCompletion::Complete { .. }) => Val::Px(20.0),
            (false, LevelCompletion::Complete { .. }) => Val::Px(0.0),
        };

        commands.insert(NodeBundle {
            style: Style {
                display: Display::Flex,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                margin: UiRect::new(Val::Auto, Val::Auto, top_margin, Val::Px(0.)),
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            ..Default::default()
        });

        // commands.insert(Transition {
        //     step: TransitionStep::<TextColorLens<0>>::new_arc(
        //         Color::NONE,
        //         Some(ScalarSpeed {
        //             amount_per_second: 0.05,
        //         }),
        //         None,
        //     ),
        // })
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct ButtonPanel;

impl HasContext for ButtonPanel {
    type Context = NC3<GameUIState, CurrentLevel, AssetServer>;
}

impl ChildrenAspect for ButtonPanel {
    fn set_children(
        &self,
        _previous: Option<&Self>,
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        if context.1.completion.is_complete() {
            if context.0.is_game_splash() {
                commands.add_child(
                    0,
                    icon_button_node(ButtonAction::MinimizeSplash),
                    &context.2,
                );
            } else {
                commands.add_child(0, icon_button_node(ButtonAction::RestoreSplash), &context.2);
            }

            commands.add_child(1, icon_button_node(ButtonAction::Share), &context.2);
            commands.add_child(2, icon_button_node(ButtonAction::NextLevel), &context.2);
        }
    }
}

impl StaticComponentsAspect for ButtonPanel {
    type B = NodeBundle;

    fn get_bundle() -> Self::B {
        NodeBundle {
            style: Style {
                display: Display::Flex,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                // max_size: Size::new(Val::Px(WINDOW_WIDTH), Val::Auto),
                margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(0.), Val::Px(0.)),
                justify_content: JustifyContent::Center,
                width: Val::Auto,
                height: Val::Auto,

                ..Default::default()
            },
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct StoreButtonPanel;

impl HasContext for StoreButtonPanel {
    type Context = NC3<GameUIState, CurrentLevel, AssetServer>;
}

impl ChildrenAspect for StoreButtonPanel {
    fn set_children(
        &self,
        _previous: Option<&Self>,
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        commands.add_child(
            4,
            image_button_node(ButtonAction::GooglePlay, "images/google-play-badge.png"),
            &context.2,
        );
        commands.add_child(
            5,
            image_button_node(ButtonAction::Apple, "images/apple-store-badge.png"),
            &context.2,
        );
    }
}

impl StaticComponentsAspect for StoreButtonPanel {
    type B = NodeBundle;

    fn get_bundle() -> Self::B {
        NodeBundle {
            style: Style {
                display: Display::Flex,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Row,
                // max_size: Size::new(Val::Px(WINDOW_WIDTH), Val::Auto),
                margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(0.), Val::Px(0.)),
                justify_content: JustifyContent::Center,
                width: Val::Auto,
                height: Val::Auto,

                ..Default::default()
            },
            ..Default::default()
        }
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BeggingPanel;

impl HasContext for BeggingPanel {
    type Context = NC3<GameUIState, CurrentLevel, AssetServer>;
}

impl ChildrenAspect for BeggingPanel {
    fn set_children<'r>(
        &self,
        _previous: Option<&Self>,
        context: &<Self::Context as NodeContext>::Wrapper<'r>,
        commands: &mut impl ChildCommands,
    ) {
        commands.add_child(
            0,
            TextNode {
                text: "Want More Steks?".to_string(),
                style: TITLE_TEXT_STYLE.clone(),
            },
            &context.2,
        );

       

        commands.add_child(
            3,
            TextNode {
                text: "Play the full game\n\n\
                Build ice towers while\n\
                 the snow swirls\n\
                \n\
                Build upside-down in\n\
                inverted gravity\n\
                \n\
                Build crazy towers on\n\
                slanted foundations\n\
                \n\
                And...\n\
                Defeat Dr. Gravity!\n\
                \n\
                Get steks now\n\
                ".to_string(),
                style: BEGGING_MESSAGE_TEXT_STYLE.clone(),
            },
            &context.2,
        );

        commands.add_child(2, StoreButtonPanel, context);
    }
}

impl StaticComponentsAspect for BeggingPanel {
    type B = NodeBundle;

    fn get_bundle() -> Self::B {
        NodeBundle {
            style: Style {
                display: Display::Flex,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                // max_size: Size::new(Val::Px(WINDOW_WIDTH), Val::Auto),
                margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(200.), Val::Px(0.)),
                justify_content: JustifyContent::Center,
                width: Val::Auto,
                height: Val::Auto,

                ..Default::default()
            },
            ..Default::default()
        }
    }
}
