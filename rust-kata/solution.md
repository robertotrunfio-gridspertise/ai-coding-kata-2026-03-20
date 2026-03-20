# Solution — Rust Pricing Module Refactor

## What was wrong in the legacy design

`calculate_total_cents` was a single ~100-line function that mixed three unrelated
concerns — discount calculation, shipping calculation, and tax calculation — into one
flat sequence of imperative steps with no internal structure.

Specific problems:

**No type safety on `customer_type`.**
Every branch compared a raw `String` with `==`. The compiler could not catch a typo
like `"employe"`, a new caller passing `"Employee"` (wrong case), or a new developer
adding a new type and forgetting to update one of the three separate spots where the
string was tested.

**No separation of concerns.**
Discount logic, shipping logic, and tax logic were interleaved. To answer "what discount
does a VIP customer get?" you had to read the whole function and mentally filter out the
shipping and tax blocks. Changing one concern risked accidentally touching another.

**Adding a new customer type required editing one large conditional block.**
There was no obvious place to add a new type. The three `if/else if` ladders (one for
discount, one for shipping, one for tax) all lived in the same function and all needed
simultaneous edits. There was no safety net to tell you whether you had covered every
relevant path.

**Zero tests.**
All the non-obvious behavioural quirks — coupon mutual exclusivity, the employee
surcharge running *after* free-shipping overrides, VIP tax reduction only in Italy,
TAXFREE blocked in Italy — were completely invisible. Any edit could silently break them.

**Dead and misleading code.**
The `regular` and `new` branches both executed `discount_percent += 0`, which is a
no-op. The Black Friday branch used `customer_type != "employee"` — a binary guard that
made it structurally impossible to express a third category (as `partner` later required)
without rewriting the condition.

---

## What changed

### A safety net came first (Phase 1)
Before touching a single line of logic, 31 integration tests were written against the
unmodified function. They called only the public API and asserted hand-calculated
expected values. All 31 passed on the legacy code. This made every subsequent
structural change safe to verify mechanically.

### `CustomerType` enum replaced stringly-typed comparisons (Phase 2)
A `pub(crate) enum CustomerType` with a `from_str` constructor centralised all
parsing. String comparisons throughout the codebase were replaced with exhaustive
`match` expressions. The compiler now enforces that every customer type is handled
everywhere it is relevant; a new variant cannot be silently ignored.

### Three pure functions were extracted (Phases 3–5)
Each concern was moved into its own module with a single public function:

- `calculate_discount_percent(customer_type, subtotal, coupon, black_friday) -> i32`
- `calculate_shipping_cents(customer_type, country, coupon, discounted_subtotal, black_friday) -> i32`
- `calculate_tax_percent(customer_type, country, coupon) -> i32`

All three functions are pure (no side effects, no shared mutable state) and are
independently testable without constructing an `Order`.

### `lib.rs` became a thin orchestrator (Phase 6)
With the three concerns extracted, `calculate_total_cents` shrank to ten lines: parse
inputs, call the three pure functions in order, sum the results, apply the floor guard.
It contains zero business logic and zero test blocks.

### The `partner` customer type was added as a proof of extensibility (Phase 7)
Eight integration tests were written and confirmed red before any implementation was
changed. The implementation touched exactly three files (`customer_type.rs`,
`discount.rs`, `shipping.rs`) and zero lines in `lib.rs` or `tax.rs`.

The final test suite has **95 tests**: 55 unit tests across the four modules and 40
integration tests against the public API.

---

## Why the new structure is easier to extend

Each rule type now has a single, bounded home.

To add a new **customer type** today:
1. Add a variant to `CustomerType` — the compiler immediately reports every `match`
   that needs updating.
2. Add one arm in the `match` inside `calculate_discount_percent` for the base discount.
3. If the type needs a free-shipping rule, add one `if` block in the ordered sequence
   inside `calculate_shipping_cents`.
4. If it needs a tax override, add one block in `calculate_tax_percent`.
5. `lib.rs` is not touched.

To add a new **coupon code**:
1. Add one `else if` branch to the mutually-exclusive coupon chain in
   `calculate_discount_percent`.
2. If it affects shipping, add a block in `calculate_shipping_cents`.
3. `lib.rs` is not touched.

The `partner` addition validated this exactly: the employee surcharge quirk, the Black
Friday pipeline, and the tax module were all completely unaffected. The Black Friday
binary guard (`!= employee`) was the only place where the old design would have forced a
structural rewrite — it was replaced with a three-way `match` that now accommodates any
number of per-type BF rates without further change:

```rust
discount_percent += match customer_type {
    CustomerType::Employee => 0,
    CustomerType::Partner  => 3,
    _                      => 5,
};
```

The step-numbered comments in `shipping.rs` make the ordering contract explicit and
durable. A future developer cannot accidentally move the employee surcharge before the
free-shipping overrides without seeing the comment that says why the order matters.

---

## One AI suggestion that was rejected and why

**Suggestion: introduce a `CustomerPricingRule` trait and implement it for each customer
type.**

The natural next step after extracting `CustomerType` was to go further and give each
variant its own behaviour via a trait:

```rust
trait CustomerPricingRule {
    fn base_discount_percent(&self, subtotal: i32) -> i32;
    fn shipping_free_threshold_cents(&self) -> Option<i32>;
    fn black_friday_bonus_percent(&self) -> i32;
}
```

Each `CustomerType` would implement the trait, and the `if/else` blocks inside
`calculate_discount_percent` and `calculate_shipping_cents` would be replaced with
method dispatch.

**Why it was rejected:**

First, the coupon pipeline cannot be moved onto the customer type. `VIPONLY` checks
`customer_type == Vip`; `PARTNER5` checks both type and subtotal. These are
*cross-cutting* rules that involve the coupon string, the customer type, and the subtotal
simultaneously. Putting them on the type would require passing context back into the
type, inverting the natural dependency direction.

Second, the `match` expressions that would remain in `calculate_discount_percent` (for
coupons and Black Friday) still need to dispatch on `CustomerType`. The trait would not
eliminate the match — it would add a layer of indirection on top of it.

Third, five customer types is not the threshold where a polymorphism abstraction earns
its complexity. The existing `match` in each function is already exhaustive, already
compiler-checked, and already readable in one screen. A trait with five implementations
scattered across five `impl` blocks would make it harder, not easier, to answer the
question "what is the total discount pipeline for a partner customer?" — you would need
to open five files instead of one.

The guiding constraint was: *introduce structure only when justified by the problem*.
The problem here was a monolithic function with no separation of concerns. Modules and
pure functions solved that. A trait hierarchy would have solved a problem that does not
yet exist.