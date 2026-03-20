# Copilot Instructions — Rust Legacy Checkout Kata

## Project Overview

This is a legacy checkout/pricing module refactoring kata in Rust. The crate is `legacy_checkout_kata` (edition 2024, no external dependencies allowed). The single source file is `rust-kata/src/lib.rs`.

## Current Code Structure

- `Order` struct with fields: `customer_type`, `subtotal_cents`, `country`, `coupon_code`, `black_friday`
- `calculate_total_cents(&Order) -> i32` — the main pricing function
- Known customer types: `regular`, `new`, `premium`, `vip`, `employee`
- Known coupons: `SAVE10`, `VIPONLY`, `BULK`, `FREESHIP`, `TAXFREE`
- Countries with specific rules: `IT`, `DE`, `US` (all others fall through to defaults)
- Discount is capped at 40%
- Total is floored at 0

## Refactoring Goal

Refactor `calculate_total_cents` so that:
- All existing behavior is preserved exactly (no regressions)
- The design becomes easier to extend (adding customer types/coupons should not require editing a large conditional block)
- A new customer type `partner` and coupon `PARTNER5` can be added safely
- The solution stays simple and understandable — no over-engineering

## New Requirement: `partner` Customer Type

- Base discount: 12%
- Free shipping when **discounted subtotal** >= 15000 cents
- Coupon `PARTNER5`: adds extra 5% discount, only for `partner`, only when **subtotal** >= 12000 cents
- On Black Friday: partner gets extra **3%** discount (not the usual 5% that other non-employee types get)

## Hard Constraints (NEVER violate)

1. **Preserve existing behavior** — do not change outputs for any existing input combination unless the new requirement explicitly demands it.
2. **Do not rewrite from scratch** — refactor incrementally.
3. **No new external dependencies** — only the Rust standard library.
4. **Do not change the input/output contract** — `Order` struct fields and `calculate_total_cents` signature must remain the same.
5. **Do not add conditional complexity** to the main legacy flow — extract, don't nest.
6. **Safety net first** — write comprehensive characterization tests before any structural change.
7. **Introduce abstractions only when justified** — a trait or enum is acceptable only if it genuinely reduces complexity.

## Workflow Rules

1. **Tests before refactoring**: Create thorough characterization tests covering all customer types, coupons, countries, Black Friday combinations, edge cases (zero subtotal, negative subtotal, discount cap at 40%, total floor at 0).
2. **Incremental refactoring**: Extract one concern at a time (discount calculation, shipping, tax). Run tests after each step.
3. **Then add `partner`**: Only after refactoring, add the new customer type and `PARTNER5` coupon with dedicated tests.
4. **Keep tests in the same file** using `#[cfg(test)] mod tests { ... }` — standard Rust convention.

## Coding Style

- Idiomatic Rust: prefer `match` over `if/else` chains where appropriate
- Use `&str` comparisons (no unnecessary allocations) when refactoring
- Derive `Debug, Clone` on types; consider `PartialEq` for test assertions
- Integer arithmetic only (cents) — no floating point
- Keep functions small and well-named
- Document non-obvious business rules with comments

## Key Business Rules Reference

### Discount (applied to subtotal)
| Customer  | Base Discount | Notes |
|-----------|--------------|-------|
| regular   | 0%           | |
| new       | 0%           | |
| premium   | 5% or 10%   | 10% when subtotal >= 10000 |
| vip       | 15%          | |
| employee  | 30%          | |
| partner   | 12%          | NEW |

### Coupons (additive to base discount)
| Coupon    | Extra Discount | Conditions |
|-----------|---------------|------------|
| SAVE10    | +10%          | subtotal >= 5000 |
| VIPONLY   | +5%           | customer = vip |
| BULK      | +7%           | subtotal >= 20000 |
| PARTNER5  | +5%           | customer = partner AND subtotal >= 12000 — NEW |
| FREESHIP  | 0% (shipping) | sets shipping to 0 when discounted_subtotal >= 8000 |
| TAXFREE   | 0% (tax)      | sets tax to 0 when country != IT |

### Black Friday (additive)
- All non-employee types: +5% discount (partner: +3% instead)
- US shipping surcharge: +300 cents

### Discount cap: 40% maximum

### Shipping
| Country | Base | Free shipping threshold |
|---------|------|------------------------|
| IT      | 700  | — |
| DE      | 900  | — |
| US      | 1500 | — |
| Other   | 2500 | — |
- vip: free when discounted_subtotal >= 15000
- premium: free when discounted_subtotal >= 20000
- partner: free when discounted_subtotal >= 15000 — NEW
- employee + country != IT: +500 surcharge
- FREESHIP coupon: free when discounted_subtotal >= 8000

### Tax
| Country | Rate | Notes |
|---------|------|-------|
| IT      | 22%  | vip in IT: 20% |
| DE      | 19%  | |
| US      | 7%   | |
| Other   | 0%   | |
- TAXFREE coupon: 0% when country != IT

### Total
`total = discounted_subtotal + shipping + tax` (minimum 0)

## Test Commands

Run tests with: `cd rust-kata && cargo test`

## Verification Examples

- `regular`, 10000, IT, no coupon, not BF → 12900
- `premium`, 10000, DE, SAVE10, not BF → 10420
- `vip`, 18000, IT, VIPONLY, not BF → 17980
