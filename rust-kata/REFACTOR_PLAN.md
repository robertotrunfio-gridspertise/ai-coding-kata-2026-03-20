# Refactor Plan: Rust Pricing Module

## Overview

This document describes a TDD-driven refactor of the `calculate_total_cents` function in
`src/lib.rs`. The goal is to preserve all existing behaviour while making the code easier to
understand, test, and extend — following the Single Responsibility Principle — without
changing the public `Order` / `calculate_total_cents` contract.

---

## Step 0 — Behavioural Audit

Before writing any tests, every rule hidden inside the function must be made explicit.

### 0.1 Discount Rules

Rules are applied additively; the result is capped at **40 %**.

| Source        | Condition                                | Discount |
|---------------|------------------------------------------|----------|
| Customer type | `vip`                                    | +15 %    |
| Customer type | `premium`, subtotal ≥ 10 000 ¢           | +10 %    |
| Customer type | `premium`, subtotal < 10 000 ¢           | + 5 %    |
| Customer type | `employee`                               | +30 %    |
| Customer type | `regular`, `new`, or unknown             |   0 %    |
| Customer type | `partner`                                | +12 %    |
| Coupon        | `SAVE10`, subtotal ≥ 5 000 ¢             | +10 %    |
| Coupon        | `VIPONLY`, customer is `vip`             | + 5 %    |
| Coupon        | `BULK`, subtotal ≥ 20 000 ¢              | + 7 %    |
| Coupon        | `PARTNER5`, customer is `partner`, subtotal ≥ 12 000 ¢ | + 5 %    |
| Black Friday  | any customer except `employee` and `partner` | + 5 %    |
| Black Friday  | `partner` customer                       | + 3 %    |
| Black Friday  | `employee` customer                      |   0 %    |

> **Note:** coupon codes are evaluated as a mutually-exclusive `else-if` chain, so only the
> first matching coupon applies.

> **Note:** the discount cap (`> 40 → 40`) is defensive code. With the current rule set the
> theoretical maximum is exactly 40 % (e.g. `employee` + `SAVE10`), so the `> 40` branch is
> never reached in practice; it must still be tested conceptually.

### 0.2 Shipping Rules (applied in this exact order)

1. **Base shipping** by country: IT = 700 ¢, DE = 900 ¢, US = 1 500 ¢, other = 2 500 ¢.
2. **Black Friday surcharge** (US only): +300 ¢.
3. **`FREESHIP` coupon override**: if `discounted_subtotal ≥ 8 000 ¢` → set to 0 ¢.
4. **VIP free shipping override**: if `discounted_subtotal ≥ 15 000 ¢` → set to 0 ¢.
5. **Premium free shipping override**: if `discounted_subtotal ≥ 20 000 ¢` → set to 0 ¢.
6. **Partner free shipping override**: if `discounted_subtotal ≥ 15 000 ¢` → set to 0 ¢.
7. **Employee surcharge** (non-IT only): +500 ¢.

> **Quirk:** steps 3–6 can set shipping to zero, but step 7 runs *after* them. An `employee`
> in a non-IT country will therefore pay 500 ¢ even when `FREESHIP` free-shipping is
> otherwise triggered. This behaviour must be preserved.

### 0.3 Tax Rules

| Country | Base rate | Override                                      |
|---------|-----------|-----------------------------------------------|
| IT      | 22 %      | `vip` in IT → 20 % (overrides base)           |
| DE      | 19 %      |                                               |
| US      |  7 %      |                                               |
| Other   |  0 %      |                                               |
| Any non-IT | —      | `TAXFREE` coupon → 0 % (overrides base)       |

### 0.4 Final Assembly

```
total = discounted_subtotal + shipping_cents + tax_cents
total = max(total, 0)
```

### 0.5 Input Sanitisation

The `safe()` helper trims leading/trailing whitespace from string fields. It does **not**
normalise case — `" vip "` is accepted, `" VIP "` is not.

---

## Phase 1 — Safety Net: Test the Legacy Implementation ✅

> **Principle:** before touching a single line of production code, write tests that lock in
> the existing behaviour. These tests are the safety net for every subsequent refactor step.
>
> These tests are **integration tests**: they exercise the public contract
> (`calculate_total_cents`) from the outside, exactly as a caller would. They live in
> `tests/checkout.rs` — Rust's conventional location for integration tests — so they never
> have access to internal implementation details and cannot accidentally couple to them.
>
> Each test calls `calculate_total_cents` directly and asserts an exact `i32` value computed
> by hand below.

### Hand-Calculated Test Cases

All monetary values are in **cents**.

---

#### Group A — Customer Type Discounts

**A1 — VIP in IT, no coupon, no Black Friday**
```
subtotal           = 10_000
discount           = 15 %
discounted         = 10_000 × 85 / 100 = 8_500
shipping           = 700  (IT base; VIP free-ship threshold 15_000, 8_500 < 15_000 → no)
tax                = 8_500 × 20 / 100 = 1_700  (VIP-in-IT override: 20 %)
total              = 8_500 + 700 + 1_700 = 10_900
```

**A2 — Premium (high subtotal) in DE, no coupon, no Black Friday**
```
subtotal           = 15_000
discount           = 10 %  (premium, ≥ 10_000)
discounted         = 15_000 × 90 / 100 = 13_500
shipping           = 900  (DE; premium free-ship threshold 20_000, 13_500 < 20_000 → no)
tax                = 13_500 × 19 / 100 = 2_565
total              = 13_500 + 900 + 2_565 = 16_965
```

**A3 — Premium (low subtotal) in DE, no coupon, no Black Friday**
```
subtotal           = 5_000
discount           = 5 %  (premium, < 10_000)
discounted         = 5_000 × 95 / 100 = 4_750
shipping           = 900
tax                = 4_750 × 19 / 100 = 902  (4_750 × 19 = 90_250; / 100 = 902)
total              = 4_750 + 900 + 902 = 6_552
```

**A4 — Employee in US, no coupon, no Black Friday**
```
subtotal           = 10_000
discount           = 30 %
discounted         = 10_000 × 70 / 100 = 7_000
shipping           = 1_500 + 500 (employee non-IT) = 2_000
tax                = 7_000 × 7 / 100 = 490
total              = 7_000 + 2_000 + 490 = 9_490
```

**A5 — Regular in IT, no coupon, no Black Friday**
```
subtotal           = 5_000
discount           = 0 %
discounted         = 5_000
shipping           = 700
tax                = 5_000 × 22 / 100 = 1_100
total              = 5_000 + 700 + 1_100 = 6_800
```

**A6 — `new` customer in IT, no coupon, no Black Friday**
```
(identical rules to `regular`)
total              = 6_800
```

**A7 — Unknown customer type ("corporate") in IT, no coupon, no Black Friday**
```
(falls to else branch → 0 % discount, identical to regular)
total              = 6_800
```

---

#### Group B — Coupon Codes

**B1 — SAVE10, subtotal at threshold (≥ 5 000), Regular in US**
```
subtotal           = 5_000
discount           = 10 %  (SAVE10 applies)
discounted         = 5_000 × 90 / 100 = 4_500
shipping           = 1_500
tax                = 4_500 × 7 / 100 = 315
total              = 4_500 + 1_500 + 315 = 6_315
```

**B2 — SAVE10, subtotal below threshold (< 5 000), Regular in US**
```
subtotal           = 4_999
discount           = 0 %  (SAVE10 does not apply)
discounted         = 4_999
shipping           = 1_500
tax                = 4_999 × 7 / 100 = 349  (4_999 × 7 = 34_993; / 100 = 349)
total              = 4_999 + 1_500 + 349 = 6_848
```

**B3 — VIPONLY applied to a VIP customer in US**
```
subtotal           = 10_000
discount           = 15 (vip) + 5 (VIPONLY) = 20 %
discounted         = 10_000 × 80 / 100 = 8_000
shipping           = 1_500  (VIP free-ship threshold 15_000, 8_000 < 15_000 → no)
tax                = 8_000 × 7 / 100 = 560
total              = 8_000 + 1_500 + 560 = 10_060
```

**B4 — VIPONLY applied to a non-VIP (Regular) customer in US**
```
subtotal           = 10_000
discount           = 0 %  (VIPONLY condition fails)
discounted         = 10_000
shipping           = 1_500
tax                = 10_000 × 7 / 100 = 700
total              = 10_000 + 1_500 + 700 = 12_200
```

**B5 — BULK, subtotal at threshold (≥ 20 000), Regular in US**
```
subtotal           = 20_000
discount           = 7 %  (BULK applies)
discounted         = 20_000 × 93 / 100 = 18_600
shipping           = 1_500
tax                = 18_600 × 7 / 100 = 1_302
total              = 18_600 + 1_500 + 1_302 = 21_402
```

**B6 — BULK, subtotal below threshold (< 20 000), Regular in US**
```
subtotal           = 19_999
discount           = 0 %  (BULK does not apply)
discounted         = 19_999
shipping           = 1_500
tax                = 19_999 × 7 / 100 = 1_399  (19_999 × 7 = 139_993; / 100 = 1_399)
total              = 19_999 + 1_500 + 1_399 = 22_898
```

**B7 — FREESHIP, discounted subtotal at threshold (≥ 8 000), Regular in DE**
```
subtotal           = 10_000
discount           = 0 %
discounted         = 10_000
shipping           = 900 → FREESHIP: 10_000 ≥ 8_000 → 0
tax                = 10_000 × 19 / 100 = 1_900
total              = 10_000 + 0 + 1_900 = 11_900
```

**B8 — FREESHIP, discounted subtotal below threshold (< 8 000), Regular in DE**
```
subtotal           = 7_000
discount           = 0 %
discounted         = 7_000
shipping           = 900  (FREESHIP: 7_000 < 8_000 → no free ship)
tax                = 7_000 × 19 / 100 = 1_330
total              = 7_000 + 900 + 1_330 = 9_230
```

**B9 — TAXFREE in a non-IT country (US)**
```
subtotal           = 10_000
discount           = 0 %
discounted         = 10_000
shipping           = 1_500
tax                = 0 %  (TAXFREE applies outside IT)
total              = 10_000 + 1_500 + 0 = 11_500
```

**B10 — TAXFREE in IT (coupon has no effect)**
```
subtotal           = 10_000
discount           = 0 %
discounted         = 10_000
shipping           = 700
tax                = 10_000 × 22 / 100 = 2_200  (TAXFREE ignored in IT)
total              = 10_000 + 700 + 2_200 = 12_900
```

---

#### Group C — Black Friday Flag

**C1 — Black Friday, non-employee in US**
```
subtotal           = 10_000
discount           = 5 %  (BF bonus)
discounted         = 10_000 × 95 / 100 = 9_500
shipping           = 1_500 + 300 (BF US surcharge) = 1_800
tax                = 9_500 × 7 / 100 = 665
total              = 9_500 + 1_800 + 665 = 11_965
```

**C2 — Black Friday, employee in US (no extra discount; BF shipping surcharge still applies)**
```
subtotal           = 10_000
discount           = 30 %  (employee; BF does NOT add extra)
discounted         = 10_000 × 70 / 100 = 7_000
shipping           = 1_500 + 300 (BF US) + 500 (employee non-IT) = 2_300
tax                = 7_000 × 7 / 100 = 490
total              = 7_000 + 2_300 + 490 = 9_790
```

**C3 — Black Friday, non-employee in IT (no BF shipping surcharge outside US)**
```
subtotal           = 10_000
discount           = 5 %
discounted         = 10_000 × 95 / 100 = 9_500
shipping           = 700  (IT; BF surcharge only applies in US)
tax                = 9_500 × 22 / 100 = 2_090
total              = 9_500 + 700 + 2_090 = 12_290
```

---

#### Group D — Discount Cap

**D1 — Employee + SAVE10 reaches exactly 40 % (boundary, no reduction)**
```
subtotal           = 10_000
discount           = 30 (employee) + 10 (SAVE10, ≥ 5_000) = 40 %  → not reduced
discounted         = 10_000 × 60 / 100 = 6_000
shipping           = 1_500 + 500 (employee non-IT) = 2_000
tax                = 6_000 × 7 / 100 = 420
total              = 6_000 + 2_000 + 420 = 8_420
```

---

#### Group E — Free Shipping Thresholds

**E1 — VIP free shipping triggered (discounted ≥ 15 000) in US**
```
subtotal           = 20_000
discount           = 15 %
discounted         = 20_000 × 85 / 100 = 17_000
shipping           = 1_500 → VIP: 17_000 ≥ 15_000 → 0
tax                = 17_000 × 7 / 100 = 1_190
total              = 17_000 + 0 + 1_190 = 18_190
```

**E2 — VIP free shipping NOT triggered (discounted < 15 000) in US**
```
subtotal           = 15_000
discount           = 15 %
discounted         = 15_000 × 85 / 100 = 12_750
shipping           = 1_500  (VIP: 12_750 < 15_000 → not free)
tax                = 12_750 × 7 / 100 = 892  (12_750 × 7 = 89_250; / 100 = 892)
total              = 12_750 + 1_500 + 892 = 15_142
```

**E3 — Premium free shipping triggered (discounted ≥ 20 000) in US**
```
subtotal           = 30_000
discount           = 10 %
discounted         = 30_000 × 90 / 100 = 27_000
shipping           = 1_500 → Premium: 27_000 ≥ 20_000 → 0
tax                = 27_000 × 7 / 100 = 1_890
total              = 27_000 + 0 + 1_890 = 28_890
```

**E4 — Employee in IT (no surcharge; employee surcharge is non-IT only)**
```
subtotal           = 10_000
discount           = 30 %
discounted         = 10_000 × 70 / 100 = 7_000
shipping           = 700  (IT; employee surcharge does NOT apply in IT)
tax                = 7_000 × 22 / 100 = 1_540
total              = 7_000 + 700 + 1_540 = 9_240
```

**E5 — Employee + FREESHIP quirk: employee surcharge applied AFTER free-ship override**
```
subtotal           = 12_000   (employee in DE with FREESHIP)
discount           = 30 %
discounted         = 12_000 × 70 / 100 = 8_400
shipping           = 900 → FREESHIP: 8_400 ≥ 8_000 → 0
                           employee (non-IT): 0 + 500 = 500
tax                = 8_400 × 19 / 100 = 1_596
total              = 8_400 + 500 + 1_596 = 10_496
```

---

#### Group F — Country / Tax Edge Cases

**F1 — Unknown country (e.g. "FR") — 0 % tax, 2 500 ¢ shipping**
```
subtotal           = 5_000
discount           = 0 %
discounted         = 5_000
shipping           = 2_500
tax                = 0 %
total              = 5_000 + 2_500 + 0 = 7_500
```

---

#### Group G — Input Sanitisation

**G1 — Whitespace trimmed from customer_type and country**
```
customer_type = " vip "  →  trimmed "vip"
country       = " IT "   →  trimmed "IT"
subtotal      = 10_000
(same computation as A1)
total         = 10_900
```

---

#### Group H — Interactions / Combinations

**H1 — VIP + VIPONLY in IT**
```
subtotal           = 10_000
discount           = 15 (vip) + 5 (VIPONLY) = 20 %
discounted         = 10_000 × 80 / 100 = 8_000
shipping           = 700  (VIP: 8_000 < 15_000 → no free ship)
tax                = 8_000 × 20 / 100 = 1_600  (VIP-in-IT: 20 %)
total              = 8_000 + 700 + 1_600 = 10_300
```

**H2 — VIP + Black Friday in US, free shipping triggered**
```
subtotal           = 20_000
discount           = 15 (vip) + 5 (BF) = 20 %
discounted         = 20_000 × 80 / 100 = 16_000
shipping           = 1_500 + 300 (BF US) = 1_800 → VIP: 16_000 ≥ 15_000 → 0
tax                = 16_000 × 7 / 100 = 1_120
total              = 16_000 + 0 + 1_120 = 17_120
```

**H3 — Zero subtotal (floor guard)**
```
subtotal           = 0   (Regular in US)
discount           = 0 %
discounted         = 0
shipping           = 1_500
tax                = 0
total              = 0 + 1_500 + 0 = 1_500
```

---

## Phase 2 — Extract `CustomerType` Enum ✅

**Goal:** replace stringly-typed customer logic with a type-safe internal enum.

**File:** `src/customer_type.rs` (new file); declared in `src/lib.rs` as `mod customer_type;`.

- Add `enum CustomerType { Vip, Premium, Employee, Regular, New, Unknown }` — `pub(crate)`.
- Add `impl CustomerType { pub(crate) fn from_str(s: &str) -> Self }` parsing trimmed,
  lowercase input.
- The public `Order` struct and the public signature of `calculate_total_cents` are **not changed**.
- Replace every string comparison of `customer_type` inside `calculate_total_cents` with a
  match on the new enum.
- Add inline `#[cfg(test)]` unit tests at the bottom of `src/customer_type.rs` covering
  every variant (including whitespace trimming and unknown fallback).
- Run all Phase 1 tests (`tests/checkout.rs`): all must pass.

---

## Phase 3 — Extract `calculate_discount_percent` ✅

**Goal:** isolate the entire discount pipeline into one pure function.

**File:** `src/discount.rs` (new file); declared in `src/lib.rs` as `mod discount;`.

```
pub(crate) fn calculate_discount_percent(
    customer_type: CustomerType,
    subtotal: i32,
    coupon: &str,
    black_friday: bool,
) -> i32
```

Rules (identical to the legacy logic):
1. Customer-type base discount.
2. Coupon additive discount (mutually-exclusive `match`).
3. Black Friday additive discount.
4. Cap at 40.

The function has **no side effects** and can be unit-tested in isolation.

Add inline `#[cfg(test)]` unit tests at the bottom of `src/discount.rs`:
- Each customer type at relevant subtotals.
- Each coupon (applies / does not apply).
- Black Friday for `partner` (3 %) vs. standard customers (5 %) vs. `employee` (0 %).
- Verify cap: `employee` + `SAVE10` with subtotal ≥ 5 000 → 40.

Run all Phase 1 (`tests/checkout.rs`) + Phase 2 tests: all must pass.

---

## Phase 4 — Extract `calculate_shipping_cents` ✅

**Goal:** isolate the entire shipping pipeline into one pure function.

**File:** `src/shipping.rs` (new file); declared in `src/lib.rs` as `mod shipping;`.

```
pub(crate) fn calculate_shipping_cents(
    customer_type: CustomerType,
    country: &str,
    coupon: &str,
    discounted_subtotal: i32,
    black_friday: bool,
) -> i32
```

Rules (identical to the legacy logic, same ordering):
1. Base by country.
2. Black Friday surcharge (US only).
3. FREESHIP override (if discounted_subtotal ≥ 8 000).
4. VIP free-ship override (if discounted_subtotal ≥ 15 000).
5. Premium free-ship override (if discounted_subtotal ≥ 20 000).
6. Employee surcharge (non-IT only) — applied last.

Add inline `#[cfg(test)]` unit tests at the bottom of `src/shipping.rs`:
- All four countries for base shipping.
- BF surcharge in US vs. non-US.
- FREESHIP at/below threshold.
- VIP, Premium, and Partner free-ship at/below their respective thresholds.
- Employee surcharge in IT (absent) vs. DE (present).
- E5 quirk: FREESHIP + employee → 500 ¢.

Run all prior tests: all must pass.

---

## Phase 5 — Extract `calculate_tax_percent` ✅

**Goal:** isolate the tax rate selection into one pure function.

**File:** `src/tax.rs` (new file); declared in `src/lib.rs` as `mod tax;`.

```
pub(crate) fn calculate_tax_percent(
    customer_type: CustomerType,
    country: &str,
    coupon: &str,
) -> i32
```

Rules (identical to the legacy logic):
1. Base rate by country.
2. VIP-in-IT override → 20 %.
3. TAXFREE in non-IT → 0 %.

Add inline `#[cfg(test)]` unit tests at the bottom of `src/tax.rs`:
- All four country cases.
- VIP in IT → 20.
- Regular in IT with TAXFREE → 22 (no effect).
- Regular in US with TAXFREE → 0.

Run all prior tests: all must pass.

---

## Phase 6 — Refactor `calculate_total_cents` to Compose Extracted Functions ✅

**Goal:** `src/lib.rs` becomes a thin crate root: module declarations, the public `Order`
struct, the public `calculate_total_cents` orchestrator, and the `safe` helper. No business
logic lives here.

```rust
// src/lib.rs
mod customer_type;
mod discount;
mod shipping;
mod tax;

use customer_type::CustomerType;
use discount::calculate_discount_percent;
use shipping::calculate_shipping_cents;
use tax::calculate_tax_percent;

pub struct Order { /* unchanged */ }

pub fn calculate_total_cents(order: &Order) -> i32 {
    let subtotal      = order.subtotal_cents;
    let customer_type = CustomerType::from_str(&order.customer_type);
    let country       = safe(&order.country);
    let coupon        = safe(&order.coupon_code);

    let discount_percent    = calculate_discount_percent(customer_type, subtotal, &coupon, order.black_friday);
    let discounted_subtotal = subtotal * (100 - discount_percent) / 100;
    let shipping_cents      = calculate_shipping_cents(customer_type, &country, &coupon, discounted_subtotal, order.black_friday);
    let tax_percent         = calculate_tax_percent(customer_type, &country, &coupon);
    let tax_cents           = discounted_subtotal * tax_percent / 100;

    let total = discounted_subtotal + shipping_cents + tax_cents;
    if total < 0 { 0 } else { total }
}

fn safe(value: &str) -> String { value.trim().to_string() }
```

- No new conditional logic is added to the main flow.
- `src/lib.rs` contains no `#[cfg(test)]` block — all tests live either inline in their
  module or in `tests/checkout.rs`.
- Run all prior tests: all must pass.

---

## Phase 7 — New Customer Type: `partner` ✅

**Rules to implement:**
- Base discount: 12 %.
- Coupon `PARTNER5`: +5 % if customer is `partner` **and** subtotal ≥ 12 000 ¢
  (sits in the existing mutually-exclusive coupon chain alongside `SAVE10`, `VIPONLY`, `BULK`).
- Black Friday bonus: +3 % (instead of the standard +5 %).
- Free shipping when discounted subtotal ≥ 15 000 ¢, inserted between the Premium override
  (step 5) and the Employee surcharge (step 7), preserving the employee-surcharge-runs-last quirk.

**Approach:** write all failing tests first (Groups P1–P8 below) in `tests/checkout.rs`,
then add `Partner` to `CustomerType` and update only `calculate_discount_percent` and
`calculate_shipping_cents`. The main flow (`src/lib.rs`) and `calculate_tax_percent`
(`src/tax.rs`) are **not changed**.

### Hand-Calculated Test Cases

**P1 — Partner, basic discount, US, no coupon, no BF**
```
subtotal           = 10_000
discount           = 12 %
discounted         = 10_000 × 88 / 100 = 8_800
shipping           = 1_500  (US; partner free-ship: 8_800 < 15_000 → no)
tax                = 8_800 × 7 / 100 = 616
total              = 8_800 + 1_500 + 616 = 10_916
```

**P2 — Partner, free shipping triggered (discounted ≥ 15 000), US, no coupon, no BF**
```
subtotal           = 20_000
discount           = 12 %
discounted         = 20_000 × 88 / 100 = 17_600
shipping           = 1_500 → partner: 17_600 ≥ 15_000 → 0
tax                = 17_600 × 7 / 100 = 1_232
total              = 17_600 + 0 + 1_232 = 18_832
```

**P3 — Partner, free shipping NOT triggered (discounted < 15 000), US, no coupon, no BF**
```
subtotal           = 15_000
discount           = 12 %
discounted         = 15_000 × 88 / 100 = 13_200
shipping           = 1_500  (partner: 13_200 < 15_000 → not free)
tax                = 13_200 × 7 / 100 = 924
total              = 13_200 + 1_500 + 924 = 15_624
```

**P4 — Partner + PARTNER5 applied (subtotal ≥ 12 000), IT, no BF**
```
subtotal           = 12_000
discount           = 12 (partner) + 5 (PARTNER5, ≥ 12_000) = 17 %
discounted         = 12_000 × 83 / 100 = 9_960
shipping           = 700  (IT; partner free-ship: 9_960 < 15_000 → no)
tax                = 9_960 × 22 / 100 = 2_191  (9_960 × 22 = 219_120; / 100 = 2_191)
total              = 9_960 + 700 + 2_191 = 12_851
```

**P5 — Partner + PARTNER5 not applied (subtotal < 12 000), IT, no BF**
```
subtotal           = 11_999
discount           = 12 %  (PARTNER5 does not apply: subtotal < 12_000)
discounted         = 11_999 × 88 / 100 = 10_559  (11_999 × 88 = 1_055_912; / 100 = 10_559)
shipping           = 700  (IT; 10_559 < 15_000 → not free)
tax                = 10_559 × 22 / 100 = 2_322  (10_559 × 22 = 232_298; / 100 = 2_322)
total              = 10_559 + 700 + 2_322 = 13_581
```

**P6 — PARTNER5 coupon applied to a non-partner customer (no effect), IT**
```
customer_type      = regular
subtotal           = 12_000
coupon             = PARTNER5
discount           = 0 %  (PARTNER5 condition fails: customer is not partner)
discounted         = 12_000
shipping           = 700
tax                = 12_000 × 22 / 100 = 2_640
total              = 12_000 + 700 + 2_640 = 15_340
```

**P7 — Partner + Black Friday (3 % bonus, not 5 %), US**
```
subtotal           = 10_000
discount           = 12 (partner) + 3 (BF: partner rate) = 15 %
discounted         = 10_000 × 85 / 100 = 8_500
shipping           = 1_500 + 300 (BF US) = 1_800  (partner: 8_500 < 15_000 → not free)
tax                = 8_500 × 7 / 100 = 595
total              = 8_500 + 1_800 + 595 = 10_895
```

**P8 — Partner + Black Friday + PARTNER5, US, free shipping triggered**
```
subtotal           = 20_000
discount           = 12 (partner) + 5 (PARTNER5, ≥ 12_000) + 3 (BF) = 20 %
discounted         = 20_000 × 80 / 100 = 16_000
shipping           = 1_500 + 300 (BF US) = 1_800 → partner: 16_000 ≥ 15_000 → 0
tax                = 16_000 × 7 / 100 = 1_120
total              = 16_000 + 0 + 1_120 = 17_120
```

### Implementation Changes (confined to two functions)

- **`CustomerType`**: add `Partner` variant; update `from_str` to map `"partner"` to it.
- **`calculate_discount_percent`**:
  - Add `Partner => 12` arm to the customer-type match.
  - Add `"PARTNER5"` branch to the coupon chain: applies only when
    `customer_type == Partner && subtotal >= 12_000`, adds +5 %.
  - Replace the binary BF `if customer_type != employee` guard with a three-way match:
    `Employee → 0`, `Partner → 3`, `_ → 5`.
- **`calculate_shipping_cents`**:
  - Insert a `Partner` free-ship check (`discounted_subtotal >= 15_000 → 0`) after the
    Premium override and before the Employee surcharge, keeping step ordering intact.

All Phase 1 tests in `tests/checkout.rs` (Groups A–H) must remain green after this phase.

---

## Constraints Checklist


| Constraint                                        | How it is met                                                  |
|---------------------------------------------------|----------------------------------------------------------------|
| Preserve existing behaviour                       | Phase 1 safety-net tests run after every step                  |
| Do not rewrite from scratch                       | Each phase is an incremental extraction, not a replacement     |
| No new external libraries                         | Only `std`; no `Cargo.toml` changes                            |
| Do not change the input/output contract           | `Order` struct and `calculate_total_cents` signature unchanged |
| No more conditional complexity in the main flow   | Main function becomes pure composition, adds zero branches     |
| Add a safety net before structural changes        | Phase 1 is completed before Phase 2 begins                     |
| Introduce structure only when justified           | Each extraction addresses a distinct, named responsibility     |

---

## File Layout After Refactor

```
src/
├── lib.rs                 ← crate root: mod declarations, pub Order, pub calculate_total_cents, fn safe
│                            no business logic, no #[cfg(test)]
├── customer_type.rs       ← pub(crate) enum CustomerType (all variants incl. Partner)
│                            pub(crate) fn from_str
│                            #[cfg(test)]: from_str parsing, whitespace, unknown fallback
├── discount.rs            ← pub(crate) fn calculate_discount_percent
│                            #[cfg(test)]: per-customer-type discounts, all coupons,
│                                          BF three-way (partner/employee/other), cap boundary
├── shipping.rs            ← pub(crate) fn calculate_shipping_cents
│                            #[cfg(test)]: base by country, BF surcharge, FREESHIP,
│                                          VIP/Premium/Partner free-ship thresholds,
│                                          employee surcharge, E5 quirk
└── tax.rs                 ← pub(crate) fn calculate_tax_percent
                             #[cfg(test)]: all countries, VIP-in-IT override, TAXFREE

tests/
└── checkout.rs            ← integration tests against the public API only
                             Phase 1: Groups A–H (legacy behaviour, 26 cases)
                             Phase 7: Groups P1–P8 (partner customer type, 8 cases)
```

Each module owns its own unit tests. The integration test file owns the end-to-end contract
tests. `src/lib.rs` is never a test file.
