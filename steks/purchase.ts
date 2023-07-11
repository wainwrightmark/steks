import { InAppPurchase2, IAPProduct, InAppPurchase2Original } from "cordova-plugin-purchase/www/store-android"

const PRODUCT_PRO_KEY = "steks_unlock1"

export class Purchases {
  is_unlocked = false
  products: IAPProduct[] = []
  store: typeof InAppPurchase2;

  constructor() {
    this.store = new InAppPurchase2Original()
    // Only for debugging!
    this.store.verbosity = this.store.DEBUG

    this.registerProducts()
    this.setupListeners()

    // Get the real product information
    this.store.ready(() => {
      this.products = this.store.products
    })
  }

  registerProducts() {
    this.store.register({
      id: PRODUCT_PRO_KEY,
      type: this.store.NON_CONSUMABLE
    })

    this.store.refresh()
  }

  setupListeners() {
    // General query to all products
    this.store
      .when("product")
      .approved(p => {
        // Handle the product deliverable
        if (p.id === PRODUCT_PRO_KEY) {
          this.is_unlocked = true
        }

        return p.verify()
      })
      .verified(p => p.finish())

    // Specific query for one ID
    this.store.when(PRODUCT_PRO_KEY).owned(p => {
      console.info("Unlock already owned")
      this.is_unlocked = true
    })
  }

  get_is_unlocked() {
    return this.is_unlocked
  }

  can_purchase() {
    return this.products.length > 0
  }

  try_purchase() {
    if (this.products.length == 0) {
      console.error(`Can not purchase - no products`)
    } else {
      let product = this.products[0]

      this.store.order(product).then(
        p => {
          // Purchase in progress!
          console.info("Purchase in progress")
        },
        e => {
          console.error(`Failed to purchase: ${e}`)
        }
      )
    }
  }

  // To comply with AppStore rules
  restore() {
    this.store.refresh()
  }
}
