use crate::prelude::*;
use maveric::prelude::*;
use strum::EnumIs;

#[derive(Debug, Clone, PartialEq, Eq, Default, EnumIs)]
pub enum GameUIState {
    #[default]
    Minimized,
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct GetTheGamePanel {
    pub top: Val,
    pub position_type: PositionType,
}

impl MavericNode for GetTheGamePanel {
    type Context = ();

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .insert_with_node(|node| NodeBundle {
                style: Style {
                    display: Display::Flex,
                    align_items: AlignItems::Center,
                    flex_direction: FlexDirection::Row,
                    // max_size: Size::new(Val::Px(WINDOW_WIDTH), Val::Auto),
                    margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(0.), Val::Px(0.)),
                    justify_content: JustifyContent::Center,
                    width: Val::Auto,
                    height: Val::Auto,
                    top: node.top,
                    position_type: node.position_type,

                    ..Default::default()
                },
                ..Default::default()
            });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.ignore_node().unordered_children(
            |commands| {
                commands.add_child(
                    0,
                    text_button_node(TextButton::GetTheGame, true, false, true),
                    &(),
                );
            },
        );
    }
}

#[derive(Debug, Clone, PartialEq, Default)]
pub struct BeggingPanel;

impl MavericNode for BeggingPanel {
    type Context = ();

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_node().ignore_context().insert(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                display: Display::Flex,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                top: Val::Percent(10.0),
                margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(0.0), Val::Px(0.)),
                justify_content: JustifyContent::Center,
                width: Val::Auto,
                height: Val::Auto,

                ..Default::default()
            },
            ..Default::default()
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands
            .ignore_node()
            .unordered_children_with_context(|context, commands| {
                commands.add_child(
                    0,
                    TextNode {
                        text: "Want More Steks?".to_string(),
                        font_size: LEVEL_TITLE_FONT_SIZE,
                        color: LEVEL_TEXT_COLOR_NORMAL_MODE,
                        font: LEVEL_TITLE_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    },
                    context,
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
                \n\n\n\
                "
                        .to_string(),
                        font_size: LEVEL_TEXT_FONT_SIZE,
                        color: LEVEL_TEXT_COLOR_NORMAL_MODE,
                        font: LEVEL_TEXT_FONT_PATH,
                        alignment: TextAlignment::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    },
                    context,
                );

                commands.add_child(
                    2,
                    GetTheGamePanel {
                        top: Val::Auto,
                        position_type: PositionType::Relative,
                    },
                    context,
                );
            });
    }
}
