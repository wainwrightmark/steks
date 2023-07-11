use bevy::prelude::*;
#[cfg(all(target_arch = "wasm32", any(feature = "android", feature = "ios")))]
use wasm_bindgen::prelude::*;

#[cfg(all(target_arch = "wasm32", any(feature = "android", feature = "ios")))]
#[wasm_bindgen(module = "/purchase.js")]
extern "C" {

    pub type Purchases;

    #[wasm_bindgen(constructor)]
    fn new() -> Purchases;

    // #[wasm_bindgen(method, js_namespace= ["purchase", "Purchases"])]
    // pub fn get_is_unlocked(this: &Purchases) -> bool;

    // #[wasm_bindgen(method, js_namespace= ["purchase", "Purchases"])]
    // pub fn can_purchase(this: &Purchases) -> bool;

    #[wasm_bindgen(method)]
    pub fn try_purchase(this: &Purchases);

    // #[wasm_bindgen(method, js_namespace= ["purchase", "Purchases"])]
    // fn restore(this: &Purchases);
}

#[derive(Debug, Event)]
pub struct TryPurchaseEvent;

#[cfg(all(target_arch = "wasm32", any(feature = "android", feature = "ios")))]
fn handle_purchases(mut events: EventReader<TryPurchaseEvent>, res: NonSend<Purchases>) {
    if !events.is_empty() {
        events.clear();
        res.try_purchase()
    }
}

#[cfg(all(target_arch = "wasm32", any(feature = "android", feature = "ios")))]
fn init_purchases(world: &mut World) {
    let purchases = Purchases::new();
    world.insert_non_send_resource(purchases);
}

pub struct PurchasesPlugin;

impl Plugin for PurchasesPlugin {
    fn build(&self, app: &mut App) {
        app.add_event::<TryPurchaseEvent>();

        #[cfg(all(target_arch = "wasm32", any(feature = "android", feature = "ios")))]
        {
            // app.add_systems(Startup,init_purchases)
            //     .add_system(handle_purchases);
        }
    }
}
