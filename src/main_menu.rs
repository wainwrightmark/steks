// use bevy::prelude::*;

// pub enum UiMessage {}

// use crate::menu_action::MenuEvent;
// use crate::{camera::camera_setup, menu_action::SteksMenuState};

// use bevy_iced::{
//     iced::widget::{slider, text, text_input, Button, Column, Row},
//     IcedContext, IcedPlugin, IcedSettings,
// };

// // #[derive(Component, Default, PartialEq, Clone)]
// // struct MyWidget;

// // impl Widget for MyWidget {}

// // // #[derive(Component, Default, PartialEq, Clone)]
// // // struct MyWidgetState {
// // //     pub show_window: bool,
// // // }

// // #[derive(Bundle)]
// // struct MyWidgetBundle {
// //     count: MyWidget,
// //     styles: KStyle,
// //     widget_name: WidgetName,
// // }

// // impl Default for MyWidgetBundle {
// //     fn default() -> Self {
// //         Self {
// //             count: MyWidget::default(),
// //             styles: KStyle::default(),
// //             widget_name: MyWidget::default().get_name(),
// //         }
// //     }
// // }

// // fn my_widget_render(
// //     In(entity): In<Entity>,
// //     widget_context: Res<KayakWidgetContext>,
// //     mut commands: Commands,
// //     query: Query<&SteksMenuState>,
// // ) -> bool {
// //     let state_entity = widget_context.use_state(&mut commands, entity, SteksMenuState::default());
// //     if let Ok(state) = query.get(state_entity) {
// //         let parent_id = Some(entity);
// //         rsx! {
// //             <ElementBundle>
// //                 <KButtonBundle
// //                     styles={KStyle {
// //                         left: Units::Stretch(1.0).into(),
// //                         right: Units::Stretch(1.0).into(),
// //                         ..Default::default()
// //                     }}
// //                     button={KButton {
// //                         text: "Menu".into(),
// //                     }}
// //                     on_event={OnEvent::new(
// //                         move |In(_entity): In<Entity>,
// //                         mut event: ResMut<KEvent>,
// //                             mut query: Query<&mut SteksMenuState>| {
// //                             event.prevent_default();
// //                             event.stop_propagation();
// //                             if let EventType::Click(..) = event.event_type {
// //                                 if let Ok(mut state) = query.get_mut(state_entity) {
// //                                     state.open = true;
// //                                 }
// //                             }
// //                         },
// //                     )}
// //                 />
// //                 {if state.open {
// //                     constructor! {
// //                         <WindowBundle
// //                             window={KWindow {
// //                                 title: "Conditional widget rendering!".into(),
// //                                 draggable: true,
// //                                 initial_position: Vec2::new(10.0, 10.0),
// //                                 size: Vec2::new(300.0, 250.0),
// //                                 ..KWindow::default()
// //                             }}
// //                         >
// //                             <KButtonBundle
// //                                 button={KButton { text: "Hide Window".into() }}
// //                                 on_event={OnEvent::new(
// //                                     move |In(_entity): In<Entity>,
// //                                     mut event: ResMut<KEvent>,
// //                                         mut query: Query<&mut SteksMenuState>| {
// //                                         if let EventType::Click(..) = event.event_type {
// //                                             event.prevent_default();
// //                                             event.stop_propagation();
// //                                             if let Ok(mut state) = query.get_mut(state_entity) {
// //                                                 state.open = false;
// //                                             }
// //                                         }
// //                                     },
// //                                 )}
// //                             />
// //                         </WindowBundle>
// //                     }
// //                 }}
// //             </ElementBundle>
// //         };
// //     }

// //     true
// // }

// // fn startup(
// //     mut commands: Commands,
// //     mut font_mapping: ResMut<FontMapping>,
// //     asset_server: Res<AssetServer>,
// //     cameras: Query<Entity, With<Camera2d>>,
// // ) {
// //     let camera_entity = cameras.single();

// //     commands.entity(camera_entity).insert(CameraUIKayak);

// //     font_mapping.set_default(asset_server.load("lato-light.kttf"));

// //     let mut widget_context = KayakRootContext::new(camera_entity);
// //     widget_context.add_plugin(KayakWidgetsContextPlugin);
// //     let parent_id = None;
// //     widget_context.add_widget_data::<MyWidget, SteksMenuState>();
// //     widget_context.add_widget_system(
// //         MyWidget::default().get_name(),
// //         widget_update::<MyWidget, SteksMenuState>,
// //         my_widget_render,
// //     );
// //     rsx! {
// //         <KayakAppBundle>
// //             <MyWidgetBundle />
// //         </KayakAppBundle>
// //     };

// //     commands.spawn((widget_context, EventDispatcher::default()));
// // }

// use bevy::prelude::{App, Plugin};

// pub struct MainMenuPlugin;

// fn ui_system(
//     state: Res<SteksMenuState>,
//     mut ctx: IcedContext<UiMessage>,
//     event_writer: &mut EventWriter<MenuEvent>,
// ) {
//     if !state.open {
//         let row = Row::new()
//             .spacing(10)
//             .align_items(iced_native::Alignment::Center)
//             .push(Button::new(text("Request box")).on_press(UiMessage::BoxRequested))
//             .push(text(format!(
//                 "{} boxes (amplitude: {})",
//                 sprites.iter().len(),
//                 data.scale
//             )));

//         let column = Column::new()
//             .align_items(iced_native::Alignment::Center)
//             .spacing(10)
//             .push(edit)
//             .push(slider(0.0..=100.0, data.scale, UiMessage::Scale))
//             .push(row);

//         ctx.display(column);
//     } else {
//     }
//     // ctx.display(text(format!(
//     //     "Hello Iced! Running for {:.2} seconds.",
//     //     time.elapsed_seconds()
//     // )));
// }

// impl Plugin for MainMenuPlugin {
//     fn build(&self, app: &mut App) {
//         app.add_plugin(IcedPlugin)
//             .add_event::<UiMessage>()
//             .add_system(ui_system);
//     }
// }
