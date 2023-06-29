use async_channel::SendError;
use bevy::{ecs::system::SystemParam, prelude::*};
use std::marker::PhantomData;

pub struct AsyncEventPlugin<T: Event>(PhantomData<T>);

impl<T: Event> Default for AsyncEventPlugin<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

impl<T: Event> Plugin for AsyncEventPlugin<T> {
    fn build(&self, app: &mut App) {
        app.add_event::<T>()
            .add_system(poll_events::<T>)
            .init_resource::<AsyncEventResource<T>>();
    }
}

fn poll_events<T: Event>(channels: Res<AsyncEventResource<T>>, mut writer: EventWriter<T>) {
    while let Ok(ev) = channels.receiver.try_recv() {
        writer.send(ev)
    }
}

#[derive(Debug, Clone)]
pub struct AsyncEventWriter<T: Event>(async_channel::Sender<T>);

impl<T: Event> AsyncEventWriter<T> {
    pub async fn send_async(&self, event: T) -> Result<(), SendError<T>> {
        self.0.send(event).await
    }

    pub fn send_blocking(&self, event: T)-> Result<(), SendError<T>> {
        self.0.send_blocking(event)
    }
}

unsafe impl<T: Event> SystemParam for AsyncEventWriter<T> {
    type State = ();

    type Item<'world, 'state> = Self;

    fn init_state(
        _world: &mut World,
        _system_meta: &mut bevy::ecs::system::SystemMeta,
    ) -> Self::State {
    }

    unsafe fn get_param<'world, 'state>(
        _state: &'state mut Self::State,
        _system_meta: &bevy::ecs::system::SystemMeta,
        world: &'world World,
        _change_tick: u32,
    ) -> Self::Item<'world, 'state> {
        let resource = world.get_resource::<AsyncEventResource<T>>().expect("Event is not registered as an async event");
        Self(resource.sender.clone())
    }
}

#[derive(Resource)]
struct AsyncEventResource<T: Event> {
    sender: async_channel::Sender<T>,
    receiver: async_channel::Receiver<T>,
}

impl<T: Event> Default for AsyncEventResource<T> {
    fn default() -> Self {
        let (sender, receiver) = async_channel::unbounded();
        Self { sender, receiver }
    }
}
