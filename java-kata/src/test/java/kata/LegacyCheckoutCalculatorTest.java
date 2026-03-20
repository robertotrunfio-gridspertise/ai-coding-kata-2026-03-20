package kata;

import org.junit.jupiter.api.Test;
import static org.junit.jupiter.api.Assertions.assertEquals;

class LegacyCheckoutCalculatorTest {

    private final LegacyCheckoutCalculator calc = new LegacyCheckoutCalculator();

    // README examples — behavior preservation
    @Test void regular_IT_no_coupon() {
        assertEquals(12900, calc.calculateTotalCents(new Order("regular", 10000, "IT", "", false)));
    }
    @Test void premium_DE_SAVE10() {
        assertEquals(10420, calc.calculateTotalCents(new Order("premium", 10000, "DE", "SAVE10", false)));
    }
    @Test void vip_IT_VIPONLY() {
        assertEquals(17980, calc.calculateTotalCents(new Order("vip", 18000, "IT", "VIPONLY", false)));
    }

    // Customer base discounts
    @Test void employee_DE_shipping_surcharge() {
        assertEquals(9730, calc.calculateTotalCents(new Order("employee", 10000, "DE", "", false)));
    }
    @Test void employee_IT_no_surcharge() {
        assertEquals(9240, calc.calculateTotalCents(new Order("employee", 10000, "IT", "", false)));
    }
    @Test void premium_low_subtotal_5pct_discount() {
        assertEquals(6552, calc.calculateTotalCents(new Order("premium", 5000, "DE", "", false)));
    }

    // Free shipping thresholds
    @Test void vip_IT_free_shipping() {
        assertEquals(20400, calc.calculateTotalCents(new Order("vip", 20000, "IT", "", false)));
    }
    @Test void vip_IT_below_free_shipping() {
        assertEquals(10900, calc.calculateTotalCents(new Order("vip", 10000, "IT", "", false)));
    }
    @Test void premium_IT_free_shipping() {
        assertEquals(27450, calc.calculateTotalCents(new Order("premium", 25000, "IT", "", false)));
    }

    // Coupon rules
    @Test void SAVE10_below_threshold_not_applied() {
        assertEquals(5422, calc.calculateTotalCents(new Order("premium", 4000, "DE", "SAVE10", false)));
    }
    @Test void BULK_large_order() {
        assertEquals(28567, calc.calculateTotalCents(new Order("regular", 25000, "DE", "BULK", false)));
    }
    @Test void VIPONLY_on_non_vip_no_effect() {
        assertEquals(11680, calc.calculateTotalCents(new Order("premium", 10000, "IT", "VIPONLY", false)));
    }
    @Test void FREESHIP_coupon() {
        assertEquals(12200, calc.calculateTotalCents(new Order("regular", 10000, "IT", "FREESHIP", false)));
    }
    @Test void TAXFREE_DE() {
        assertEquals(10900, calc.calculateTotalCents(new Order("regular", 10000, "DE", "TAXFREE", false)));
    }
    @Test void TAXFREE_IT_not_applied() {
        assertEquals(12900, calc.calculateTotalCents(new Order("regular", 10000, "IT", "TAXFREE", false)));
    }

    // Black Friday
    @Test void regular_BF_US_shipping_surcharge() {
        assertEquals(11965, calc.calculateTotalCents(new Order("regular", 10000, "US", "", true)));
    }
    @Test void employee_BF_no_bonus() {
        assertEquals(9240, calc.calculateTotalCents(new Order("employee", 10000, "IT", "", true)));
    }
    @Test void vip_BF_IT() {
        assertEquals(10300, calc.calculateTotalCents(new Order("vip", 10000, "IT", "", true)));
    }

    // Discount cap
    @Test void employee_SAVE10_hits_cap() {
        assertEquals(8020, calc.calculateTotalCents(new Order("employee", 10000, "IT", "SAVE10", false)));
    }

    // Edge cases
    @Test void unknown_customer_type() {
        assertEquals(12900, calc.calculateTotalCents(new Order("unknown", 10000, "IT", "", false)));
    }
    @Test void unknown_country_default_shipping() {
        assertEquals(12500, calc.calculateTotalCents(new Order("regular", 10000, "FR", "", false)));
    }
    @Test void null_fields_handled() {
        assertEquals(12500, calc.calculateTotalCents(new Order(null, 10000, null, null, false)));
    }

    // Partner — new requirement
    @Test void partner_IT_below_free_ship_threshold() {
        assertEquals(16804, calc.calculateTotalCents(new Order("partner", 15000, "IT", "", false)));
    }
    @Test void partner_IT_free_shipping() {
        assertEquals(21472, calc.calculateTotalCents(new Order("partner", 20000, "IT", "", false)));
    }
    @Test void partner_IT_PARTNER5_coupon() {
        assertEquals(15889, calc.calculateTotalCents(new Order("partner", 15000, "IT", "PARTNER5", false)));
    }
    @Test void partner_PARTNER5_below_subtotal_threshold() {
        assertEquals(11436, calc.calculateTotalCents(new Order("partner", 10000, "IT", "PARTNER5", false)));
    }
    @Test void partner_BF_gets_plus3_not_plus5() {
        assertEquals(16255, calc.calculateTotalCents(new Order("partner", 15000, "IT", "", true)));
    }
    @Test void PARTNER5_on_non_partner_no_discount() {
        assertEquals(19000, calc.calculateTotalCents(new Order("regular", 15000, "IT", "PARTNER5", false)));
    }
}
