use legacy_checkout_kata::{Order, calculate_total_cents};

fn order(
    customer_type: &str,
    subtotal_cents: i32,
    country: &str,
    coupon_code: &str,
    black_friday: bool,
) -> Order {
    Order {
        customer_type: customer_type.to_string(),
        subtotal_cents,
        country: country.to_string(),
        coupon_code: coupon_code.to_string(),
        black_friday,
    }
}

// ── Group A: Customer Type Discounts ─────────────────────────────────────────

#[test]
fn a1_vip_in_it_no_coupon_no_bf() {
    // discount 15 %, VIP-in-IT tax override 20 %
    // discounted = 8_500, shipping = 700, tax = 1_700
    assert_eq!(
        calculate_total_cents(&order("vip", 10_000, "IT", "", false)),
        10_900
    );
}

#[test]
fn a2_premium_high_subtotal_in_de() {
    // discount 10 % (subtotal >= 10_000)
    // discounted = 13_500, shipping = 900, tax = 2_565
    assert_eq!(
        calculate_total_cents(&order("premium", 15_000, "DE", "", false)),
        16_965
    );
}

#[test]
fn a3_premium_low_subtotal_in_de() {
    // discount 5 % (subtotal < 10_000)
    // discounted = 4_750, shipping = 900, tax = 902
    assert_eq!(
        calculate_total_cents(&order("premium", 5_000, "DE", "", false)),
        6_552
    );
}

#[test]
fn a4_employee_in_us_no_coupon_no_bf() {
    // discount 30 %, employee non-IT surcharge +500
    // discounted = 7_000, shipping = 1_500 + 500 = 2_000, tax = 490
    assert_eq!(
        calculate_total_cents(&order("employee", 10_000, "US", "", false)),
        9_490
    );
}

#[test]
fn a5_regular_in_it_no_coupon_no_bf() {
    // discount 0 %
    // discounted = 5_000, shipping = 700, tax = 1_100
    assert_eq!(
        calculate_total_cents(&order("regular", 5_000, "IT", "", false)),
        6_800
    );
}

#[test]
fn a6_new_customer_in_it() {
    // same rules as regular: discount 0 %
    // discounted = 5_000, shipping = 700, tax = 1_100
    assert_eq!(
        calculate_total_cents(&order("new", 5_000, "IT", "", false)),
        6_800
    );
}

#[test]
fn a7_unknown_customer_type_in_it() {
    // unknown type falls to else branch: discount 0 %
    // discounted = 5_000, shipping = 700, tax = 1_100
    assert_eq!(
        calculate_total_cents(&order("corporate", 5_000, "IT", "", false)),
        6_800
    );
}

// ── Group B: Coupon Codes ─────────────────────────────────────────────────────

#[test]
fn b1_save10_subtotal_at_threshold() {
    // SAVE10 applies (subtotal 5_000 >= 5_000): discount 10 %
    // discounted = 4_500, shipping = 1_500, tax = 315
    assert_eq!(
        calculate_total_cents(&order("regular", 5_000, "US", "SAVE10", false)),
        6_315
    );
}

#[test]
fn b2_save10_subtotal_below_threshold() {
    // SAVE10 does not apply (4_999 < 5_000): discount 0 %
    // discounted = 4_999, shipping = 1_500, tax = 349
    assert_eq!(
        calculate_total_cents(&order("regular", 4_999, "US", "SAVE10", false)),
        6_848
    );
}

#[test]
fn b3_viponly_applied_to_vip() {
    // discount 15 (vip) + 5 (VIPONLY) = 20 %
    // discounted = 8_000, shipping = 1_500, tax = 560
    assert_eq!(
        calculate_total_cents(&order("vip", 10_000, "US", "VIPONLY", false)),
        10_060
    );
}

#[test]
fn b4_viponly_not_applied_to_non_vip() {
    // VIPONLY condition fails: discount 0 %
    // discounted = 10_000, shipping = 1_500, tax = 700
    assert_eq!(
        calculate_total_cents(&order("regular", 10_000, "US", "VIPONLY", false)),
        12_200
    );
}

#[test]
fn b5_bulk_subtotal_at_threshold() {
    // BULK applies (subtotal 20_000 >= 20_000): discount 7 %
    // discounted = 18_600, shipping = 1_500, tax = 1_302
    assert_eq!(
        calculate_total_cents(&order("regular", 20_000, "US", "BULK", false)),
        21_402
    );
}

#[test]
fn b6_bulk_subtotal_below_threshold() {
    // BULK does not apply (19_999 < 20_000): discount 0 %
    // discounted = 19_999, shipping = 1_500, tax = 1_399
    assert_eq!(
        calculate_total_cents(&order("regular", 19_999, "US", "BULK", false)),
        22_898
    );
}

#[test]
fn b7_freeship_discounted_subtotal_at_threshold() {
    // FREESHIP applies (discounted 10_000 >= 8_000): shipping = 0
    // discounted = 10_000, shipping = 0, tax = 1_900
    assert_eq!(
        calculate_total_cents(&order("regular", 10_000, "DE", "FREESHIP", false)),
        11_900
    );
}

#[test]
fn b8_freeship_discounted_subtotal_below_threshold() {
    // FREESHIP does not apply (7_000 < 8_000): shipping = 900
    // discounted = 7_000, shipping = 900, tax = 1_330
    assert_eq!(
        calculate_total_cents(&order("regular", 7_000, "DE", "FREESHIP", false)),
        9_230
    );
}

#[test]
fn b9_taxfree_in_non_it_country() {
    // TAXFREE applies outside IT: tax = 0 %
    // discounted = 10_000, shipping = 1_500, tax = 0
    assert_eq!(
        calculate_total_cents(&order("regular", 10_000, "US", "TAXFREE", false)),
        11_500
    );
}

#[test]
fn b10_taxfree_in_it_has_no_effect() {
    // TAXFREE ignored in IT: tax = 22 %
    // discounted = 10_000, shipping = 700, tax = 2_200
    assert_eq!(
        calculate_total_cents(&order("regular", 10_000, "IT", "TAXFREE", false)),
        12_900
    );
}

// ── Group C: Black Friday Flag ────────────────────────────────────────────────

#[test]
fn c1_black_friday_non_employee_in_us() {
    // discount 5 % (BF), BF US shipping surcharge +300
    // discounted = 9_500, shipping = 1_800, tax = 665
    assert_eq!(
        calculate_total_cents(&order("regular", 10_000, "US", "", true)),
        11_965
    );
}

#[test]
fn c2_black_friday_employee_in_us_no_extra_discount() {
    // employee: no BF discount bonus; BF shipping surcharge still applies
    // discount 30 %, discounted = 7_000, shipping = 1_500 + 300 + 500 = 2_300, tax = 490
    assert_eq!(
        calculate_total_cents(&order("employee", 10_000, "US", "", true)),
        9_790
    );
}

#[test]
fn c3_black_friday_non_us_no_shipping_surcharge() {
    // BF shipping surcharge only applies in US
    // discount 5 %, discounted = 9_500, shipping = 700, tax = 2_090
    assert_eq!(
        calculate_total_cents(&order("regular", 10_000, "IT", "", true)),
        12_290
    );
}

// ── Group D: Discount Cap ─────────────────────────────────────────────────────

#[test]
fn d1_employee_save10_reaches_40_percent_boundary() {
    // 30 (employee) + 10 (SAVE10, >= 5_000) = 40 %: at cap, not reduced
    // discounted = 6_000, shipping = 1_500 + 500 = 2_000, tax = 420
    assert_eq!(
        calculate_total_cents(&order("employee", 10_000, "US", "SAVE10", false)),
        8_420
    );
}

// ── Group E: Free Shipping Thresholds ────────────────────────────────────────

#[test]
fn e1_vip_free_shipping_triggered() {
    // discounted 17_000 >= 15_000: VIP free shipping
    // discount 15 %, discounted = 17_000, shipping = 0, tax = 1_190
    assert_eq!(
        calculate_total_cents(&order("vip", 20_000, "US", "", false)),
        18_190
    );
}

#[test]
fn e2_vip_free_shipping_not_triggered() {
    // discounted 12_750 < 15_000: shipping not free
    // discount 15 %, discounted = 12_750, shipping = 1_500, tax = 892
    assert_eq!(
        calculate_total_cents(&order("vip", 15_000, "US", "", false)),
        15_142
    );
}

#[test]
fn e3_premium_free_shipping_triggered() {
    // discounted 27_000 >= 20_000: Premium free shipping
    // discount 10 %, discounted = 27_000, shipping = 0, tax = 1_890
    assert_eq!(
        calculate_total_cents(&order("premium", 30_000, "US", "", false)),
        28_890
    );
}

#[test]
fn e4_employee_in_it_no_surcharge() {
    // employee in IT: no +500 surcharge
    // discount 30 %, discounted = 7_000, shipping = 700, tax = 1_540
    assert_eq!(
        calculate_total_cents(&order("employee", 10_000, "IT", "", false)),
        9_240
    );
}

#[test]
fn e5_employee_freeship_surcharge_applied_after_free_shipping_override() {
    // FREESHIP sets shipping to 0, then employee non-IT +500 is added after
    // discount 30 %, discounted = 8_400, shipping = 0 + 500 = 500, tax = 1_596
    assert_eq!(
        calculate_total_cents(&order("employee", 12_000, "DE", "FREESHIP", false)),
        10_496
    );
}

// ── Group F: Unknown Country ──────────────────────────────────────────────────

#[test]
fn f1_unknown_country_default_shipping_and_zero_tax() {
    // other country: base shipping = 2_500, tax = 0 %
    // discounted = 5_000, shipping = 2_500, tax = 0
    assert_eq!(
        calculate_total_cents(&order("regular", 5_000, "FR", "", false)),
        7_500
    );
}

// ── Group G: Input Sanitisation ───────────────────────────────────────────────

#[test]
fn g1_whitespace_trimmed_from_customer_type_and_country() {
    // " vip " → "vip", " IT " → "IT": identical result to A1
    assert_eq!(
        calculate_total_cents(&order(" vip ", 10_000, " IT ", "", false)),
        10_900
    );
}

// ── Group H: Interactions / Combinations ─────────────────────────────────────

#[test]
fn h1_vip_plus_viponly_in_it() {
    // discount 15 (vip) + 5 (VIPONLY) = 20 %, VIP-in-IT tax 20 %
    // discounted = 8_000, shipping = 700, tax = 1_600
    assert_eq!(
        calculate_total_cents(&order("vip", 10_000, "IT", "VIPONLY", false)),
        10_300
    );
}

#[test]
fn h2_vip_plus_bf_in_us_free_shipping_triggered() {
    // discount 15 (vip) + 5 (BF) = 20 %; discounted 16_000 >= 15_000 → free ship
    // shipping = 1_500 + 300 (BF) then VIP override → 0, tax = 1_120
    assert_eq!(
        calculate_total_cents(&order("vip", 20_000, "US", "", true)),
        17_120
    );
}

#[test]
fn h3_zero_subtotal_floor_guard() {
    // subtotal 0: no discount or tax, but base shipping still applies
    // discounted = 0, shipping = 1_500, tax = 0
    assert_eq!(
        calculate_total_cents(&order("regular", 0, "US", "", false)),
        1_500
    );
}

// ── Group P: Partner Customer Type ───────────────────────────────────────────

#[test]
fn p1_partner_basic_discount_us_no_coupon_no_bf() {
    // discount 12 %, partner free-ship: 8_800 < 15_000 → no
    // discounted = 8_800, shipping = 1_500, tax = 616
    assert_eq!(
        calculate_total_cents(&order("partner", 10_000, "US", "", false)),
        10_916
    );
}

#[test]
fn p2_partner_free_shipping_triggered() {
    // discount 12 %; discounted 17_600 >= 15_000 → free shipping
    // discounted = 17_600, shipping = 0, tax = 1_232
    assert_eq!(
        calculate_total_cents(&order("partner", 20_000, "US", "", false)),
        18_832
    );
}

#[test]
fn p3_partner_free_shipping_not_triggered() {
    // discount 12 %; discounted 13_200 < 15_000 → shipping not free
    // discounted = 13_200, shipping = 1_500, tax = 924
    assert_eq!(
        calculate_total_cents(&order("partner", 15_000, "US", "", false)),
        15_624
    );
}

#[test]
fn p4_partner5_applied_subtotal_at_threshold() {
    // discount 12 (partner) + 5 (PARTNER5, >= 12_000) = 17 %
    // discounted = 9_960, shipping = 700, tax = 2_191
    assert_eq!(
        calculate_total_cents(&order("partner", 12_000, "IT", "PARTNER5", false)),
        12_851
    );
}

#[test]
fn p5_partner5_not_applied_subtotal_below_threshold() {
    // PARTNER5 does not apply (11_999 < 12_000): discount 12 %
    // discounted = 10_559, shipping = 700, tax = 2_322
    assert_eq!(
        calculate_total_cents(&order("partner", 11_999, "IT", "PARTNER5", false)),
        13_581
    );
}

#[test]
fn p6_partner5_has_no_effect_for_non_partner_customer() {
    // PARTNER5 condition fails: customer is not partner → discount 0 %
    // discounted = 12_000, shipping = 700, tax = 2_640
    assert_eq!(
        calculate_total_cents(&order("regular", 12_000, "IT", "PARTNER5", false)),
        15_340
    );
}

#[test]
fn p7_partner_black_friday_gets_3_percent_not_5() {
    // discount 12 (partner) + 3 (BF: partner rate) = 15 %
    // discounted = 8_500, shipping = 1_500 + 300 (BF US) = 1_800, tax = 595
    assert_eq!(
        calculate_total_cents(&order("partner", 10_000, "US", "", true)),
        10_895
    );
}

#[test]
fn p8_partner_black_friday_partner5_free_shipping_triggered() {
    // discount 12 (partner) + 5 (PARTNER5, >= 12_000) + 3 (BF) = 20 %
    // discounted = 16_000; shipping = 1_500 + 300 (BF US) → partner: 16_000 >= 15_000 → 0
    // tax = 1_120
    assert_eq!(
        calculate_total_cents(&order("partner", 20_000, "US", "PARTNER5", true)),
        17_120
    );
}
