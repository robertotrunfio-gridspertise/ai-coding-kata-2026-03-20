# Prompts

---

## Prompt 1 — Understand the legacy code

**Purpose:** Get a clear picture of what the legacy code does before touching it.

**Prompt:**
> Read `LegacyCheckoutCalculator.java` and `Order.java`. Summarize what the code does, identify the responsibilities mixed into the single method, and list any hidden quirks or implicit behaviors that must be preserved during refactoring.

**Outcome:**
Identified three distinct concerns in one method: discount calculation, shipping calculation, tax calculation. Found hidden quirks: `safe()` trims null/whitespace inputs silently; case-sensitivity of customer type and country strings; discount cap at 40%; `+ 0` no-ops that mask missing else branches; employee shipping surcharge for non-IT countries.

**Notes:**
Useful starting point before writing any tests. The quirks list directly shaped the boundary test cases.

---

## Prompt 2 — Write characterization tests before refactoring (test-oriented)

**Purpose:** Build a safety net that locks existing behavior so structural changes can be made confidently.

**Prompt:**
> Write JUnit 5 characterization tests for `LegacyCheckoutCalculator` that cover all customer types, all coupons and their threshold conditions, Black Friday effects, country-based shipping and tax rates, free shipping thresholds, the discount cap, and edge cases (null/empty/whitespace inputs, negative subtotal, unknown customer type, unknown coupon, case sensitivity). The goal is to preserve existing behavior exactly — do not test what the code should do, test what it currently does.

**Outcome:**
52 tests written covering all branches and boundary conditions. All passed against the original code before any refactoring.

**Notes:**
The case-sensitivity tests (`"VIP"` treated as unknown, `"it"` treated as other country) were discovered during this step — the legacy code never normalizes case. These are preserved as-is.

---

## Prompt 3 — Ask for design alternatives and tradeoffs before refactoring

**Purpose:** Understand the design space before committing to an approach, and make an informed choice.

**Prompt:**
> The legacy `calculateTotalCents` method mixes discount, shipping, and tax logic with customer-type conditionals scattered throughout. I need to add a `partner` customer type without making the conditional chain longer. What are the main design options for restructuring this? For each option, describe the tradeoffs: complexity, extensibility, how much it changes the existing code, and whether it fits the constraint of not rewriting everything from scratch.

**Outcome:**
Three approaches were proposed:
1. **Extract private methods only** — low risk, low extensibility gain; customer type conditionals remain
2. **Registry/data-driven customer rules** — moderate change, good extensibility; adding a type = adding one map entry
3. **Strategy pattern with interfaces** — high extensibility, high complexity; requires new files per type

Chose option 2 (registry) combined with option 1 (extracted methods). Option 3 was rejected.

**Notes:**
See `solution.md` for the rejected approach and the reasoning.

---

## Prompt 4 — Implement the refactoring while preserving behavior

**Purpose:** Perform the structural change guided by the chosen design, keeping the safety net green.

**Prompt:**
> Refactor `LegacyCheckoutCalculator` using a registry-based `CustomerRules` record and extracted private methods for discount, shipping, and tax. Requirements:
> - All 52 existing tests must continue to pass without modification
> - Do not change the `Order` record or the public `calculateTotalCents` signature
> - Do not add external libraries
> - Adding a new customer type should require only one line in the registry, not changes to the calculator logic
> - Keep `safe()` and all current input-handling behavior intact

**Outcome:**
Refactored into `CustomerRules` (registry record) and three focused private method groups in the calculator. All 52 tests passed without changes.

**Notes:**
The premium tiered discount (5% below 10000, 10% at or above) could not be fully data-driven without added complexity — it was kept as a named branch in `baseDiscount()` with a comment explaining why.

---

## Prompt 5 — Add the partner customer type

**Purpose:** Implement the new requirement cleanly using the refactored structure.

**Prompt:**
> Add the `partner` customer type with these rules: 12% base discount, free shipping when discounted subtotal >= 15000 cents, coupon `PARTNER5` adds 5% extra only for partners when subtotal >= 12000 cents, and on Black Friday partners get 3% discount instead of the usual 5%. Add tests for all partner-specific cases including the Black Friday coupon interaction. Do not add conditional complexity to the main flow.

**Outcome:**
Partner added as one registry entry in `CustomerRules`. `PARTNER5` added as one case in the coupon switch. The 3% Black Friday discount is expressed as `blackFridayDiscountPercent = 3` in the registry row — no special-casing in the calculator. 8 new partner tests added, all passing.

**Notes:**
A clarification was needed: "on Black Friday, partner gets an extra 3% instead of the usual 5%" refers to the general Black Friday discount, not the coupon. The `PARTNER5` coupon still gives +5%. This was confirmed before finalizing the test values.
