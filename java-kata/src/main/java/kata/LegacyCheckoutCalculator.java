package kata;

public class LegacyCheckoutCalculator {

    private static final int MAX_DISCOUNT_PERCENT = 40;

    public int calculateTotalCents(Order order) {
        String customerType = safe(order.customerType());
        String country = safe(order.country());
        String coupon = safe(order.couponCode());
        int subtotal = order.subtotalCents();

        int discountPercent = computeDiscount(customerType, subtotal, coupon, order.blackFriday());
        int discountedSubtotal = subtotal * (100 - discountPercent) / 100;

        int shippingCents = computeShipping(customerType, country, coupon, discountedSubtotal, order.blackFriday());
        int taxCents = computeTax(customerType, country, coupon, discountedSubtotal);

        int total = discountedSubtotal + shippingCents + taxCents;
        return Math.max(total, 0);
    }

    private int computeDiscount(String customerType, int subtotal, String coupon, boolean blackFriday) {
        CustomerRules rules = CustomerRules.forType(customerType);

        int discount = baseDiscount(customerType, subtotal, rules);
        discount += couponDiscount(customerType, subtotal, coupon, blackFriday);
        discount += blackFridayDiscount(rules, blackFriday);

        return Math.min(discount, MAX_DISCOUNT_PERCENT);
    }

    private int baseDiscount(String customerType, int subtotal, CustomerRules rules) {
        // premium has a tiered base discount
        if (customerType.equals("premium")) {
            return subtotal >= 10000 ? 10 : 5;
        }
        return rules.baseDiscountPercent();
    }

    private int couponDiscount(String customerType, int subtotal, String coupon, boolean blackFriday) {
        return switch (coupon) {
            case "SAVE10"   -> subtotal >= 5000 ? 10 : 0;
            case "VIPONLY"  -> customerType.equals("vip") ? 5 : 0;
            case "BULK"     -> subtotal >= 20000 ? 7 : 0;
            case "PARTNER5" -> partnerCouponDiscount(customerType, subtotal, blackFriday);
            default         -> 0;
        };
    }

    private int partnerCouponDiscount(String customerType, int subtotal, boolean blackFriday) {
        if (!customerType.equals("partner") || subtotal < 12000) return 0;
        return 5;
    }

    private int blackFridayDiscount(CustomerRules rules, boolean blackFriday) {
        if (!blackFriday) return 0;
        return rules.blackFridayDiscountPercent();
    }

    private int computeShipping(String customerType, String country, String coupon,
                                 int discountedSubtotal, boolean blackFriday) {
        int shipping = baseShipping(country);

        if (blackFriday && country.equals("US")) {
            shipping += 300;
        }

        if (customerType.equals("employee") && !country.equals("IT")) {
            shipping += 500;
        }

        CustomerRules rules = CustomerRules.forType(customerType);
        if (rules.freeShippingThresholdCents() > 0
                && discountedSubtotal >= rules.freeShippingThresholdCents()) {
            return 0;
        }

        if (coupon.equals("FREESHIP") && discountedSubtotal >= 8000) {
            return 0;
        }

        return shipping;
    }

    private int baseShipping(String country) {
        return switch (country) {
            case "IT" -> 700;
            case "DE" -> 900;
            case "US" -> 1500;
            default   -> 2500;
        };
    }

    private int computeTax(String customerType, String country, String coupon, int discountedSubtotal) {
        int taxPercent = baseTax(country);

        if (customerType.equals("vip") && country.equals("IT")) {
            taxPercent = 20;
        }

        if (coupon.equals("TAXFREE") && !country.equals("IT")) {
            taxPercent = 0;
        }

        return discountedSubtotal * taxPercent / 100;
    }

    private int baseTax(String country) {
        return switch (country) {
            case "IT" -> 22;
            case "DE" -> 19;
            case "US" -> 7;
            default   -> 0;
        };
    }

    private String safe(String value) {
        return value == null ? "" : value.trim();
    }
}
