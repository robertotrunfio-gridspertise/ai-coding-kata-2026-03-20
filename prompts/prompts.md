# Prompts Used During Kata

---

## Prompt 1 — Understand and preserve existing behavior

**Prompt:**
> Read the legacy checkout code and identify all business rules it currently implements. List each rule explicitly (customer discounts, coupon logic, shipping adjustments, tax rates, caps and overrides). Do not change anything yet. I want a safety net before touching the logic.

**Purpose:**
To extract a precise, exhaustive list of existing behaviors so tests can be written to lock them in before any structural change is made.

**Outcome:**
Produced a complete inventory of rules: 4 customer discount tiers (including subtotal-dependent premium), 3 coupon behaviors, Black Friday bonuses (with employee exception), 3 country shipping rates + surcharges, 2 free-shipping conditions, country tax rates + VIP Italy override, and the 40% discount cap.

**Notes:**
This prompt directly satisfies the "preserve existing behavior" constraint. Writing the safety net first was a deliberate choice — it forces understanding before refactoring and prevents silent regressions.

---

## Prompt 2 — Design alternatives and tradeoffs

**Prompt:**
> Given the business rules I just listed, propose two or three different ways to restructure the code so that adding a new customer type no longer requires editing a large if/else chain. Describe each option with its tradeoffs: simplicity, extensibility, testability, and alignment with the constraint "do not rewrite everything from scratch". Which would you recommend for this specific problem and why?

**Purpose:**
To evaluate structural options before committing to one, and to document the rejected alternative as required.

**Outcome:**
Three options were proposed:
1. **Strategy pattern** — one class/interface per customer type, registered in a map. Highly OO, but over-engineered for 5–6 customer types; every new type requires a new file.
2. **Data-driven config table** — a map of lightweight config structs with small functions/lambdas per field. Simple, testable, no boilerplate. Adding a customer type is one map entry.
3. **Single function with extracted private methods** — same if/else logic, just broken into `discountFor()`, `shippingFor()`, etc. Cleaner surface but still requires editing conditional blocks to add types.

Option 2 was selected. Option 1 was rejected (see `solution.md`).

**Notes:**
This prompt surfaced the key design decision early, making the tradeoff explicit and intentional rather than accidental.

---

## Prompt 3 — Test-oriented safety net

**Prompt:**
> Before I refactor anything, write a comprehensive set of tests for the current checkout behavior. Use the examples from the README as your starting cases, then add tests for: each customer type's discount, each coupon's eligibility condition, the VIP and Premium free-shipping thresholds, employee shipping surcharge, Black Friday bonuses, the 40% discount cap, TAXFREE and FREESHIP edge cases, and unknown/empty inputs. Tests must pass against the current legacy code. Each test should be a single, isolated case — no shared mutable state.

**Purpose:**
To create a safety net that makes refactoring safe. If any test breaks after a structural change, the behavior has regressed.

**Outcome:**
25+ table-driven test cases covering all identified business rules. Validated against the unmodified legacy code before applying any structural changes.

**Notes:**
The three README examples were used as the first three tests — these are the golden cases. Additional edge cases (unknown country, null/empty strings, discount cap) were added to catch regressions in less obvious paths.
