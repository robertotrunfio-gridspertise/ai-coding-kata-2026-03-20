package kata;

import java.util.Map;

public class LegacyCheckoutCalculator {

    @FunctionalInterface
    private interface DiscountFn { int apply(int subtotal); }

    @FunctionalInterface
    private interface ShippingExtraFn { int apply(String country); }

    @FunctionalInterface
    private interface CouponFn { int apply(String customerType, int subtotal); }

    private record CustomerConfig(
            DiscountFn baseDiscount,
            int freeShipThreshold,     // 0 = no threshold
            ShippingExtraFn extraShip, // null = none
            Integer blackFridayBonus   // null = default 5
    ) {}

    private static final Map<String, CustomerConfig> CUSTOMER_RULES = Map.of(
            "vip",      new CustomerConfig(s -> 15,                         15000, null,          null),
            "premium",  new CustomerConfig(s -> s >= 10000 ? 10 : 5,       20000, null,          null),
            "employee", new CustomerConfig(s -> 30,                         0,     c -> c.equals("IT") ? 0 : 500, 0),
            "regular",  new CustomerConfig(s -> 0,                          0,     null,          null),
            "new",      new CustomerConfig(s -> 0,                          0,     null,          null),
            "partner",  new CustomerConfig(s -> 12,                         15000, null,          3)
    );

    private static final Map<String, CouponFn> COUPON_RULES = Map.of(
            "SAVE10",   (ct, s) -> s >= 5000 ? 10 : 0,
            "VIPONLY",  (ct, s) -> ct.equals("vip") ? 5 : 0,
            "BULK",     (ct, s) -> s >= 20000 ? 7 : 0,
            "PARTNER5", (ct, s) -> ct.equals("partner") && s >= 12000 ? 5 : 0
    );

    private static final Map<String, Integer> COUNTRY_SHIPPING = Map.of("IT", 700, "DE", 900, "US", 1500);
    private static final Map<String, Integer> COUNTRY_TAX      = Map.of("IT", 22,  "DE", 19,  "US", 7);

    private static final int DEFAULT_SHIPPING = 2500;
    private static final int MAX_DISCOUNT     = 40;
    private static final int DEFAULT_BF_BONUS = 5;

    public int calculateTotalCents(Order order) {
        int subtotal = order.subtotalCents();
        String ct      = safe(order.customerType());
        String country = safe(order.country());
        String coupon  = safe(order.couponCode());

        CustomerConfig customer = CUSTOMER_RULES.getOrDefault(ct,
                new CustomerConfig(s -> 0, 0, null, null));

        // Discount
        int discount = customer.baseDiscount().apply(subtotal);
        if (COUPON_RULES.containsKey(coupon)) {
            discount += COUPON_RULES.get(coupon).apply(ct, subtotal);
        }
        if (order.blackFriday()) {
            int bonus = customer.blackFridayBonus() != null ? customer.blackFridayBonus() : DEFAULT_BF_BONUS;
            discount += bonus;
        }
        if (discount > MAX_DISCOUNT) {
            discount = MAX_DISCOUNT;
        }

        int discountedSubtotal = subtotal * (100 - discount) / 100;

        // Shipping
        int shipping = COUNTRY_SHIPPING.getOrDefault(country, DEFAULT_SHIPPING);
        if (order.blackFriday() && country.equals("US")) {
            shipping += 300;
        }
        if (coupon.equals("FREESHIP") && discountedSubtotal >= 8000) {
            shipping = 0;
        }
        if (customer.freeShipThreshold() > 0 && discountedSubtotal >= customer.freeShipThreshold()) {
            shipping = 0;
        }
        if (customer.extraShip() != null) {
            shipping += customer.extraShip().apply(country);
        }

        // Tax
        int tax = COUNTRY_TAX.getOrDefault(country, 0);
        if (ct.equals("vip") && country.equals("IT")) {
            tax = 20;
        }
        if (coupon.equals("TAXFREE") && !country.equals("IT")) {
            tax = 0;
        }

        int taxCents = discountedSubtotal * tax / 100;
        int total = discountedSubtotal + shipping + taxCents;

        if (total < 0) {
            return 0;
        }
        return total;
    }

    private String safe(String value) {
        return value == null ? "" : value.trim();
    }
}
