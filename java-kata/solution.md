# Solution

## What was wrong in the legacy design

**Single method with three mixed responsibilities.** `calculateTotalCents` computed discounts, shipping, and taxes in one 115-line method with no internal structure. A change to shipping required reading past all the discount logic to find the right lines.

**Customer type logic was scattered.** Rules for a single customer type were spread across multiple unrelated blocks — a VIP's free shipping threshold appeared 60 lines away from VIP's discount. Adding a new type meant finding and editing every block.

**No-op branches obscured intent.** `discountPercent = discountPercent + 0` for `regular` and `new` (and the final `else`) made the code look intentional while hiding the fact that these types had no discount.

**Coupon logic was implicit.** Some coupons affected discount, some affected shipping, some affected tax — but this was only discoverable by reading the whole method. There was no structure indicating which coupons belonged to which concern.

**Hidden quirks with no documentation.** The code silently: trims and null-handles all string inputs; treats all string comparisons as case-sensitive; applies a 40% discount cap mid-flow; adds a 500-cent shipping surcharge for employees outside IT. None of these were named or commented.

---

## What changed

**`CustomerRules` record** holds per-type pricing rules: base discount %, free shipping threshold, and Black Friday discount %. A static registry (`Map.of(...)`) maps customer type strings to their rules. `CustomerRules.forType()` handles unknown types with a safe default.

**`LegacyCheckoutCalculator` split into focused private methods:**
- `computeDiscount` → calls `baseDiscount`, `couponDiscount`, `blackFridayDiscount`
- `computeShipping` → calls `baseShipping` + applies adjustments
- `computeTax` → calls `baseTax` + applies overrides

**`PARTNER5` coupon** added as one `case` in the coupon switch. **`partner`** customer type added as one line in the registry. The calculator itself required no new conditional branches.

All 60 tests pass. The public API (`calculateTotalCents`, `Order`) is unchanged.

---

## Why the new structure is easier to extend

**Adding a customer type** = one entry in `CustomerRules.REGISTRY`. No changes to the calculator.

**Adding a coupon** = one `case` in `couponDiscount()`. The logic for that coupon is self-contained and does not require understanding the rest of the method.

**Each concern is readable in isolation.** `computeShipping` can be read and changed without reading discount or tax logic. The three concerns no longer interfere with each other.

**Rules are named.** `blackFridayDiscountPercent = 0` for employee is explicit — "this type is exempt" — rather than a conditional check scattered in the middle of unrelated logic.

---

## AI suggestion that was rejected

**Rejected:** Strategy pattern — one interface (`CustomerPricingStrategy`) with a concrete class per customer type (`VipPricingStrategy`, `PremiumPricingStrategy`, etc.), each implementing methods like `baseDiscountPercent()`, `freeShippingThreshold()`, `blackFridayDiscountPercent()`.

**Why rejected:**
The strategy pattern would require creating a new file for every customer type. For simple data that varies per type (a few integers and a boolean), a record with a registry is sufficient and much less code. The strategy pattern adds indirection and file proliferation without a meaningful benefit here — there is no behavior that differs structurally between types, only values. The registry approach achieves the same extensibility goal (add one entry, done) at a fraction of the complexity.

The pattern is not the goal. The goal is that adding a new customer type is safe, local, and requires no understanding of the calculator internals. The registry satisfies that goal with less code and less indirection.
