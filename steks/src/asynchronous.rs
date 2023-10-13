use std::future::Future;

pub fn spawn_and_run(future: impl Future<Output = ()> + 'static) {
    let pool = bevy::tasks::IoTaskPool::get();

    #[cfg(target_arch = "wasm32")]
    pool.spawn(future).detach();
    #[cfg(not(target_arch = "wasm32"))]
    pool.spawn_local(async_compat::Compat::new(future)).detach();
}
