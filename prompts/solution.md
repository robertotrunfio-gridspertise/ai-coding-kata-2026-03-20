# Solution

## What was wrong in the legacy design

The original `calculate_total_cents` function was a single monolithic function (~100 lines) with several structural problems:

1. **Long `if/else` chains for unrelated concerns** — Customer-type discounts, coupon logic, Black Friday rules, shipping, and tax were all interleaved in one sequential block. There was no separation between business domains.

2. **Duplicated conditional patterns** — The `customer_type` string was checked repeatedly in the discount section, the shipping section, and the tax section. The `coupon` string was similarly scattered across discount and shipping logic.

3. **Mixed responsibilities** — A single function computed discounts, applied coupons, determined shipping costs (including free shipping thresholds), calculated taxes (including per-country and per-customer overrides), and enforced the total floor. These are five distinct business concerns collapsed into one.

4. **Hidden interaction order** — Shipping depends on `discounted_subtotal`, which depends on the discount cap, which depends on the coupon and Black Friday rules. The FREESHIP coupon zeroes shipping, but the employee surcharge is applied *after* that, silently re-adding cost. These interactions are invisible without reading line by line.

5. **Poor extensibility** — Adding a new customer type required editing at least three separate `if/else` chains (discount, shipping, tax) and carefully inserting new branches in the correct order to avoid breaking existing interactions.

6. **No safety net** — There were zero tests, making any change high-risk.

## What changed

### Phase 1 — Safety net (102 characterization tests)

Before touching any production code, a comprehensive test suite was written covering:
- All 5 customer types × 4 countries
- All 5 coupons (qualifying and non-qualifying thresholds)
- Black Friday interactions per customer type
- Discount cap at 40%
- Shipping: free shipping thresholds for vip/premium, employee surcharge, FREESHIP coupon, BF US surcharge
- Tax: VIP IT override (20% vs 22%), TAXFREE coupon blocked in IT
- Edge cases: zero, negative, and 1-cent subtotals; empty/unknown strings; whitespace trimming; case sensitivity

### Phase 2 — Implement `partner` requirement (4 surgical additions)

With the safety net in place, four additions were made to the existing function — no existing lines were modified:

| Addition | What it does |
|---|---|
| `else if customer_type == "partner"` in discount chain | Base discount: 12% |
| `else if coupon == "PARTNER5"` in coupon chain | +5% when customer=partner AND subtotal ≥ 12000 |
| `if customer_type == "partner"` in BF block | +3% on Black Friday (instead of the +5% other non-employees get) |
| `if customer_type == "partner"` in shipping | Free shipping when discounted_subtotal ≥ 15000 |

### Phase 3 — Partner test suite (29 new tests)

New tests cover: base discount across all countries, free shipping threshold boundaries, PARTNER5 qualifying/non-qualifying/ignored-for-other-types, Black Friday +3%, coupon stacking (SAVE10, BULK), FREESHIP and TAXFREE interactions, and edge cases (zero/negative subtotals, exact thresholds).

**Final result: 131 tests, all passing, zero regressions.**

## Why the new structure is easier to extend

The current approach — adding branches to clearly delineated sections — is a minimal-but-effective improvement over the original because:

1. **Each section handles one concern.** The discount chain, coupon chain, BF block, shipping block, and tax block are sequential and self-contained. Adding a new customer type means adding one branch per section, not rewriting conditionals.

2. **The test suite is the real safety net.** The 102 legacy characterization tests catch any accidental regressions instantly. Adding `partner` required zero changes to existing tests — proof that the additions were non-destructive.

3. **Extension pattern is clear.** To add another customer type (e.g., `affiliate`), the developer follows the same pattern: one branch in the discount chain, one in shipping if needed, and new tests. The 29 partner tests serve as a template.

4. **No over-engineering.** The constraints explicitly said "introduce structure only when justified." A trait hierarchy or strategy pattern would add indirection without reducing the actual complexity of the business rules (which are inherently conditional). The current flat structure is easier to read and trace.

## One AI suggestion that was rejected and why

**Rejected: Extracting business rules into a trait-based strategy pattern.**

The AI proposed creating a `CustomerPolicy` trait with methods like `base_discount()`, `free_shipping_threshold()`, and `black_friday_bonus()`, with one implementation per customer type (`VipPolicy`, `PremiumPolicy`, `PartnerPolicy`, etc.). A factory function would map the `customer_type` string to the appropriate implementation.

**Why it was rejected:**

- **Complexity doesn't justify it.** The current module has 6 customer types. A trait + 6 structs + a factory function would triple the code surface without making the business logic any clearer — the conditionals would just move from `if/else` branches into trait method bodies.

- **Not all rules are per-customer.** Coupons, tax, and country-based shipping are orthogonal to customer type. The strategy pattern would only cover part of the logic, leaving the rest as-is. This would create an inconsistent hybrid that's *harder* to reason about.

- **The real problem was the lack of tests, not the lack of abstractions.** With 131 tests protecting the behavior, the flat conditional structure is safe to extend. If the module grows to 15+ customer types, the trait approach would become justified — but premature abstraction now would violate YAGNI.

- **The kata constraints explicitly warn against it:** "Introduce structure only when justified by the problem" and "The design pattern is not the goal by itself."
