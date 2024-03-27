use bevy::prelude::*;
use bevy::window::PrimaryWindow;
use bevy_prototype_lyon::prelude::*;

use maveric::prelude::*;
use steks_base::prelude::*;

use strum::IntoEnumIterator;

fn main() {
    let mut app = App::new();

    app.add_plugins(DefaultPlugins)
        .init_resource::<LevelState>()
        .init_resource::<UiState>()
        .add_systems(Startup, setup)
        .add_systems(Update, button_system);

    app.add_systems(Update, mousebutton_listener);
    app.add_systems(Update, handle_drag_start.after(mousebutton_listener));

    app.add_event::<DragStartEvent>();
    app.add_event::<DragMoveEvent>();
    app.add_event::<DragEndingEvent>();

    app.insert_resource(Msaa::Sample4).add_plugins(ShapePlugin);

    // app.register_transition::<StyleLeftLens>();
    // app.register_transition::<TransformScaleLens>();
    // app.register_transition::<BackgroundColorLens>();

    app.register_maveric::<UIRoot>();
    app.run();
}

fn setup(mut commands: Commands) {
    // ui camera
    commands.spawn(Camera2dBundle::default());
}

#[derive(Debug, Resource, MavericContext)]
pub struct UiState {
    pub spawn_shape: ShapeIndex,
    pub spawn_state: ShapeState,
    pub wand_type: Option<WandType>,
}

#[derive(Debug, Resource, Default, MavericContext)]
pub struct LevelState {
    pub placed_shapes: Vec<EncodableShape>,
    //pub dragged_shape: Option<ShapeIndex>,
}

impl Default for UiState {
    fn default() -> Self {
        Self {
            spawn_shape: ShapeIndex(0),
            spawn_state: ShapeState::Normal,
            wand_type: None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum WandType {
    Delete,
    SetState(ShapeState),
    PlaceShape(ShapeIndex),
}

pub fn handle_drag_start(
    mut er_drag_start: EventReader<DragStartEvent>,
    // rapier_context: Res<RapierContext>,

    // mut touch_rotate: ResMut<TouchRotateResource>,
    // mut picked_up_events: EventWriter<ShapePickedUpEvent>,
    mut ui_state: ResMut<UiState>,
    mut level_state: ResMut<LevelState>,
    node_query: Query<(&Node, &GlobalTransform, &ViewVisibility), With<Button>>,
    windows: Query<&Window, With<PrimaryWindow>>,
    ui_scale: Res<UiScale>,
) {
    'events: for event in er_drag_start.read() {
        if let Ok(window) = windows.get_single() {
            //check this isn't a ui event
            let event_ui_position = Vec2 {
                x: event.position.x * ui_scale.0 as f32 + (window.width() * 0.5),
                y: (window.height() * 0.5) - (event.position.y * ui_scale.0 as f32),
            };

            for (node, global_transform, _) in node_query.iter().filter(|x| x.2.get()) {
                let physical_rect = node.physical_rect(global_transform, 1.0, ui_scale.0);

                if physical_rect.contains(event_ui_position) {
                    continue 'events;
                }
            }
        }

        if let Some((index, shape)) = level_state
            .placed_shapes
            .iter()
            .enumerate()
            .filter(|(_, shape)| shape.contains_point(event.position))
            .next()
        {
            if let Some(wand_type) = ui_state.wand_type {
                match wand_type {
                    WandType::Delete => {
                        level_state.placed_shapes.remove(index);
                    }
                    WandType::SetState(state) => {
                        level_state
                            .placed_shapes
                            .get_mut(index)
                            .map(|x| x.state = state);
                    }
                    WandType::PlaceShape(_) => {}
                }
            } else {
                ui_state.wand_type = Some(WandType::PlaceShape(shape.shape));
                //level_state.dragged_shape = Some(shape.shape);
                level_state.placed_shapes.remove(index);
            }

            continue 'events;
        }

        if let Some(WandType::PlaceShape(shape)) = ui_state.wand_type {
            level_state.placed_shapes.push(EncodableShape {
                shape,
                location: Location {
                    position: event.position,
                    angle: 0.0,
                },
                state: ShapeState::Normal,
                modifiers: ShapeModifiers::Normal,
            })
        }
    }
}

fn button_system(
    interaction_query: Query<(&Interaction, &ButtonMarker), (Changed<Interaction>, With<Button>)>,
    mut ui_state: ResMut<UiState>,
) {
    for (interaction, marker) in interaction_query.iter() {
        if *interaction != Interaction::Pressed {
            continue;
        }

        match marker {
            ButtonMarker::PreviousShape => {
                ui_state.spawn_shape.0 = ui_state.spawn_shape.0.checked_sub(1).unwrap_or(20)
            }
            ButtonMarker::NextShape => {
                ui_state.spawn_shape.0 = ui_state.spawn_shape.0.wrapping_add(1) % 21
            }
            ButtonMarker::SpawnShape(shape) => {
                ui_state.wand_type = Some(WandType::PlaceShape(*shape))
            }
            ButtonMarker::SetState(new_state) => {
                ui_state.wand_type = Some(WandType::SetState(*new_state))
            }
            ButtonMarker::Delete => ui_state.wand_type = Some(WandType::Delete),
        }
    }
}

#[derive(MavericRoot)]
struct UIRoot;

impl MavericRootChildren for UIRoot {
    type Context = (LevelState, UiState);

    fn set_children(
        context: &<Self::Context as NodeContext>::Wrapper<'_>,
        commands: &mut impl ChildCommands,
    ) {
        commands.add_child(0, UiPreview, &context.0);
        commands.add_child(1, UiMenu, &context.1);
    }
}

#[derive(Debug, PartialEq)]
struct UiPreview;

impl MavericNode for UiPreview {
    type Context = LevelState;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_context()
            .ignore_node()
            .insert(SpatialBundle::default())
           ;
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands
            .ignore_node()
            .ordered_children_with_context(|context, commands| {
                for (index, shape) in context.placed_shapes.iter().enumerate() {
                    commands.add_child(
                        index as u32,
                        PlacedShape {
                            shape: *shape,
                            index,
                        },
                        &(),
                    )
                }
            });
    }
}

#[derive(Debug, PartialEq)]
struct UiMenu;

impl MavericNode for UiMenu {
    type Context = UiState;

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.ignore_node().ignore_context().insert(NodeBundle {
            style: Style {
                position_type: PositionType::Absolute,

                right: Val::Px(100.0), // Val::Px(MENU_OFFSET),

                display: Display::Flex,
                flex_direction: FlexDirection::Column,

                ..Default::default()
            },
            z_index: ZIndex::Global(10),
            ..Default::default()
        });
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands
            .ignore_node()
            .ordered_children_with_context(|context, commands| {
                commands.add_child(
                    "spawn",
                    button_node(ButtonMarker::SpawnShape(context.spawn_shape)),
                    &(),
                );

                commands.add_child("next", button_node(ButtonMarker::NextShape), &());
                commands.add_child("prev", button_node(ButtonMarker::NextShape), &());

                for x in ShapeState::iter() {
                    let key: &'static str = x.into();
                    commands.add_child(key, button_node(ButtonMarker::SetState(x)), &());
                }

                commands.add_child("delete", button_node(ButtonMarker::Delete), &());
            });
    }
}

#[derive(Debug, Component, PartialEq, Clone, Copy)]
pub enum ButtonMarker {
    PreviousShape,
    NextShape,
    SpawnShape(ShapeIndex),
    SetState(ShapeState),
    Delete,
}

pub fn button_node(marker: ButtonMarker) -> impl MavericNode<Context = ()> {
    let (background_color, color, border_color) =
        (TEXT_BUTTON_BACKGROUND, BUTTON_TEXT_COLOR, BUTTON_BORDER);

    let style = TextButtonStyle::AD;

    let text = match marker {
        ButtonMarker::PreviousShape => "previous shape",
        ButtonMarker::NextShape => "next shape",
        ButtonMarker::SpawnShape(shape_index) => shape_index.game_shape().name,
        ButtonMarker::SetState(state) => match state {
            ShapeState::Normal => "normal",
            ShapeState::Locked => "locked",
            ShapeState::Fixed => "fixed",
            ShapeState::Void => "void",
        },
        ButtonMarker::Delete => "delete",
    };

    ButtonNode {
        style,
        visibility: Visibility::Visible,
        background_color,
        border_color,
        marker,
        children: (TextNode {
            text: text,
            font_size: BUTTON_FONT_SIZE,
            color,
            font: MENU_TEXT_FONT_PATH,
            justify_text: JustifyText::Center,
            linebreak_behavior: bevy::text::BreakLineOn::NoWrap,
        },),
    }
}

#[derive(Debug, PartialEq)]
struct PlacedShape {
    pub shape: EncodableShape,
    pub index: usize,
}

#[derive(Debug, PartialEq, Component)]
struct ShapeMarker(pub usize);

impl MavericNode for PlacedShape {
    type Context = ();

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands.insert_with_node(|node| {
            let scd = ShapeCreationData::from_encodable(node.shape, ShapeStage(0));
            //let shape = node.0.shape.game_shape();
            let fill = scd.fill(false);
            let stroke = scd.stroke(false);
            let mut bundle = scd.shape.body.get_shape_bundle(SHAPE_SIZE);
            let Location { position, angle } = scd.location.unwrap_or_default();

            bundle.spatial.transform = Transform {
                translation: (position.extend(1.0)),
                rotation: Quat::from_rotation_z(angle),
                scale: Vec3::ONE,
            };

            (bundle, stroke, fill, ShapeMarker(node.index))
        });
    }

    fn set_children<R: MavericRoot>(_commands: SetChildrenCommands<Self, Self::Context, R>) {}
}
