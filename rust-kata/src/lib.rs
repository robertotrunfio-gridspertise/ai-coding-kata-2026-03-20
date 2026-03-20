#[derive(Debug, Clone)]
pub struct Order {
    pub customer_type: String,
    pub subtotal_cents: i32,
    pub country: String,
    pub coupon_code: String,
    pub black_friday: bool,
}

pub fn calculate_total_cents(order: &Order) -> i32 {
    let subtotal = order.subtotal_cents;
    let customer_type = safe(&order.customer_type);
    let country = safe(&order.country);
    let coupon = safe(&order.coupon_code);

    let mut discount_percent = 0;

    if customer_type == "vip" {
        discount_percent += 15;
    } else if customer_type == "premium" {
        if subtotal >= 10000 {
            discount_percent += 10;
        } else {
            discount_percent += 5;
        }
    } else if customer_type == "employee" {
        discount_percent += 30;
    } else if customer_type == "partner" {
        discount_percent += 12;
    } else if customer_type == "regular" || customer_type == "new" {
        discount_percent += 0;
    } else {
        discount_percent += 0;
    }

    if coupon == "SAVE10" {
        if subtotal >= 5000 {
            discount_percent += 10;
        }
    } else if coupon == "VIPONLY" {
        if customer_type == "vip" {
            discount_percent += 5;
        }
    } else if coupon == "BULK" {
        if subtotal >= 20000 {
            discount_percent += 7;
        }
    } else if coupon == "PARTNER5" {
        if customer_type == "partner" && subtotal >= 12000 {
            discount_percent += 5;
        }
    }

    if order.black_friday {
        if customer_type == "partner" {
            discount_percent += 3;
        } else if customer_type != "employee" {
            discount_percent += 5;
        }
    }

    if discount_percent > 40 {
        discount_percent = 40;
    }

    let discounted_subtotal = subtotal * (100 - discount_percent) / 100;

    let mut shipping_cents = if country == "IT" {
        700
    } else if country == "DE" {
        900
    } else if country == "US" {
        1500
    } else {
        2500
    };

    if order.black_friday && country == "US" {
        shipping_cents += 300;
    }

    if coupon == "FREESHIP" && discounted_subtotal >= 8000 {
        shipping_cents = 0;
    }

    if customer_type == "vip" && discounted_subtotal >= 15000 {
        shipping_cents = 0;
    }

    if customer_type == "premium" && discounted_subtotal >= 20000 {
        shipping_cents = 0;
    }

    if customer_type == "partner" && discounted_subtotal >= 15000 {
        shipping_cents = 0;
    }

    if customer_type == "employee" && country != "IT" {
        shipping_cents += 500;
    }

    let mut tax_percent = if country == "IT" {
        22
    } else if country == "DE" {
        19
    } else if country == "US" {
        7
    } else {
        0
    };

    if customer_type == "vip" && country == "IT" {
        tax_percent = 20;
    }

    if coupon == "TAXFREE" && country != "IT" {
        tax_percent = 0;
    }

    let tax_cents = discounted_subtotal * tax_percent / 100;
    let total = discounted_subtotal + shipping_cents + tax_cents;

    if total < 0 { 0 } else { total }
}

fn safe(value: &str) -> String {
    value.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper to build an Order concisely.
    fn order(customer_type: &str, subtotal_cents: i32, country: &str, coupon_code: &str, black_friday: bool) -> Order {
        Order {
            customer_type: customer_type.to_string(),
            subtotal_cents,
            country: country.to_string(),
            coupon_code: coupon_code.to_string(),
            black_friday,
        }
    }

    // =========================================================================
    // 1. README verification examples
    // =========================================================================

    #[test]
    fn readme_example_regular_10000_it_no_coupon() {
        // disc=0%, ds=10000, ship=700, tax=2200 → 12900
        assert_eq!(calculate_total_cents(&order("regular", 10000, "IT", "", false)), 12900);
    }

    #[test]
    fn readme_example_premium_10000_de_save10() {
        // disc=10%+10%=20%, ds=8000, ship=900, tax=1520 → 10420
        assert_eq!(calculate_total_cents(&order("premium", 10000, "DE", "SAVE10", false)), 10420);
    }

    #[test]
    fn readme_example_vip_18000_it_viponly() {
        // disc=15%+5%=20%, ds=14400, ship=700, tax=14400*20/100=2880 → 17980
        assert_eq!(calculate_total_cents(&order("vip", 18000, "IT", "VIPONLY", false)), 17980);
    }

    // =========================================================================
    // 2. Customer type base discounts — happy path per country
    // =========================================================================

    // --- regular (discount 0%) ---
    #[test]
    fn regular_it() {
        // ds=10000, ship=700, tax=2200 → 12900
        assert_eq!(calculate_total_cents(&order("regular", 10000, "IT", "", false)), 12900);
    }

    #[test]
    fn regular_de() {
        // ds=10000, ship=900, tax=1900 → 12800
        assert_eq!(calculate_total_cents(&order("regular", 10000, "DE", "", false)), 12800);
    }

    #[test]
    fn regular_us() {
        // ds=10000, ship=1500, tax=700 → 12200
        assert_eq!(calculate_total_cents(&order("regular", 10000, "US", "", false)), 12200);
    }

    #[test]
    fn regular_other_country() {
        // ds=10000, ship=2500, tax=0 → 12500
        assert_eq!(calculate_total_cents(&order("regular", 10000, "FR", "", false)), 12500);
    }

    // --- new (discount 0%, same as regular) ---
    #[test]
    fn new_it() {
        assert_eq!(calculate_total_cents(&order("new", 10000, "IT", "", false)), 12900);
    }

    #[test]
    fn new_de() {
        assert_eq!(calculate_total_cents(&order("new", 10000, "DE", "", false)), 12800);
    }

    // --- premium (5% below 10000, 10% at/above 10000) ---
    #[test]
    fn premium_5pct_it() {
        // subtotal=5000<10000 → disc=5%, ds=4750, ship=700, tax=1045 → 6495
        assert_eq!(calculate_total_cents(&order("premium", 5000, "IT", "", false)), 6495);
    }

    #[test]
    fn premium_10pct_it() {
        // subtotal=10000>=10000 → disc=10%, ds=9000, ship=700, tax=1980 → 11680
        assert_eq!(calculate_total_cents(&order("premium", 10000, "IT", "", false)), 11680);
    }

    #[test]
    fn premium_10pct_de() {
        // disc=10%, ds=9000, ship=900, tax=1710 → 11610
        assert_eq!(calculate_total_cents(&order("premium", 10000, "DE", "", false)), 11610);
    }

    #[test]
    fn premium_10pct_us() {
        // disc=10%, ds=9000, ship=1500, tax=630 → 11130
        assert_eq!(calculate_total_cents(&order("premium", 10000, "US", "", false)), 11130);
    }

    // --- vip (discount 15%) ---
    #[test]
    fn vip_it() {
        // disc=15%, ds=8500, ship=700, tax=8500*20/100=1700 (vip+IT→20%) → 10900
        assert_eq!(calculate_total_cents(&order("vip", 10000, "IT", "", false)), 10900);
    }

    #[test]
    fn vip_de() {
        // disc=15%, ds=8500, ship=900, tax=8500*19/100=1615 → 11015
        assert_eq!(calculate_total_cents(&order("vip", 10000, "DE", "", false)), 11015);
    }

    #[test]
    fn vip_us() {
        // disc=15%, ds=8500, ship=1500, tax=8500*7/100=595 → 10595
        assert_eq!(calculate_total_cents(&order("vip", 10000, "US", "", false)), 10595);
    }

    #[test]
    fn vip_other_country() {
        // disc=15%, ds=8500, ship=2500, tax=0 → 11000
        assert_eq!(calculate_total_cents(&order("vip", 10000, "FR", "", false)), 11000);
    }

    // --- employee (discount 30%) ---
    #[test]
    fn employee_it() {
        // disc=30%, ds=7000, ship=700 (IT: no surcharge), tax=1540 → 9240
        assert_eq!(calculate_total_cents(&order("employee", 10000, "IT", "", false)), 9240);
    }

    #[test]
    fn employee_de() {
        // disc=30%, ds=7000, ship=900+500=1400, tax=1330 → 9730
        assert_eq!(calculate_total_cents(&order("employee", 10000, "DE", "", false)), 9730);
    }

    #[test]
    fn employee_us() {
        // disc=30%, ds=7000, ship=1500+500=2000, tax=490 → 9490
        assert_eq!(calculate_total_cents(&order("employee", 10000, "US", "", false)), 9490);
    }

    #[test]
    fn employee_other_country() {
        // disc=30%, ds=7000, ship=2500+500=3000, tax=0 → 10000
        assert_eq!(calculate_total_cents(&order("employee", 10000, "FR", "", false)), 10000);
    }

    // =========================================================================
    // 3. Coupon effects
    // =========================================================================

    // --- SAVE10: +10% when subtotal >= 5000 ---
    #[test]
    fn save10_qualifies() {
        // regular, 5000, SAVE10: disc=10%, ds=4500, ship=700, tax=990 → 6190
        assert_eq!(calculate_total_cents(&order("regular", 5000, "IT", "SAVE10", false)), 6190);
    }

    #[test]
    fn save10_does_not_qualify() {
        // regular, 4999, SAVE10: disc=0%, ds=4999, ship=700, tax=1099 → 6798
        assert_eq!(calculate_total_cents(&order("regular", 4999, "IT", "SAVE10", false)), 6798);
    }

    #[test]
    fn save10_with_vip() {
        // vip, 10000, SAVE10: disc=15+10=25%, ds=7500, ship=700, tax=7500*20/100=1500 → 9700
        assert_eq!(calculate_total_cents(&order("vip", 10000, "IT", "SAVE10", false)), 9700);
    }

    #[test]
    fn save10_with_employee() {
        // employee, 5000, IT, SAVE10: disc=30+10=40%, ds=3000, ship=700, tax=660 → 4360
        assert_eq!(calculate_total_cents(&order("employee", 5000, "IT", "SAVE10", false)), 4360);
    }

    // --- VIPONLY: +5% only for vip ---
    #[test]
    fn viponly_for_vip() {
        // vip, 10000, IT, VIPONLY: disc=15+5=20%, ds=8000, ship=700, tax=8000*20/100=1600 → 10300
        assert_eq!(calculate_total_cents(&order("vip", 10000, "IT", "VIPONLY", false)), 10300);
    }

    #[test]
    fn viponly_for_regular() {
        // regular, 10000, IT, VIPONLY: disc=0%, ds=10000, ship=700, tax=2200 → 12900
        assert_eq!(calculate_total_cents(&order("regular", 10000, "IT", "VIPONLY", false)), 12900);
    }

    #[test]
    fn viponly_for_premium() {
        // premium, 10000, IT, VIPONLY: disc=10%, ds=9000, ship=700, tax=1980 → 11680
        assert_eq!(calculate_total_cents(&order("premium", 10000, "IT", "VIPONLY", false)), 11680);
    }

    #[test]
    fn viponly_for_employee() {
        // employee, 10000, IT, VIPONLY: disc=30%, ds=7000, ship=700, tax=1540 → 9240
        assert_eq!(calculate_total_cents(&order("employee", 10000, "IT", "VIPONLY", false)), 9240);
    }

    // --- BULK: +7% when subtotal >= 20000 ---
    #[test]
    fn bulk_qualifies() {
        // regular, 20000, IT, BULK: disc=7%, ds=18600, ship=700, tax=4092 → 23392
        assert_eq!(calculate_total_cents(&order("regular", 20000, "IT", "BULK", false)), 23392);
    }

    #[test]
    fn bulk_does_not_qualify() {
        // regular, 19999, IT, BULK: disc=0%, ds=19999, ship=700, tax=4399 → 25098
        assert_eq!(calculate_total_cents(&order("regular", 19999, "IT", "BULK", false)), 25098);
    }

    #[test]
    fn bulk_with_vip() {
        // vip, 20000, DE, BULK: disc=15+7=22%, ds=15600, ship=0 (vip>=15000), tax=2964 → 18564
        assert_eq!(calculate_total_cents(&order("vip", 20000, "DE", "BULK", false)), 18564);
    }

    // --- FREESHIP: sets shipping=0 when discounted subtotal >= 8000 ---
    #[test]
    fn freeship_qualifies() {
        // regular, 10000, IT, FREESHIP: ds=10000>=8000→ship=0, tax=2200 → 12200
        assert_eq!(calculate_total_cents(&order("regular", 10000, "IT", "FREESHIP", false)), 12200);
    }

    #[test]
    fn freeship_does_not_qualify() {
        // regular, 7999, IT, FREESHIP: ds=7999<8000→ship=700, tax=1759 → 10458
        assert_eq!(calculate_total_cents(&order("regular", 7999, "IT", "FREESHIP", false)), 10458);
    }

    #[test]
    fn freeship_at_exact_threshold() {
        // regular, 8000, IT, FREESHIP: ds=8000>=8000→ship=0, tax=1760 → 9760
        assert_eq!(calculate_total_cents(&order("regular", 8000, "IT", "FREESHIP", false)), 9760);
    }

    #[test]
    fn freeship_then_employee_surcharge() {
        // employee, 20000, DE, FREESHIP: disc=30%, ds=14000>=8000→ship=0, employee+DE→+500→ship=500
        // tax=14000*19/100=2660, total=17160
        assert_eq!(calculate_total_cents(&order("employee", 20000, "DE", "FREESHIP", false)), 17160);
    }

    #[test]
    fn freeship_employee_us_bf() {
        // employee, 20000, US, FREESHIP, BF: disc=30%, ds=14000
        // ship=1500+300(BF+US)=1800, FREESHIP→0, employee+US→+500=500
        // tax=14000*7/100=980, total=15480
        assert_eq!(calculate_total_cents(&order("employee", 20000, "US", "FREESHIP", true)), 15480);
    }

    // --- TAXFREE: sets tax=0 when country != IT ---
    #[test]
    fn taxfree_de() {
        // regular, 10000, DE, TAXFREE: disc=0%, ds=10000, ship=900, tax=0 → 10900
        assert_eq!(calculate_total_cents(&order("regular", 10000, "DE", "TAXFREE", false)), 10900);
    }

    #[test]
    fn taxfree_us() {
        // regular, 10000, US, TAXFREE: disc=0%, ds=10000, ship=1500, tax=0 → 11500
        assert_eq!(calculate_total_cents(&order("regular", 10000, "US", "TAXFREE", false)), 11500);
    }

    #[test]
    fn taxfree_blocked_in_it() {
        // regular, 10000, IT, TAXFREE: TAXFREE doesn't work in IT, tax=2200 → 12900
        assert_eq!(calculate_total_cents(&order("regular", 10000, "IT", "TAXFREE", false)), 12900);
    }

    #[test]
    fn taxfree_vip_it_still_pays_tax() {
        // vip, 10000, IT, TAXFREE: TAXFREE blocked in IT, tax=8500*20/100=1700 → 10900
        assert_eq!(calculate_total_cents(&order("vip", 10000, "IT", "TAXFREE", false)), 10900);
    }

    #[test]
    fn taxfree_other_country() {
        // regular, 10000, FR, TAXFREE: tax already 0% for other countries → 12500
        assert_eq!(calculate_total_cents(&order("regular", 10000, "FR", "TAXFREE", false)), 12500);
    }

    // --- Unknown coupon (no effect) ---
    #[test]
    fn unknown_coupon_ignored() {
        assert_eq!(calculate_total_cents(&order("regular", 10000, "IT", "INVALID", false)), 12900);
    }

    // =========================================================================
    // 4. Black Friday effects
    // =========================================================================

    #[test]
    fn bf_regular_it() {
        // disc=0+5=5%, ds=9500, ship=700, tax=9500*22/100=2090 → 12290
        assert_eq!(calculate_total_cents(&order("regular", 10000, "IT", "", true)), 12290);
    }

    #[test]
    fn bf_new_de() {
        // disc=0+5=5%, ds=9500, ship=900, tax=9500*19/100=1805 → 12205
        assert_eq!(calculate_total_cents(&order("new", 10000, "DE", "", true)), 12205);
    }

    #[test]
    fn bf_premium_it() {
        // disc=10+5=15%, ds=8500, ship=700, tax=8500*22/100=1870 → 11070
        assert_eq!(calculate_total_cents(&order("premium", 10000, "IT", "", true)), 11070);
    }

    #[test]
    fn bf_vip_it() {
        // disc=15+5=20%, ds=8000, ship=700, tax=8000*20/100=1600 → 10300
        assert_eq!(calculate_total_cents(&order("vip", 10000, "IT", "", true)), 10300);
    }

    #[test]
    fn bf_employee_no_extra_discount() {
        // employee on BF: no extra discount. disc=30%, ds=7000, ship=700, tax=1540 → 9240
        assert_eq!(calculate_total_cents(&order("employee", 10000, "IT", "", true)), 9240);
    }

    #[test]
    fn bf_us_shipping_surcharge() {
        // regular, 10000, US, BF: disc=5%, ds=9500, ship=1500+300=1800, tax=665 → 11965
        assert_eq!(calculate_total_cents(&order("regular", 10000, "US", "", true)), 11965);
    }

    #[test]
    fn bf_employee_us_shipping_surcharge() {
        // employee, 10000, US, BF: disc=30%, ds=7000, ship=1500+300+500=2300, tax=490 → 9790
        assert_eq!(calculate_total_cents(&order("employee", 10000, "US", "", true)), 9790);
    }

    #[test]
    fn bf_non_us_no_shipping_surcharge() {
        // regular, 10000, DE, BF: disc=5%, ds=9500, ship=900 (no BF surcharge outside US)
        // tax=9500*19/100=1805 → 12205
        assert_eq!(calculate_total_cents(&order("regular", 10000, "DE", "", true)), 12205);
    }

    #[test]
    fn bf_with_save10() {
        // regular, 10000, IT, SAVE10, BF: disc=0+10+5=15%, ds=8500, ship=700
        // tax=8500*22/100=1870 → 11070
        assert_eq!(calculate_total_cents(&order("regular", 10000, "IT", "SAVE10", true)), 11070);
    }

    #[test]
    fn bf_vip_viponly() {
        // vip, 10000, IT, VIPONLY, BF: disc=15+5+5=25%, ds=7500, ship=700
        // tax=7500*20/100=1500 → 9700
        assert_eq!(calculate_total_cents(&order("vip", 10000, "IT", "VIPONLY", true)), 9700);
    }

    #[test]
    fn bf_freeship_us() {
        // regular, 10000, US, FREESHIP, BF: disc=5%, ds=9500
        // ship=1500+300=1800, FREESHIP+9500>=8000→0, tax=665 → 10165
        assert_eq!(calculate_total_cents(&order("regular", 10000, "US", "FREESHIP", true)), 10165);
    }

    // =========================================================================
    // 5. Discount cap at 40%
    // =========================================================================

    #[test]
    fn discount_cap_exactly_40() {
        // employee(30%) + SAVE10(10%) = 40% — at cap, not exceeded
        // employee, 10000, IT, SAVE10: ds=6000, ship=700, tax=6000*22/100=1320 → 8020
        assert_eq!(calculate_total_cents(&order("employee", 10000, "IT", "SAVE10", false)), 8020);
    }

    #[test]
    fn discount_cap_employee_bulk() {
        // employee(30%) + BULK(7%) = 37% — below cap
        // employee, 20000, IT, BULK: ds=20000*63/100=12600, ship=700
        // tax=12600*22/100=2772 → 16072
        assert_eq!(calculate_total_cents(&order("employee", 20000, "IT", "BULK", false)), 16072);
    }

    #[test]
    fn discount_cap_employee_save10_bf() {
        // employee(30%) + SAVE10(10%) + BF(0% for employee) = 40%
        // employee, 10000, DE, SAVE10, BF: ds=6000, ship=900+500=1400
        // tax=6000*19/100=1140 → 8540
        assert_eq!(calculate_total_cents(&order("employee", 10000, "DE", "SAVE10", true)), 8540);
    }

    // =========================================================================
    // 6. Shipping — free shipping thresholds
    // =========================================================================

    #[test]
    fn vip_free_shipping_at_threshold() {
        // vip, 17648, DE: disc=15%, ds=17648*85/100=15000→ship=0
        // tax=15000*19/100=2850 → 17850
        assert_eq!(calculate_total_cents(&order("vip", 17648, "DE", "", false)), 17850);
    }

    #[test]
    fn vip_no_free_shipping_below_threshold() {
        // vip, 17647, DE: disc=15%, ds=17647*85/100=14999→ship=900
        // tax=14999*19/100=2849 → 18748
        assert_eq!(calculate_total_cents(&order("vip", 17647, "DE", "", false)), 18748);
    }

    #[test]
    fn vip_free_shipping_it() {
        // vip, 18000, IT: disc=15%, ds=15300>=15000→ship=0
        // tax=15300*20/100=3060 → 18360
        assert_eq!(calculate_total_cents(&order("vip", 18000, "IT", "", false)), 18360);
    }

    #[test]
    fn premium_free_shipping_at_threshold() {
        // premium, 22223, IT: disc=10%, ds=22223*90/100=20000→ship=0
        // tax=20000*22/100=4400 → 24400
        assert_eq!(calculate_total_cents(&order("premium", 22223, "IT", "", false)), 24400);
    }

    #[test]
    fn premium_no_free_shipping_below_threshold() {
        // premium, 22222, IT: disc=10%, ds=22222*90/100=19999→ship=700
        // tax=19999*22/100=4399 → 25098
        assert_eq!(calculate_total_cents(&order("premium", 22222, "IT", "", false)), 25098);
    }

    // =========================================================================
    // 7. Tax — VIP IT override, TAXFREE
    // =========================================================================

    #[test]
    fn vip_it_tax_20_not_22() {
        // vip+IT → 20% tax (not the standard 22%)
        // vip, 10000, IT: ds=8500, tax=8500*20/100=1700, ship=700 → 10900
        assert_eq!(calculate_total_cents(&order("vip", 10000, "IT", "", false)), 10900);
    }

    #[test]
    fn non_vip_it_tax_22() {
        // regular, 10000, IT: tax=2200
        assert_eq!(calculate_total_cents(&order("regular", 10000, "IT", "", false)), 12900);
    }

    #[test]
    fn taxfree_vip_de() {
        // vip, 10000, DE, TAXFREE: disc=15%, ds=8500, ship=900, tax=0 → 9400
        assert_eq!(calculate_total_cents(&order("vip", 10000, "DE", "TAXFREE", false)), 9400);
    }

    // =========================================================================
    // 8. Boundary conditions — exact thresholds
    // =========================================================================

    #[test]
    fn premium_threshold_at_9999() {
        // subtotal=9999 <10000 → 5% discount
        // ds=9999*95/100=9499, ship=700, tax=9499*22/100=2089 → 12288
        assert_eq!(calculate_total_cents(&order("premium", 9999, "IT", "", false)), 12288);
    }

    #[test]
    fn premium_threshold_at_10000() {
        // subtotal=10000>=10000 → 10% discount
        // ds=9000, ship=700, tax=1980 → 11680
        assert_eq!(calculate_total_cents(&order("premium", 10000, "IT", "", false)), 11680);
    }

    #[test]
    fn save10_threshold_at_4999() {
        // 4999 < 5000: SAVE10 inactive. regular → disc=0%, ds=4999
        // ship=700, tax=4999*22/100=1099 → 6798
        assert_eq!(calculate_total_cents(&order("regular", 4999, "IT", "SAVE10", false)), 6798);
    }

    #[test]
    fn save10_threshold_at_5000() {
        // 5000 >= 5000: SAVE10 active. disc=10%, ds=4500
        // ship=700, tax=990 → 6190
        assert_eq!(calculate_total_cents(&order("regular", 5000, "IT", "SAVE10", false)), 6190);
    }

    #[test]
    fn bulk_threshold_at_19999() {
        // 19999 < 20000: BULK inactive. regular → disc=0%, ds=19999
        // ship=700, tax=19999*22/100=4399 → 25098
        assert_eq!(calculate_total_cents(&order("regular", 19999, "IT", "BULK", false)), 25098);
    }

    #[test]
    fn bulk_threshold_at_20000() {
        // 20000 >= 20000: BULK active. disc=7%, ds=18600
        // ship=700, tax=18600*22/100=4092 → 23392
        assert_eq!(calculate_total_cents(&order("regular", 20000, "IT", "BULK", false)), 23392);
    }

    #[test]
    fn freeship_threshold_at_7999() {
        // ds=7999 < 8000: FREESHIP inactive, ship stays 700
        // tax=7999*22/100=1759 → 10458
        assert_eq!(calculate_total_cents(&order("regular", 7999, "IT", "FREESHIP", false)), 10458);
    }

    #[test]
    fn freeship_threshold_at_8000() {
        // ds=8000 >= 8000: FREESHIP active → ship=0
        // tax=8000*22/100=1760 → 9760
        assert_eq!(calculate_total_cents(&order("regular", 8000, "IT", "FREESHIP", false)), 9760);
    }

    // =========================================================================
    // 9. Edge cases — zero, negative, empty, whitespace, unknown
    // =========================================================================

    #[test]
    fn zero_subtotal_regular() {
        // ds=0, ship=700, tax=0 → 700
        assert_eq!(calculate_total_cents(&order("regular", 0, "IT", "", false)), 700);
    }

    #[test]
    fn zero_subtotal_vip() {
        // disc=15%, ds=0, ship=700 (vip 0<15000), tax=0 → 700
        assert_eq!(calculate_total_cents(&order("vip", 0, "IT", "", false)), 700);
    }

    #[test]
    fn zero_subtotal_employee_de() {
        // disc=30%, ds=0, ship=900+500=1400, tax=0 → 1400
        assert_eq!(calculate_total_cents(&order("employee", 0, "DE", "", false)), 1400);
    }

    #[test]
    fn negative_subtotal_floors_to_zero() {
        // ds=-1000*(100-0)/100=-1000, ship=700, tax=-1000*22/100=-220
        // total=-1000+700-220=-520 → clamped to 0
        assert_eq!(calculate_total_cents(&order("regular", -1000, "IT", "", false)), 0);
    }

    #[test]
    fn negative_subtotal_vip_floors_to_zero() {
        // disc=15%, ds=-1000*85/100=-850, ship=700, tax=-850*20/100=-170
        // total=-850+700-170=-320 → 0
        assert_eq!(calculate_total_cents(&order("vip", -1000, "IT", "", false)), 0);
    }

    #[test]
    fn negative_subtotal_employee_de_floors_to_zero() {
        // disc=30%, ds=-5000*70/100=-3500, ship=900+500=1400, tax=-3500*19/100=-665
        // total=-3500+1400-665=-2765 → 0
        assert_eq!(calculate_total_cents(&order("employee", -5000, "DE", "", false)), 0);
    }

    #[test]
    fn subtotal_one_cent() {
        // ds=1, ship=700, tax=1*22/100=0 → 701
        assert_eq!(calculate_total_cents(&order("regular", 1, "IT", "", false)), 701);
    }

    #[test]
    fn empty_customer_type() {
        // falls to else branch → disc=0%, behaves like regular
        // ds=10000, ship=700, tax=2200 → 12900
        assert_eq!(calculate_total_cents(&order("", 10000, "IT", "", false)), 12900);
    }

    #[test]
    fn empty_country() {
        // falls to else branch → ship=2500, tax=0%
        // ds=10000 → 12500
        assert_eq!(calculate_total_cents(&order("regular", 10000, "", "", false)), 12500);
    }

    #[test]
    fn empty_everything() {
        // disc=0%, ds=0, ship=2500, tax=0 → 2500
        assert_eq!(calculate_total_cents(&order("", 0, "", "", false)), 2500);
    }

    #[test]
    fn unknown_customer_type_treated_as_zero_discount() {
        // "unknown" → disc=0%
        assert_eq!(calculate_total_cents(&order("unknown", 10000, "IT", "", false)), 12900);
    }

    #[test]
    fn whitespace_trimmed_customer_type() {
        // " regular " trimmed to "regular" by safe()
        assert_eq!(calculate_total_cents(&order(" regular ", 10000, "IT", "", false)), 12900);
    }

    #[test]
    fn whitespace_trimmed_country() {
        // " IT " trimmed to "IT"
        assert_eq!(calculate_total_cents(&order("regular", 10000, " IT ", "", false)), 12900);
    }

    #[test]
    fn whitespace_trimmed_coupon() {
        // " SAVE10 " trimmed to "SAVE10" → disc=10%, ds=9000, ship=700, tax=1980 → 11680
        assert_eq!(calculate_total_cents(&order("regular", 10000, " IT ", " SAVE10 ", false)), 11680);
    }

    #[test]
    fn case_sensitive_coupon_lowercase_ignored() {
        // "save10" != "SAVE10" → no coupon effect
        assert_eq!(calculate_total_cents(&order("regular", 10000, "IT", "save10", false)), 12900);
    }

    #[test]
    fn case_sensitive_customer_type() {
        // "VIP" != "vip" → no VIP discount, treated as unknown (0%)
        assert_eq!(calculate_total_cents(&order("VIP", 10000, "IT", "", false)), 12900);
    }

    #[test]
    fn case_sensitive_country() {
        // "it" != "IT" → falls to other country (ship=2500, tax=0%)
        assert_eq!(calculate_total_cents(&order("regular", 10000, "it", "", false)), 12500);
    }

    #[test]
    fn large_subtotal() {
        // regular, 1000000, IT: ds=1000000, ship=700, tax=220000 → 1220700
        assert_eq!(calculate_total_cents(&order("regular", 1000000, "IT", "", false)), 1220700);
    }

    // =========================================================================
    // 10. Complex combinations
    // =========================================================================

    #[test]
    fn employee_save10_bf_us() {
        // employee(30%) + SAVE10(10%) = 40%, BF adds 0% for employee
        // ds=20000*60/100=12000, ship=1500+300(BF+US)+500(employee)=2300
        // tax=12000*7/100=840 → 15140
        assert_eq!(calculate_total_cents(&order("employee", 20000, "US", "SAVE10", true)), 15140);
    }

    #[test]
    fn vip_high_subtotal_it_no_shipping() {
        // vip, 30000, IT: disc=15%, ds=25500>=15000→ship=0
        // tax=25500*20/100=5100 → 30600
        assert_eq!(calculate_total_cents(&order("vip", 30000, "IT", "", false)), 30600);
    }

    #[test]
    fn premium_save10_bf_de() {
        // premium(10%,sub>=10000) + SAVE10(10%) + BF(5%) = 25%
        // ds=15000*75/100=11250, ship=900, tax=11250*19/100=2137 → 14287
        assert_eq!(calculate_total_cents(&order("premium", 15000, "DE", "SAVE10", true)), 14287);
    }

    #[test]
    fn vip_bulk_bf_us() {
        // vip(15%) + BULK(7%,sub>=20000) + BF(5%) = 27%
        // ds=25000*73/100=18250, ship=1500+300(BF+US)=1800, vip 18250>=15000→0
        // tax=18250*7/100=1277 → 19527
        assert_eq!(calculate_total_cents(&order("vip", 25000, "US", "BULK", true)), 19527);
    }

    #[test]
    fn premium_small_subtotal_other_country() {
        // premium, 3000, JP: disc=5%(sub<10000), ds=2850, ship=2500, tax=0 → 5350
        assert_eq!(calculate_total_cents(&order("premium", 3000, "JP", "", false)), 5350);
    }

    #[test]
    fn employee_freeship_it_bf() {
        // employee, 20000, IT, FREESHIP, BF: disc=30%+0(BF employee)=30%
        // ds=14000>=8000→FREESHIP→ship=0, employee+IT→no surcharge → ship=0
        // tax=14000*22/100=3080 → 17080
        assert_eq!(calculate_total_cents(&order("employee", 20000, "IT", "FREESHIP", true)), 17080);
    }

    #[test]
    fn new_save10_bf_it() {
        // new(0%) + SAVE10(10%) + BF(5%) = 15%
        // ds=10000*85/100=8500, ship=700, tax=8500*22/100=1870 → 11070
        assert_eq!(calculate_total_cents(&order("new", 10000, "IT", "SAVE10", true)), 11070);
    }

    #[test]
    fn regular_freeship_bf_us_below_threshold() {
        // regular, 5000, US, FREESHIP, BF: disc=5%, ds=4750<8000→ship stays
        // ship=1500+300(BF+US)=1800, tax=4750*7/100=332 → 6882
        assert_eq!(calculate_total_cents(&order("regular", 5000, "US", "FREESHIP", true)), 6882);
    }

    #[test]
    fn vip_taxfree_de_bf() {
        // vip(15%) + BF(5%) = 20%, ds=10000*80/100=8000
        // ship=900 (vip 8000<15000→no free), tax=0 (TAXFREE+DE) → 8900
        assert_eq!(calculate_total_cents(&order("vip", 10000, "DE", "TAXFREE", true)), 8900);
    }

    #[test]
    fn employee_taxfree_us_bf() {
        // employee(30%) + BF(0%) = 30%, ds=10000*70/100=7000
        // ship=1500+300(BF+US)+500(employee)=2300, tax=0(TAXFREE+US) → 9300
        assert_eq!(calculate_total_cents(&order("employee", 10000, "US", "TAXFREE", true)), 9300);
    }

    #[test]
    fn vip_freeship_de_above_both_thresholds() {
        // vip, 20000, DE, FREESHIP: disc=15%, ds=17000
        // ship=900→FREESHIP (17000>=8000)→0→vip(17000>=15000)→0, ship=0
        // tax=17000*19/100=3230 → 20230
        assert_eq!(calculate_total_cents(&order("vip", 20000, "DE", "FREESHIP", false)), 20230);
    }

    // =========================================================================
    // 11. NEW — partner customer type (base discount 12%)
    // =========================================================================

    // --- Base discount across countries ---
    #[test]
    fn partner_it() {
        // disc=12%, ds=10000*88/100=8800, ship=700, tax=8800*22/100=1936 → 11436
        assert_eq!(calculate_total_cents(&order("partner", 10000, "IT", "", false)), 11436);
    }

    #[test]
    fn partner_de() {
        // disc=12%, ds=8800, ship=900, tax=8800*19/100=1672 → 11372
        assert_eq!(calculate_total_cents(&order("partner", 10000, "DE", "", false)), 11372);
    }

    #[test]
    fn partner_us() {
        // disc=12%, ds=8800, ship=1500, tax=8800*7/100=616 → 10916
        assert_eq!(calculate_total_cents(&order("partner", 10000, "US", "", false)), 10916);
    }

    #[test]
    fn partner_other_country() {
        // disc=12%, ds=8800, ship=2500, tax=0 → 11300
        assert_eq!(calculate_total_cents(&order("partner", 10000, "FR", "", false)), 11300);
    }

    // --- Free shipping at discounted subtotal >= 15000 ---
    #[test]
    fn partner_free_shipping_at_threshold() {
        // Need ds>=15000. disc=12%, ds=subtotal*88/100.
        // 17046*88/100=15000 (integer div). disc=12%, ds=15000→ship=0
        // tax=15000*19/100=2850 → 17850
        assert_eq!(calculate_total_cents(&order("partner", 17046, "DE", "", false)), 17850);
    }

    #[test]
    fn partner_no_free_shipping_below_threshold() {
        // 17045*88/100=14999. ship=900
        // tax=14999*19/100=2849 → 18748
        assert_eq!(calculate_total_cents(&order("partner", 17045, "DE", "", false)), 18748);
    }

    #[test]
    fn partner_free_shipping_it() {
        // partner, 20000, IT: disc=12%, ds=17600>=15000→ship=0
        // tax=17600*22/100=3872 → 21472
        assert_eq!(calculate_total_cents(&order("partner", 20000, "IT", "", false)), 21472);
    }

    #[test]
    fn partner_no_free_shipping_small_subtotal() {
        // partner, 5000, IT: disc=12%, ds=4400<15000→ship=700
        // tax=4400*22/100=968 → 6068
        assert_eq!(calculate_total_cents(&order("partner", 5000, "IT", "", false)), 6068);
    }

    // --- PARTNER5 coupon: +5% only for partner, subtotal >= 12000 ---
    #[test]
    fn partner5_qualifies() {
        // partner, 12000, IT, PARTNER5: disc=12+5=17%, ds=12000*83/100=9960
        // ship=700, tax=9960*22/100=2191 → 12851
        assert_eq!(calculate_total_cents(&order("partner", 12000, "IT", "PARTNER5", false)), 12851);
    }

    #[test]
    fn partner5_does_not_qualify_below_threshold() {
        // partner, 11999, IT, PARTNER5: subtotal<12000→no coupon, disc=12%
        // ds=11999*88/100=10559, ship=700, tax=10559*22/100=2322 → 13581
        assert_eq!(calculate_total_cents(&order("partner", 11999, "IT", "PARTNER5", false)), 13581);
    }

    #[test]
    fn partner5_ignored_for_regular() {
        // regular, 15000, IT, PARTNER5: not partner→coupon ignored
        // disc=0%, ds=15000, ship=700, tax=3300 → 19000
        assert_eq!(calculate_total_cents(&order("regular", 15000, "IT", "PARTNER5", false)), 19000);
    }

    #[test]
    fn partner5_ignored_for_vip() {
        // vip, 15000, IT, PARTNER5: not partner→no effect
        // disc=15%, ds=12750, ship=700, tax=12750*20/100=2550 → 16000
        assert_eq!(calculate_total_cents(&order("vip", 15000, "IT", "PARTNER5", false)), 16000);
    }

    #[test]
    fn partner5_ignored_for_employee() {
        // employee, 15000, IT, PARTNER5: not partner→no effect
        // disc=30%, ds=10500, ship=700, tax=10500*22/100=2310 → 13510
        assert_eq!(calculate_total_cents(&order("employee", 15000, "IT", "PARTNER5", false)), 13510);
    }

    // --- Black Friday: partner gets +3% (not +5%) ---
    #[test]
    fn partner_bf_it() {
        // partner, 10000, IT, BF: disc=12+3=15%, ds=8500
        // ship=700, tax=8500*22/100=1870 → 11070
        assert_eq!(calculate_total_cents(&order("partner", 10000, "IT", "", true)), 11070);
    }

    #[test]
    fn partner_bf_de() {
        // disc=12+3=15%, ds=8500, ship=900, tax=8500*19/100=1615 → 11015
        assert_eq!(calculate_total_cents(&order("partner", 10000, "DE", "", true)), 11015);
    }

    #[test]
    fn partner_bf_us() {
        // disc=12+3=15%, ds=8500, ship=1500+300=1800, tax=595 → 10895
        assert_eq!(calculate_total_cents(&order("partner", 10000, "US", "", true)), 10895);
    }

    // --- Discount cap interactions ---
    #[test]
    fn partner_partner5_bf_save10_no_stacking() {
        // PARTNER5 is in else-if chain, so SAVE10 won't stack with PARTNER5.
        // partner, 20000, IT, SAVE10, BF: disc=12+10(SAVE10)+3(BF)=25%
        // ds=20000*75/100=15000→ship=0(partner>=15000)
        // tax=15000*22/100=3300 → 18300
        assert_eq!(calculate_total_cents(&order("partner", 20000, "IT", "SAVE10", true)), 18300);
    }

    #[test]
    fn partner_bulk_bf() {
        // partner, 25000, DE, BULK, BF: disc=12+7(BULK)+3(BF)=22%
        // ds=25000*78/100=19500>=15000→ship=0
        // tax=19500*19/100=3705 → 23205
        assert_eq!(calculate_total_cents(&order("partner", 25000, "DE", "BULK", true)), 23205);
    }

    #[test]
    fn partner_partner5_bf() {
        // partner, 15000, IT, PARTNER5, BF: disc=12+5(PARTNER5)+3(BF)=20%
        // ds=15000*80/100=12000, ship=700 (12000<15000)
        // tax=12000*22/100=2640 → 15340
        assert_eq!(calculate_total_cents(&order("partner", 15000, "IT", "PARTNER5", true)), 15340);
    }

    // --- FREESHIP interaction ---
    #[test]
    fn partner_freeship_qualifies() {
        // partner, 12000, US, FREESHIP: disc=12%, ds=10560>=8000→ship=0
        // (partner 10560<15000→no partner free ship, but FREESHIP applies first)
        // tax=10560*7/100=739 → 11299
        assert_eq!(calculate_total_cents(&order("partner", 12000, "US", "FREESHIP", false)), 11299);
    }

    #[test]
    fn partner_freeship_does_not_qualify() {
        // partner, 5000, US, FREESHIP: disc=12%, ds=4400<8000→ship stays 1500
        // tax=4400*7/100=308 → 6208
        assert_eq!(calculate_total_cents(&order("partner", 5000, "US", "FREESHIP", false)), 6208);
    }

    // --- TAXFREE interaction ---
    #[test]
    fn partner_taxfree_de() {
        // partner, 10000, DE, TAXFREE: disc=12%, ds=8800, ship=900, tax=0 → 9700
        assert_eq!(calculate_total_cents(&order("partner", 10000, "DE", "TAXFREE", false)), 9700);
    }

    #[test]
    fn partner_taxfree_it_blocked() {
        // partner, 10000, IT, TAXFREE: TAXFREE blocked in IT
        // disc=12%, ds=8800, ship=700, tax=8800*22/100=1936 → 11436
        assert_eq!(calculate_total_cents(&order("partner", 10000, "IT", "TAXFREE", false)), 11436);
    }

    // --- Edge cases ---
    #[test]
    fn partner_zero_subtotal() {
        // disc=12%, ds=0, ship=700, tax=0 → 700
        assert_eq!(calculate_total_cents(&order("partner", 0, "IT", "", false)), 700);
    }

    #[test]
    fn partner_negative_subtotal_floors_to_zero() {
        // disc=12%, ds=-1000*88/100=-880, ship=700, tax=-880*22/100=-193
        // total=-880+700-193=-373 → 0
        assert_eq!(calculate_total_cents(&order("partner", -1000, "IT", "", false)), 0);
    }

    #[test]
    fn partner_high_subtotal_free_shipping_all_countries() {
        // partner, 50000, FR: disc=12%, ds=44000>=15000→ship=0, tax=0 → 44000
        assert_eq!(calculate_total_cents(&order("partner", 50000, "FR", "", false)), 44000);
    }

    #[test]
    fn partner_partner5_at_exact_threshold() {
        // partner, 12000, DE, PARTNER5: subtotal=12000>=12000→+5%
        // disc=12+5=17%, ds=12000*83/100=9960, ship=900, tax=9960*19/100=1892 → 12752
        assert_eq!(calculate_total_cents(&order("partner", 12000, "DE", "PARTNER5", false)), 12752);
    }

    #[test]
    fn partner_save10_stacks() {
        // partner, 10000, IT, SAVE10: disc=12+10=22%
        // ds=10000*78/100=7800, ship=700, tax=7800*22/100=1716 → 10216
        assert_eq!(calculate_total_cents(&order("partner", 10000, "IT", "SAVE10", false)), 10216);
    }

    #[test]
    fn partner_viponly_no_effect() {
        // VIPONLY only works for vip, not partner
        // disc=12%, ds=8800, ship=700, tax=1936 → 11436
        assert_eq!(calculate_total_cents(&order("partner", 10000, "IT", "VIPONLY", false)), 11436);
    }
}
