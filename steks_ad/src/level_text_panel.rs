use crate::prelude::*;

use maveric::prelude::*;

#[derive(Debug, Clone, PartialEq, Default)]
pub struct LevelTextPanel {
    pub level: GameLevel,
    pub stage: usize,
    pub touch_enabled: bool,
}

impl MavericNode for LevelTextPanel {
    type Context = ();

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_node().ignore_context().insert(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,
                top: Val::Px(80.0),
                display: Display::Flex,
                align_items: AlignItems::Center,
                flex_direction: FlexDirection::Column,
                margin: UiRect::new(Val::Auto, Val::Auto, Val::Px(0.0), Val::Px(0.)),
                justify_content: JustifyContent::Center,
                ..Default::default()
            },
            ..Default::default()
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands.ordered_children_with_node(|args,  commands| {
            let level = &args.level;
            let stage = args.stage;
            let initial_color = LEVEL_TEXT_COLOR_NORMAL_MODE;
            let destination_color = if level.text_fade(stage) {
                initial_color.with_a(0.0)
            } else {
                initial_color
            };

            const FADE_SECS: f32 = 20.;
            if let Some(level_number_text) = level.get_level_number_text(true, stage) {
                commands.add_child(
                    "level_number",
                    TextNode {
                        text: level_number_text,
                        font_size: LEVEL_NUMBER_FONT_SIZE,
                        color: LEVEL_TEXT_COLOR_NORMAL_MODE,
                        font: LEVEL_NUMBER_FONT_PATH,
                        justify_text: JustifyText::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    }
                    .with_transition_in::<TextColorLens<0>>(
                        initial_color,
                        destination_color,
                        Duration::from_secs_f32(FADE_SECS),
                        None
                    ),
                    &(),
                );
            }

            if let Some(title_text) = level.get_title(stage) {
                commands.add_child(
                    "title",
                    TextNode {
                        text: title_text,
                        font_size: LEVEL_TITLE_FONT_SIZE,
                        color: LEVEL_TEXT_COLOR_NORMAL_MODE,
                        font: LEVEL_TITLE_FONT_PATH,
                        justify_text: JustifyText::Center,
                        linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
                    }
                    .with_transition_in::<TextColorLens<0>>(
                        initial_color,
                        destination_color,
                        Duration::from_secs_f32(FADE_SECS),
                        None
                    ),
                    &(),
                );
            }

            if let Some(message) = level.get_level_text(stage, args.touch_enabled) {
                //info!("Message {initial_color:?} {destination_color:?}");
                commands.add_child(
                    stage as u32,
                    panel_text_node(message).with_transition_in::<TextColorLens<0>>(
                        initial_color,
                        destination_color,
                        Duration::from_secs_f32(FADE_SECS),
                        None
                    ),
                    &(),
                )
            }
        })
    }
}
