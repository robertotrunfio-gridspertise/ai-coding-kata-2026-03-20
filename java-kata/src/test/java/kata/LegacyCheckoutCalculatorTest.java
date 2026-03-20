package kata;

import org.junit.jupiter.api.Test;

import static org.junit.jupiter.api.Assertions.assertEquals;

/**
 * Characterization tests — lock existing behavior before refactoring.
 * These tests must all pass before and after any structural change.
 */
class LegacyCheckoutCalculatorTest {

    private final LegacyCheckoutCalculator calc = new LegacyCheckoutCalculator();

    // --- README examples ---

    @Test
    void regular_IT_noCoupon_notBlackFriday() {
        // regular, subtotal 10000, country IT, no coupon, not Black Friday -> total 12900
        var order = new Order("regular", 10000, "IT", null, false);
        assertEquals(12900, calc.calculateTotalCents(order));
    }

    @Test
    void premium_DE_SAVE10_notBlackFriday() {
        // premium, subtotal 10000, country DE, coupon SAVE10, not Black Friday -> total 10420
        var order = new Order("premium", 10000, "DE", "SAVE10", false);
        assertEquals(10420, calc.calculateTotalCents(order));
    }

    @Test
    void vip_IT_VIPONLY_notBlackFriday() {
        // vip, subtotal 18000, country IT, coupon VIPONLY, not Black Friday -> total 17980
        var order = new Order("vip", 18000, "IT", "VIPONLY", false);
        assertEquals(17980, calc.calculateTotalCents(order));
    }

    // --- Discount: customer types ---

    @Test
    void vip_gets_15pct_discount() {
        // vip, 10000, DE, no coupon, not BF -> 8500 discounted + 900 ship + 1615 tax = 11015
        var order = new Order("vip", 10000, "DE", null, false);
        assertEquals(11015, calc.calculateTotalCents(order));
    }

    @Test
    void premium_below10000_gets_5pct_discount() {
        // premium, 8000, DE, no coupon -> 7600 + 900 ship + 1444 tax = 9944
        var order = new Order("premium", 8000, "DE", null, false);
        assertEquals(9944, calc.calculateTotalCents(order));
    }

    @Test
    void premium_at10000_gets_10pct_discount() {
        // premium, 10000, DE, no coupon -> 9000 + 900 ship + 1710 tax = 11610
        var order = new Order("premium", 10000, "DE", null, false);
        assertEquals(11610, calc.calculateTotalCents(order));
    }

    @Test
    void employee_gets_30pct_discount() {
        // employee, 10000, DE, no coupon -> 7000 + 900+500 ship + 1330 tax = 9730
        var order = new Order("employee", 10000, "DE", null, false);
        assertEquals(9730, calc.calculateTotalCents(order));
    }

    @Test
    void new_customer_gets_no_discount() {
        // new, 10000, DE, no coupon -> 10000 + 900 + 1900 = 12800
        var order = new Order("new", 10000, "DE", null, false);
        assertEquals(12800, calc.calculateTotalCents(order));
    }

    @Test
    void unknown_customer_gets_no_discount() {
        // unknown type -> treated same as regular (0% discount)
        var order = new Order("stranger", 10000, "DE", null, false);
        assertEquals(12800, calc.calculateTotalCents(order));
    }

    // --- Discount: coupons ---

    @Test
    void SAVE10_applies_when_subtotal_at_least_5000() {
        // regular, 5000, DE, SAVE10 -> 4500 discounted + 900 + 855 tax = 6255
        var order = new Order("regular", 5000, "DE", "SAVE10", false);
        assertEquals(6255, calc.calculateTotalCents(order));
    }

    @Test
    void SAVE10_ignored_when_subtotal_below_5000() {
        // regular, 4999, DE, SAVE10 -> no discount applied -> 4999 + 900 + 949 = 6848
        var order = new Order("regular", 4999, "DE", "SAVE10", false);
        assertEquals(6848, calc.calculateTotalCents(order));
    }

    @Test
    void VIPONLY_applies_for_vip() {
        // vip, 10000, DE, VIPONLY -> 20% discount -> 8000 + 900 + 1520 = 10420
        var order = new Order("vip", 10000, "DE", "VIPONLY", false);
        assertEquals(10420, calc.calculateTotalCents(order));
    }

    @Test
    void VIPONLY_ignored_for_non_vip() {
        // regular, 10000, DE, VIPONLY -> 0% discount -> 10000 + 900 + 1900 = 12800
        var order = new Order("regular", 10000, "DE", "VIPONLY", false);
        assertEquals(12800, calc.calculateTotalCents(order));
    }

    @Test
    void BULK_applies_when_subtotal_at_least_20000() {
        // regular, 20000, DE, BULK -> 7% discount -> 18600 + 900 + 3534 = 23034
        var order = new Order("regular", 20000, "DE", "BULK", false);
        assertEquals(23034, calc.calculateTotalCents(order));
    }

    @Test
    void BULK_ignored_when_subtotal_below_20000() {
        // regular, 19999, DE, BULK -> no discount -> 19999 + 900 + 3799 = 24698
        var order = new Order("regular", 19999, "DE", "BULK", false);
        assertEquals(24698, calc.calculateTotalCents(order));
    }

    // --- Discount: Black Friday ---

    @Test
    void blackFriday_adds_5pct_for_non_employee() {
        // regular, 10000, DE, no coupon, BF -> 5% discount -> 9500 + 900 + 1805 = 12205
        var order = new Order("regular", 10000, "DE", null, true);
        assertEquals(12205, calc.calculateTotalCents(order));
    }

    @Test
    void blackFriday_does_not_add_discount_for_employee() {
        // employee, 10000, DE, no coupon, BF -> still 30% only -> same as non-BF employee
        var order = new Order("employee", 10000, "DE", null, true);
        assertEquals(9730, calc.calculateTotalCents(order));
    }

    // --- Discount cap at 40% ---

    @Test
    void discount_capped_at_40pct() {
        // employee(30%) + BF would exceed cap — BF not applied to employee, so cap only via coupons
        // employee(30%) + SAVE10(10%) = 40% exactly -> no cap needed
        var order = new Order("employee", 10000, "DE", "SAVE10", false);
        // 40% discount -> 6000 + 900+500 ship + 1140 tax = 8540
        assertEquals(8540, calc.calculateTotalCents(order));
    }

    @Test
    void discount_not_exceeding_40pct() {
        // vip(15%) + VIPONLY(5%) + BF(5%) = 25%, no cap
        var order = new Order("vip", 10000, "DE", "VIPONLY", true);
        // 25% discount -> 7500 + 900 + 1425 = 9825
        assertEquals(9825, calc.calculateTotalCents(order));
    }

    // --- Shipping ---

    @Test
    void shipping_IT_is_700() {
        var order = new Order("regular", 10000, "IT", null, false);
        // 0% discount -> 10000, ship 700, tax 22% = 2200 -> total 12900
        assertEquals(12900, calc.calculateTotalCents(order));
    }

    @Test
    void shipping_DE_is_900() {
        var order = new Order("regular", 10000, "DE", null, false);
        assertEquals(12800, calc.calculateTotalCents(order));
    }

    @Test
    void shipping_US_is_1500() {
        var order = new Order("regular", 10000, "US", null, false);
        // 0% + 1500 ship + 7% tax = 10000 + 1500 + 700 = 12200
        assertEquals(12200, calc.calculateTotalCents(order));
    }

    @Test
    void shipping_other_country_is_2500() {
        var order = new Order("regular", 10000, "FR", null, false);
        // 0% + 2500 ship + 0% tax = 12500
        assertEquals(12500, calc.calculateTotalCents(order));
    }

    @Test
    void blackFriday_adds_300_shipping_for_US() {
        var order = new Order("regular", 10000, "US", null, true);
        // 5% BF discount -> 9500 + (1500+300) ship + 665 tax = 11965
        assertEquals(11965, calc.calculateTotalCents(order));
    }

    @Test
    void blackFriday_does_not_add_shipping_surcharge_outside_US() {
        var order = new Order("regular", 10000, "DE", null, true);
        // 5% BF -> 9500 + 900 ship + 1805 = 12205
        assertEquals(12205, calc.calculateTotalCents(order));
    }

    @Test
    void FREESHIP_applies_when_discounted_subtotal_at_least_8000() {
        // regular, 10000, DE, FREESHIP -> 0% discount -> 10000 >= 8000 -> free ship + 1900 tax = 11900
        var order = new Order("regular", 10000, "DE", "FREESHIP", false);
        assertEquals(11900, calc.calculateTotalCents(order));
    }

    @Test
    void FREESHIP_ignored_when_discounted_subtotal_below_8000() {
        // regular, 7999, DE, FREESHIP -> 7999 < 8000 -> ship 900 + 1519 = 10418
        var order = new Order("regular", 7999, "DE", "FREESHIP", false);
        assertEquals(10418, calc.calculateTotalCents(order));
    }

    @Test
    void vip_free_shipping_when_discounted_subtotal_at_least_15000() {
        // vip, 18000, DE -> 15% discount -> 15300 >= 15000 -> free ship + 2907 tax = 18207
        var order = new Order("vip", 18000, "DE", null, false);
        assertEquals(18207, calc.calculateTotalCents(order));
    }

    @Test
    void vip_not_free_shipping_when_discounted_subtotal_below_15000() {
        // vip, 17000, DE -> 15% -> 14450 < 15000 -> ship 900 + 2745 tax = 18095
        var order = new Order("vip", 17000, "DE", null, false);
        assertEquals(18095, calc.calculateTotalCents(order));
    }

    @Test
    void premium_free_shipping_when_discounted_subtotal_at_least_20000() {
        // premium, 23000, DE -> 10% -> 20700 >= 20000 -> free ship + 3933 tax = 24633
        var order = new Order("premium", 23000, "DE", null, false);
        assertEquals(24633, calc.calculateTotalCents(order));
    }

    @Test
    void employee_outside_IT_pays_extra_500_shipping() {
        // employee, 10000, DE -> 30% -> 7000 + (900+500) ship + 1330 = 9730
        var order = new Order("employee", 10000, "DE", null, false);
        assertEquals(9730, calc.calculateTotalCents(order));
    }

    @Test
    void employee_in_IT_does_not_pay_extra_shipping() {
        // employee, 10000, IT -> 30% -> 7000 + 700 ship + 1540 tax = 9240
        var order = new Order("employee", 10000, "IT", null, false);
        assertEquals(9240, calc.calculateTotalCents(order));
    }

    // --- Tax ---

    @Test
    void tax_IT_is_22pct() {
        var order = new Order("regular", 10000, "IT", null, false);
        assertEquals(12900, calc.calculateTotalCents(order)); // 10000 + 700 + 2200
    }

    @Test
    void tax_DE_is_19pct() {
        var order = new Order("regular", 10000, "DE", null, false);
        assertEquals(12800, calc.calculateTotalCents(order)); // 10000 + 900 + 1900
    }

    @Test
    void tax_US_is_7pct() {
        var order = new Order("regular", 10000, "US", null, false);
        assertEquals(12200, calc.calculateTotalCents(order)); // 10000 + 1500 + 700
    }

    @Test
    void tax_other_country_is_0pct() {
        var order = new Order("regular", 10000, "FR", null, false);
        assertEquals(12500, calc.calculateTotalCents(order)); // 10000 + 2500 + 0
    }

    @Test
    void vip_in_IT_gets_reduced_tax_20pct() {
        // vip, 10000, IT -> 15% discount -> 8500 + 700 ship (8500 < 15000) + 20% tax 1700 = 10900
        var order = new Order("vip", 10000, "IT", null, false);
        assertEquals(10900, calc.calculateTotalCents(order));
    }

    @Test
    void TAXFREE_coupon_sets_tax_to_0_outside_IT() {
        // regular, 10000, DE, TAXFREE -> 10000 + 900 + 0 = 10900
        var order = new Order("regular", 10000, "DE", "TAXFREE", false);
        assertEquals(10900, calc.calculateTotalCents(order));
    }

    @Test
    void TAXFREE_coupon_ignored_in_IT() {
        // regular, 10000, IT, TAXFREE -> tax still 22% -> 10000 + 700 + 2200 = 12900
        var order = new Order("regular", 10000, "IT", "TAXFREE", false);
        assertEquals(12900, calc.calculateTotalCents(order));
    }

    // --- Edge cases ---

    @Test
    void null_customer_type_treated_as_unknown() {
        var order = new Order(null, 10000, "DE", null, false);
        assertEquals(12800, calc.calculateTotalCents(order));
    }

    @Test
    void null_country_treated_as_other() {
        // null country -> 2500 ship, 0 tax
        var order = new Order("regular", 10000, null, null, false);
        assertEquals(12500, calc.calculateTotalCents(order));
    }

    @Test
    void null_coupon_has_no_effect() {
        var order = new Order("regular", 10000, "DE", null, false);
        assertEquals(12800, calc.calculateTotalCents(order));
    }

    @Test
    void total_cannot_be_negative() {
        // Extreme discount capped at 40%, but subtotal 0 -> total should be 0+shipping+tax, not negative
        var order = new Order("employee", 0, "DE", null, false);
        assertEquals(1400, calc.calculateTotalCents(order)); // 0 + 900+500 + 0
    }

    // --- Partner customer type ---

    @Test
    void partner_gets_12pct_base_discount() {
        // partner, 10000, DE, no coupon -> 12% -> 8800 + 900 ship + 1672 tax = 11372
        var order = new Order("partner", 10000, "DE", null, false);
        assertEquals(11372, calc.calculateTotalCents(order));
    }

    @Test
    void partner_free_shipping_when_discounted_subtotal_at_least_15000() {
        // partner, 18000, DE -> 12% -> 15840 >= 15000 -> free ship + 3009 tax = 18849
        var order = new Order("partner", 18000, "DE", null, false);
        assertEquals(18849, calc.calculateTotalCents(order));
    }

    @Test
    void partner_not_free_shipping_when_discounted_subtotal_below_15000() {
        // partner, 17000, DE -> 12% -> 14960 < 15000 -> ship 900 + 2842 tax = 18702
        var order = new Order("partner", 17000, "DE", null, false);
        assertEquals(18702, calc.calculateTotalCents(order));
    }

    @Test
    void partner_PARTNER5_adds_5pct_when_subtotal_at_least_12000() {
        // partner, 12000, DE, PARTNER5, not BF -> 12+5=17% -> 9960 + 900 ship + 1892 tax = 12752
        var order = new Order("partner", 12000, "DE", "PARTNER5", false);
        assertEquals(12752, calc.calculateTotalCents(order));
    }

    @Test
    void partner_PARTNER5_ignored_when_subtotal_below_12000() {
        // partner, 11999, DE, PARTNER5 -> only 12% base -> 10559 + 900 + 2006 = 13465
        var order = new Order("partner", 11999, "DE", "PARTNER5", false);
        assertEquals(13465, calc.calculateTotalCents(order));
    }

    @Test
    void partner_blackFriday_discount_is_3pct_not_5pct() {
        // partner, 10000, DE, no coupon, BF -> 12+3(BF)=15% -> 8500 + 900 + 1615 = 11015
        var order = new Order("partner", 10000, "DE", null, true);
        assertEquals(11015, calc.calculateTotalCents(order));
    }

    @Test
    void partner_PARTNER5_on_blackFriday_stacks_coupon_5pct_and_BF_3pct() {
        // partner, 12000, DE, PARTNER5, BF -> 12+5(coupon)+3(BF)=20% -> 9600 + 900 + 1824 = 12324
        var order = new Order("partner", 12000, "DE", "PARTNER5", true);
        assertEquals(12324, calc.calculateTotalCents(order));
    }

    @Test
    void PARTNER5_ignored_for_non_partner_customer() {
        // regular, 12000, DE, PARTNER5 -> 0% discount -> 12000 + 900 + 2280 = 15180
        var order = new Order("regular", 12000, "DE", "PARTNER5", false);
        assertEquals(15180, calc.calculateTotalCents(order));
    }

    // --- Invalid / boundary inputs ---

    @Test
    void negative_subtotal_is_clamped_to_zero() {
        // discountedSubtotal=-1000, ship=900, tax=-190 -> total=-290 -> clamped to 0
        var order = new Order("regular", -1000, "DE", null, false);
        assertEquals(0, calc.calculateTotalCents(order));
    }

    @Test
    void zero_subtotal_returns_only_shipping_and_tax() {
        // 0 discounted + 900 ship + 0 tax = 900
        var order = new Order("regular", 0, "DE", null, false);
        assertEquals(900, calc.calculateTotalCents(order));
    }

    @Test
    void empty_string_customer_type_treated_as_unknown() {
        // safe() trims -> "" -> no match -> 0% discount; same as regular
        var order = new Order("", 10000, "DE", null, false);
        assertEquals(12800, calc.calculateTotalCents(order));
    }

    @Test
    void whitespace_customer_type_treated_as_unknown() {
        // safe() trims "  " -> "" -> 0% discount
        var order = new Order("   ", 10000, "DE", null, false);
        assertEquals(12800, calc.calculateTotalCents(order));
    }

    @Test
    void empty_string_country_treated_as_other() {
        // "" -> no match -> 2500 ship, 0 tax
        var order = new Order("regular", 10000, "", null, false);
        assertEquals(12500, calc.calculateTotalCents(order));
    }

    @Test
    void empty_string_coupon_has_no_effect() {
        // safe() trims -> "" -> no coupon matched
        var order = new Order("regular", 10000, "DE", "", false);
        assertEquals(12800, calc.calculateTotalCents(order));
    }

    @Test
    void unknown_coupon_has_no_effect() {
        // unrecognised coupon code -> ignored
        var order = new Order("regular", 10000, "DE", "NOTACOUPON", false);
        assertEquals(12800, calc.calculateTotalCents(order));
    }

    @Test
    void customer_type_is_case_sensitive() {
        // "VIP" != "vip" -> treated as unknown, 0% discount
        var order = new Order("VIP", 10000, "DE", null, false);
        assertEquals(12800, calc.calculateTotalCents(order));
    }

    @Test
    void country_is_case_sensitive() {
        // "it" != "IT" -> treated as other country -> 2500 ship, 0 tax
        var order = new Order("regular", 10000, "it", null, false);
        assertEquals(12500, calc.calculateTotalCents(order));
    }
}
