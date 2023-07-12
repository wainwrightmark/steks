const PRODUCT_PRO_KEY = "steks_unlock1";

export class Purchases {
  is_unlocked = false;
  products = [];
  store;

  constructor() {
    // Only for debugging!
    CdvPurchase.store.verbosity = CdvPurchase.store.DEBUG;

    this.registerProducts();
    this.setupListeners();

    // Get the real product information
    CdvPurchase.store.ready(() => {
      this.products = CdvPurchase.store.products;
    });
  }

  registerProducts() {
    CdvPurchase.store.register({
      id: PRODUCT_PRO_KEY,
      type: CdvPurchase.store.NON_CONSUMABLE,
    });

    CdvPurchase.store.initialize();
  }

  setupListeners() {
    // General query to all products
    CdvPurchase.store
      .when("product")
      .approved((p) => {
        // Handle the product deliverable
        if (p.id === PRODUCT_PRO_KEY) {
          this.is_unlocked = true;
        }

        return p.verify();
      })
      .verified((p) => p.finish());

    // Specific query for one ID

    if (CdvPurchase.store.owned(PRODUCT_PRO_KEY)) {
      console.info("Unlock already owned");
      this.is_unlocked = true;
    }
  }

  get_is_unlocked() {
    return this.is_unlocked;
  }

  can_purchase() {
    return this.products.length > 0;
  }

  try_purchase() {
    if (this.products.length == 0) {
      console.error(`Can not purchase - no products`);
    } else {
      let product = this.products[0];

      CdvPurchase.store.order(product).then(
        (p) => {
          // Purchase in progress!
          console.info("Purchase in progress");
        },
        (e) => {
          console.error(`Failed to purchase: ${e}`);
        }
      );
    }
  }

  // To comply with AppStore rules
  restore() {
    CdvPurchase.store.initialize();
  }
}
