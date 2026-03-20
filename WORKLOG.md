# Worklog — Rust Pricing Module Refactor

---

## 2026-03-20 — Phase 1: Safety Net Integration Tests

**Status:** ✅ Complete

Created `tests/checkout.rs` with 31 integration tests covering all documented behaviours of
the legacy `calculate_total_cents` function. Tests call only the public API and assert
hand-calculated expected values.

**Test groups:**
- Group A (7 tests): customer type discounts — vip, premium (high/low), employee, regular, new, unknown
- Group B (10 tests): all coupon codes — SAVE10, VIPONLY, BULK, FREESHIP, TAXFREE (at/below each threshold)
- Group C (3 tests): Black Friday — non-employee discount bonus, employee exemption, non-US shipping
- Group D (1 test): discount cap boundary — employee + SAVE10 = exactly 40 %
- Group E (5 tests): free shipping thresholds — VIP, Premium, employee IT/non-IT, employee-after-FREESHIP quirk
- Group F (1 test): unknown country — default 2 500 ¢ shipping, 0 % tax
- Group G (1 test): whitespace trimming via `safe()`
- Group H (3 tests): interaction combinations — VIP+VIPONLY, VIP+BF free-ship, zero subtotal floor

**Result:** 31/31 passed on the unmodified legacy code.

**Files created:**
- `rust-kata/tests/checkout.rs`

---

## 2026-03-20 — Phase 2: Extract `CustomerType` Enum

**Status:** ✅ Complete

Replaced all stringly-typed customer comparisons in `lib.rs` with a `pub(crate)` enum.
`CustomerType::from_str` encapsulates the parsing logic (trim-only, no case normalisation)
and is the single place that maps raw strings to variants.

**Unit tests added** (inline in `src/customer_type.rs`):
- `parses_all_known_variants` — vip, premium, employee, regular, new
- `unknown_strings_map_to_unknown` — corporate, wholesale, empty string
- `leading_and_trailing_whitespace_is_trimmed` — space, double-space, tab
- `case_is_not_normalised_uppercase_maps_to_unknown` — VIP, Premium, EMPLOYEE

**Result:** 35/35 passed (4 new unit tests + 31 integration tests).

**Files created:**
- `rust-kata/src/customer_type.rs`

**Files modified:**
- `rust-kata/src/lib.rs` — added `mod customer_type`, replaced string comparisons with enum
  matches, replaced chained `if/else if` with `match`, simplified Black Friday guard

---

## 2026-03-20 — Phase 3: Extract `calculate_discount_percent`

**Status:** ✅ Complete

Moved the entire discount pipeline (customer-type base discount, mutually-exclusive coupon
chain, Black Friday bonus, 40 % cap) into a pure `pub(crate)` function in `src/discount.rs`.
The function has no side effects and is independently testable.

**Unit tests added** (inline in `src/discount.rs`):
- All 7 customer-type base cases (including both Premium thresholds)
- SAVE10 at/below 5 000 ¢ threshold
- VIPONLY for VIP vs. non-VIP
- BULK at/below 20 000 ¢ threshold
- Coupon mutual-exclusivity assertion
- Black Friday bonus for non-employee vs. employee
- Discount cap boundary: employee + SAVE10 = 40 %

**Result:** 52/52 passed (17 new unit tests + previous 35).

**Files created:**
- `rust-kata/src/discount.rs`

**Files modified:**
- `rust-kata/src/lib.rs` — added `mod discount`, replaced inline discount block with a
  single `calculate_discount_percent(...)` call

---

## 2026-03-20 — Phase 4: Extract `calculate_shipping_cents`

**Status:** ✅ Complete

Moved the entire shipping pipeline into a pure `pub(crate)` function in `src/shipping.rs`.
The step ordering (base → BF surcharge → free-ship overrides → employee surcharge last) is
preserved exactly. Each step is commented in the source so the ordering intent is explicit.

**Unit tests added** (inline in `src/shipping.rs`):
- Base rates for IT, DE, US, unknown country
- Black Friday surcharge applies only in US (including for employees)
- Black Friday surcharge absent outside US
- FREESHIP at/below 8 000 ¢ threshold
- VIP free-ship at/below 15 000 ¢ threshold
- Premium free-ship at/below 20 000 ¢ threshold
- Employee surcharge present in DE, absent in IT
- **Employee-after-free-ship quirk (E5):** FREESHIP sets shipping to 0 then employee adds 500

**Result:** 69/69 passed (17 new unit tests + previous 52).

**Files created:**
- `rust-kata/src/shipping.rs`

**Files modified:**
- `rust-kata/src/lib.rs` — added `mod shipping`, replaced inline shipping block with a
  single `calculate_shipping_cents(...)` call

---

## 2026-03-20 — Phase 5 + Phase 6: Extract `calculate_tax_percent` / Final Orchestrator

**Status:** ✅ Complete

Moved the tax pipeline into a pure `pub(crate)` function in `src/tax.rs`. With all three
responsibilities extracted, `src/lib.rs` became the clean orchestrator described in Phase 6
without any additional changes — extracting the last function completes the composition.

**Unit tests added** (inline in `src/tax.rs`):
- Base rates for IT (22 %), DE (19 %), US (7 %), unknown (0 %)
- VIP-in-IT reduced rate (20 %)
- VIP outside IT uses standard country rate
- Non-VIP in IT uses standard 22 %
- TAXFREE sets rate to 0 for US, DE, and unknown countries
- TAXFREE has no effect in IT
- TAXFREE has no effect on VIP-in-IT (20 % preserved)

**`src/lib.rs` final state:**
- Module declarations only (`mod customer_type`, `mod discount`, `mod shipping`, `mod tax`)
- `pub struct Order` (unchanged)
- `pub fn calculate_total_cents` — 10 lines: parse inputs, call three pure functions, sum, floor
- `fn safe` — unchanged helper
- Zero `#[cfg(test)]` blocks — all tests live in their respective modules or in `tests/`

**Result:** 79/79 passed (10 new unit tests + previous 69).

**Files created:**
- `rust-kata/src/tax.rs`

**Files modified:**
- `rust-kata/src/lib.rs` — added `mod tax`, replaced inline tax block with a single
  `calculate_tax_percent(...)` call; function is now a pure recipe with no business logic

---

## 2026-03-20 — Phase 7: New Customer Type `partner`

**Status:** ✅ Complete

Added the `partner` customer type following strict TDD: all 8 integration tests (Group P)
were written and confirmed red before any implementation file was touched.

**Partner rules implemented:**
- Base discount: 12 %
- Coupon `PARTNER5`: +5 % when customer is `partner` **and** subtotal ≥ 12 000 ¢; sits in
  the existing mutually-exclusive coupon chain
- Black Friday bonus: +3 % (binary `!= employee` guard replaced with a three-way match:
  `Employee → 0`, `Partner → 3`, `_ → 5`)
- Free shipping when discounted subtotal ≥ 15 000 ¢, inserted as step 6 between the Premium
  override and the employee surcharge — preserving the employee-surcharge-runs-last quirk

**Changes confined to three files as planned:**

`src/customer_type.rs`
- Added `Partner` variant to the enum
- Added `"partner"` arm to `from_str`
- Extended unit tests: `parses_all_known_variants`, `leading_and_trailing_whitespace_is_trimmed`,
  `case_is_not_normalised_uppercase_maps_to_unknown`

`src/discount.rs`
- Added `CustomerType::Partner => discount_percent += 12` arm to the customer-type match
- Added `"PARTNER5"` branch to the coupon chain (after `"BULK"`)
- Replaced binary BF guard with three-way `match` on `customer_type`
- New unit tests: `partner_gets_12_percent`, `partner5_applies_to_partner_at_threshold`,
  `partner5_does_not_apply_below_threshold`, `partner5_has_no_effect_for_non_partner`,
  `black_friday_adds_3_for_partner`; renamed `black_friday_adds_5_for_non_employee`
  → `black_friday_adds_5_for_standard_customers`

`src/shipping.rs`
- Inserted Partner free-ship check (`discounted_subtotal >= 15_000 → 0`) as step 6,
  between the Premium override (step 5) and the employee surcharge (step 7)
- Updated step comment from `3–5` to `3–6`
- New unit tests: `partner_free_shipping_at_threshold`,
  `partner_free_shipping_not_triggered_below_threshold`

**`src/lib.rs` and `src/tax.rs` were not touched.**

**Result:** 94/94 passed (15 new tests + previous 79).

**Files modified:**
- `rust-kata/src/customer_type.rs`
- `rust-kata/src/discount.rs`
- `rust-kata/src/shipping.rs`

---

## Final Test Inventory (updated)

| Suite | Location | Tests |
|---|---|---|
| `customer_type` unit tests | `src/customer_type.rs` | 5 |
| `discount` unit tests | `src/discount.rs` | 26 |
| `shipping` unit tests | `src/shipping.rs` | 19 |
| `tax` unit tests | `src/tax.rs` | 10 |
| Integration tests (A–H legacy + P partner) | `tests/checkout.rs` | 39 |
| **Total** | | **99** |
