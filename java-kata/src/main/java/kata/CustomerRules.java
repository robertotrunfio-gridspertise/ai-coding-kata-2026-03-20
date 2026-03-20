package kata;

import java.util.Map;

/**
 * Holds the pricing rules for each known customer type.
 * Adding a new customer type means adding one entry here — no changes to the calculator.
 */
record CustomerRules(
        int baseDiscountPercent,
        int freeShippingThresholdCents,  // 0 = no free-shipping threshold
        int blackFridayDiscountPercent   // 0 = exempt from Black Friday discount
) {
    static final CustomerRules DEFAULT = new CustomerRules(0, 0, 5);

    private static final Map<String, CustomerRules> REGISTRY = Map.of(
            "regular",  new CustomerRules(0,     0,     5),
            "new",      new CustomerRules(0,     0,     5),
            "premium",  new CustomerRules(10,    20000, 5), // 10% only when subtotal >= 10000 (handled below)
            "vip",      new CustomerRules(15,    15000, 5),
            "employee", new CustomerRules(30,    0,     0), // exempt from BF discount
            "partner",  new CustomerRules(12,    15000, 3)  // partner gets 3% on BF instead of 5%
    );

    static CustomerRules forType(String customerType) {
        return REGISTRY.getOrDefault(customerType, DEFAULT);
    }
}
