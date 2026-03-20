# Refactor Legacy Checkout

Refactor the legacy checkout pricing module for the language: $ARGUMENTS

## What to do

### 1. Read the existing legacy code

Read the implementation file for the requested language:
- **go** → `go-kata/pricing/legacy_checkout.go`
- **java** → `java-kata/src/main/java/kata/LegacyCheckoutCalculator.java`
- **rust** → `rust-kata/src/lib.rs`

### 2. Write tests first (safety net)

Before changing any logic, add tests that cover the known correct behaviors from the README:

| Customer | Subtotal | Country | Coupon | Black Friday | Expected Total |
|----------|----------|---------|--------|--------------|----------------|
| regular  | 10000    | IT      | —      | no           | 12900          |
| premium  | 10000    | DE      | SAVE10 | no           | 10420          |
| vip      | 18000    | IT      | VIPONLY| no           | 17980          |

Also add tests for:
- Employee shipping surcharge (non-IT +500)
- VIP free shipping (discounted subtotal ≥ 15000)
- Premium free shipping (discounted subtotal ≥ 20000)
- Black Friday bonus (+5% for most, +0 for employee)
- FREESHIP coupon (sets shipping to 0 when discounted subtotal ≥ 8000)
- TAXFREE coupon (removes tax outside IT)
- BULK coupon (7% when subtotal ≥ 20000)
- Max discount cap (40%)
- Unknown customer type / empty coupon / unknown country

### 3. Refactor — do NOT change behavior

Replace the long if/else chains with a **data-driven / table-driven** approach:

- A map or match of **customer configs** (base discount fn, free ship threshold, extra ship fn, black friday bonus)
- A map or match of **coupon rules** (eligibility + discount amount)
- Separate helper functions/constants for **country shipping** and **country tax**
- One orchestrating function that reads from these tables

Rules:
- Preserve the exact same public interface (same struct fields, same function signature)
- No new external libraries
- No rewrite from scratch — start from the existing structure
- Introduce only the structure justified by the problem

### 4. Add the `partner` customer type

Partner rules:
- Base discount: **12%**
- Free shipping when discounted subtotal ≥ **15000** cents
- Coupon `PARTNER5` adds **+5%** only for partner customers **and** only when subtotal ≥ 12000 cents
- On Black Friday, partner gets **+3%** instead of the usual +5%

Add tests covering all four partner scenarios above.

### 5. Verify

Run the tests and confirm all pass before finishing.

## Language-specific notes

**Go** (`go-kata/`):
- Tests go in `go-kata/pricing/checkout_test.go` — use table-driven tests with `t.Run`
- Use `map[string]customerConfig` where fields can be functions (`func(int) int`)
- Use `*int` for nullable black-friday bonus (nil = default 5)
- Run: `cd go-kata && go test ./pricing/...`

**Java** (`java-kata/`):
- Tests go in `java-kata/src/test/java/kata/LegacyCheckoutCalculatorTest.java`
- Use JUnit 5 (`@Test`, `assertEquals`) — already in build.gradle
- Use `private record CustomerConfig(...)` with functional interface fields
- Nullable `Integer` for black-friday bonus (null = default 5)
- Run: `cd java-kata && ./gradlew test`

**Rust** (`rust-kata/`):
- Inline tests in `rust-kata/src/lib.rs` under `#[cfg(test)] mod tests`
- Use a `CustomerConfig` struct with `fn(i32) -> i32` function pointer fields
- Use a `coupon_discount(coupon, customer_type, subtotal)` match function
- Use `Option<i32>` for black-friday bonus (None = default 5)
- Run: `cd rust-kata && cargo test`
