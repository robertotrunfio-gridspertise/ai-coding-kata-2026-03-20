# Solution Notes

## What was wrong in the legacy design

1. **Single monolithic function with mixed responsibilities.** Discount, shipping, and tax logic were all interleaved in one large function with no separation of concerns.

2. **Long if/else chains for customer types.** Adding a new customer type required editing the main function in three separate places: base discount, free-shipping check, and extra-shipping check. There was no single place to look up "what does a VIP customer mean?"

3. **Implicit coupling between conditions.** The `if customerType == "employee"` shipping surcharge appeared far away from the `if customerType == "employee"` discount block. A future reader had to search the whole function to understand a single customer type.

4. **Dead code masking intent.** `discountPercent += 0` for `regular` and `new` customers added noise without meaning, and the final `else { discountPercent += 0 }` block was unreachable given the preceding conditions.

5. **Hardcoded magic strings everywhere.** `"IT"`, `"DE"`, `"vip"`, `"SAVE10"` appeared scattered across all three logic sections with no central definition.

6. **Adding a new coupon required editing the middle of the discount section.** The coupon if/else chain was embedded inside the same function alongside customer and Black Friday logic.

## What changed

The monolithic function was replaced with a **data-driven config table** approach:

- `customerRules` (or `CUSTOMER_RULES` / `customer_config`) — maps each customer type to a small config that encodes: base discount function, free-shipping threshold, extra shipping function, and Black Friday bonus override. Adding a new customer type is one entry in this table.
- `couponRules` (or `COUPON_RULES` / `coupon_discount`) — maps each coupon code to its eligibility + discount logic. Adding a new coupon is one entry.
- `countryShipping` / `countryTax` — flat lookup tables, replacing if/else chains.
- The orchestrating `calculateTotalCents` function now reads from these tables. It contains no customer-type-specific or coupon-specific conditionals.

The public interface (`Order` struct/record and `calculateTotalCents`/`CalculateTotalCents`) was preserved unchanged.

## Why the new structure is easier to extend

**Before:** Adding `partner` required finding and editing three separate if/else chains in the middle of the main function. It was easy to miss the shipping surcharge check or the Black Friday section.

**After:** Adding `partner` is a single entry in `customerRules`:
```go
"partner": {
    baseDiscount:      func(_ int) int { return 12 },
    freeShipThreshold: 15000,
    blackFridayBonus:  intPtr(3),
},
```
And one entry in `couponRules` for `PARTNER5`. The main function does not change at all.

The same applies to adding a new country (one line in the shipping/tax maps), a new coupon (one entry in coupon rules), or a new customer type (one entry in customer rules).

## AI suggestion that was rejected

**Suggestion:** Use the **Strategy design pattern** — define a `CustomerPolicy` interface (or abstract class / trait) with methods `baseDiscount()`, `hasFreeShipping()`, `extraShipping()`, and `blackFridayBonus()`. Implement one concrete class per customer type (`VipPolicy`, `PremiumPolicy`, `EmployeePolicy`, etc.), and register them in a map.

**Why it was rejected:** For this problem size (5–6 customer types, no complex per-type behavior beyond a few values and a condition), the Strategy pattern introduces unnecessary structure. Every new customer type would require a new file and a new class, increasing indirection with no practical benefit over a config entry. The data-driven table approach achieves the same extensibility goal ("adding a type does not require editing the main function") with significantly less boilerplate and indirection. The kata's own constraint says "introduce structure only when justified by the problem" — Strategy is not justified here.
