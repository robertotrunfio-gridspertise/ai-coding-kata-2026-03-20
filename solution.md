# Refactoring Solution

## What was wrong in the legacy design

### Single monolithic function
All business logic (~100 lines) lived inside one `calculate_total_cents` function. Discount, shipping, and tax rules were tangled together with no separation of concerns, making it impossible to reason about any single rule in isolation.

### Stringly-typed domain
Customer types (`"vip"`, `"employee"`, …), countries (`"IT"`, `"DE"`, …), and coupon codes (`"SAVE10"`, `"BULK"`, …) were all plain `String` values compared with `==`. A typo would silently produce wrong results with no compiler feedback.

### Magic numbers
Constants like `10000`, `5000`, `20000`, `700`, `1500`, `15000`, `22`, `7` were scattered across the code with no names. Their intent — thresholds, rates, fixed costs — could not be understood without reading surrounding context.

### Implicit, order-dependent mutations
`discount_percent`, `shipping_cents`, and `tax_percent` were declared early and then mutated across many loosely related `if` blocks. The final value depended on the order of execution, and adding a new rule anywhere in the chain risked silently breaking another.

### No testability at the rule level
Because everything was fused into one function, the only way to test a single rule was to construct a full `Order` and assert the final total — making it hard to identify which rule was broken when a test failed.

---

## What changed

### Enum types for the domain
Three enums replace raw strings:

- `CustomerType` — `Regular`, `New`, `Vip`, `Premium`, `Employee`, `Partner`
- `Country` — `IT`, `DE`, `US`, `Other`
- `CouponCode` — `None`, `Save10`, `VipOnly`, `Bulk`, `FreeShip`, `TaxFree`, `Partner5`

Each enum has a `parse()` method that converts the incoming string at the boundary of `calculate_total_cents`. From that point on, the compiler enforces exhaustive matching — an unknown value cannot be silently ignored.

### Named constants
Every threshold, rate, and fixed amount has a name:

```rust
const VIP_DISCOUNT: i32 = 15;
const PARTNER_FREE_SHIPPING_THRESHOLD: i32 = 15_000;
const BLACK_FRIDAY_US_SURCHARGE: i32 = 300;
// ...
```

A reader can now understand the intent of a rule without tracking down where the number comes from.

### Focused functions
The single function was split into eight clearly scoped functions:

| Function | Responsibility |
|---|---|
| `calculate_total_cents` | Entry point — parses inputs, orchestrates the three components |
| `calculate_discount_percent` | Combines discount sources and applies the cap |
| `customer_type_discount` | Discount from customer tier |
| `coupon_discount` | Discount from coupon code |
| `black_friday_discount` | Conditional Black Friday bonus |
| `calculate_shipping_cents` | Base shipping + all surcharges and waivers |
| `base_shipping` | Country → base shipping cost |
| `calculate_tax_cents` / `calculate_tax_percent` | Tax rate with overrides |

### Characterization tests
41 tests were added before and during the refactoring, covering every customer type, coupon code, country, Black Friday combination, and edge case. They serve as a safety net for future changes.

---

## Why the new structure is easier to extend

**Adding a new customer type** (e.g., `Partner`) requires:
1. One new variant in the `CustomerType` enum
2. One new arm in `CustomerType::parse()`
3. One new arm in `customer_type_discount()` for the base discount
4. Optionally: one new `if` block in `calculate_shipping_cents()` for free-shipping rules
5. Optionally: one new arm in `black_friday_discount()` if the BF rule differs

No existing logic needs to be touched. The compiler will warn if the new variant is missing from any exhaustive `match`.

**Adding a new coupon code** requires:
1. One new variant in `CouponCode` and one arm in `CouponCode::parse()`
2. One new arm in `coupon_discount()` (and/or the relevant shipping/tax function)

**Adding a new country** requires:
1. One new variant in `Country` and one arm in `Country::parse()`
2. One new arm in `base_shipping()` and `calculate_tax_percent()`

In every case, the change is localized to the relevant function, the scope of the edit is small and obvious, and the compiler guides the developer to every site that needs updating.
