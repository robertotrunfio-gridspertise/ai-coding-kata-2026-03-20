use crate::customer_type::CustomerType;

pub(crate) fn calculate_shipping_cents(
    customer_type: CustomerType,
    country: &str,
    coupon: &str,
    discounted_subtotal: i32,
    black_friday: bool,
) -> i32 {
    // ── 1. Base rate by country ───────────────────────────────────────────────
    let mut shipping_cents = match country {
        "IT" => 700,
        "DE" => 900,
        "US" => 1_500,
        _ => 2_500,
    };

    // ── 2. Black Friday surcharge (US only) ───────────────────────────────────
    if black_friday && country == "US" {
        shipping_cents += 300;
    }

    // ── 3–6. Free shipping overrides (set to 0) ───────────────────────────────
    // NOTE: these override the base + BF surcharge, but the employee surcharge
    // in step 7 runs *after* them. An employee outside IT therefore still pays
    // +500 even when a free-shipping condition is met.
    if coupon == "FREESHIP" && discounted_subtotal >= 8_000 {
        shipping_cents = 0;
    }

    if customer_type == CustomerType::Vip && discounted_subtotal >= 15_000 {
        shipping_cents = 0;
    }

    if customer_type == CustomerType::Premium && discounted_subtotal >= 20_000 {
        shipping_cents = 0;
    }

    if customer_type == CustomerType::Partner && discounted_subtotal >= 15_000 {
        shipping_cents = 0;
    }

    // ── 7. Employee surcharge (non-IT only) — applied last ────────────────────
    if customer_type == CustomerType::Employee && country != "IT" {
        shipping_cents += 500;
    }

    shipping_cents
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::customer_type::CustomerType;

    // ── Base rates by country ─────────────────────────────────────────────────

    #[test]
    fn base_shipping_it() {
        assert_eq!(
            calculate_shipping_cents(CustomerType::Regular, "IT", "", 5_000, false),
            700
        );
    }

    #[test]
    fn base_shipping_de() {
        assert_eq!(
            calculate_shipping_cents(CustomerType::Regular, "DE", "", 5_000, false),
            900
        );
    }

    #[test]
    fn base_shipping_us() {
        assert_eq!(
            calculate_shipping_cents(CustomerType::Regular, "US", "", 5_000, false),
            1_500
        );
    }

    #[test]
    fn base_shipping_unknown_country() {
        assert_eq!(
            calculate_shipping_cents(CustomerType::Regular, "FR", "", 5_000, false),
            2_500
        );
    }

    // ── Black Friday surcharge ────────────────────────────────────────────────

    #[test]
    fn black_friday_surcharge_applies_only_in_us() {
        // US: base 1_500 + BF 300 = 1_800
        assert_eq!(
            calculate_shipping_cents(CustomerType::Regular, "US", "", 5_000, true),
            1_800
        );
    }

    #[test]
    fn black_friday_surcharge_does_not_apply_outside_us() {
        assert_eq!(
            calculate_shipping_cents(CustomerType::Regular, "IT", "", 5_000, true),
            700
        );
        assert_eq!(
            calculate_shipping_cents(CustomerType::Regular, "DE", "", 5_000, true),
            900
        );
    }

    #[test]
    fn black_friday_surcharge_applies_for_employee_in_us() {
        // BF surcharge is a shipping rule, not a discount rule — employees are not exempt
        // base 1_500 + BF 300 + employee 500 = 2_300
        assert_eq!(
            calculate_shipping_cents(CustomerType::Employee, "US", "", 5_000, true),
            2_300
        );
    }

    // ── FREESHIP coupon ───────────────────────────────────────────────────────

    #[test]
    fn freeship_sets_shipping_to_zero_at_threshold() {
        assert_eq!(
            calculate_shipping_cents(CustomerType::Regular, "DE", "FREESHIP", 8_000, false),
            0
        );
    }

    #[test]
    fn freeship_does_not_apply_below_threshold() {
        assert_eq!(
            calculate_shipping_cents(CustomerType::Regular, "DE", "FREESHIP", 7_999, false),
            900
        );
    }

    // ── VIP free shipping ─────────────────────────────────────────────────────

    #[test]
    fn vip_free_shipping_at_threshold() {
        assert_eq!(
            calculate_shipping_cents(CustomerType::Vip, "US", "", 15_000, false),
            0
        );
    }

    #[test]
    fn vip_free_shipping_not_triggered_below_threshold() {
        assert_eq!(
            calculate_shipping_cents(CustomerType::Vip, "US", "", 14_999, false),
            1_500
        );
    }

    // ── Premium free shipping ─────────────────────────────────────────────────

    #[test]
    fn premium_free_shipping_at_threshold() {
        assert_eq!(
            calculate_shipping_cents(CustomerType::Premium, "US", "", 20_000, false),
            0
        );
    }

    #[test]
    fn premium_free_shipping_not_triggered_below_threshold() {
        assert_eq!(
            calculate_shipping_cents(CustomerType::Premium, "US", "", 19_999, false),
            1_500
        );
    }

    // ── Partner free shipping ─────────────────────────────────────────────────

    #[test]
    fn partner_free_shipping_at_threshold() {
        assert_eq!(
            calculate_shipping_cents(CustomerType::Partner, "US", "", 15_000, false),
            0
        );
    }

    #[test]
    fn partner_free_shipping_not_triggered_below_threshold() {
        assert_eq!(
            calculate_shipping_cents(CustomerType::Partner, "US", "", 14_999, false),
            1_500
        );
    }

    // ── Employee surcharge ────────────────────────────────────────────────────

    #[test]
    fn employee_surcharge_applies_in_non_it_country() {
        // base 900 + employee 500 = 1_400
        assert_eq!(
            calculate_shipping_cents(CustomerType::Employee, "DE", "", 5_000, false),
            1_400
        );
    }

    #[test]
    fn employee_surcharge_does_not_apply_in_it() {
        assert_eq!(
            calculate_shipping_cents(CustomerType::Employee, "IT", "", 5_000, false),
            700
        );
    }

    // ── Employee-after-free-ship quirk (E5) ───────────────────────────────────

    #[test]
    fn employee_surcharge_is_added_after_freeship_sets_shipping_to_zero() {
        // FREESHIP sets shipping to 0, then the employee surcharge adds 500 on top.
        // This is intentional legacy behaviour that must be preserved.
        assert_eq!(
            calculate_shipping_cents(CustomerType::Employee, "DE", "FREESHIP", 8_400, false),
            500
        );
    }

    #[test]
    fn employee_surcharge_is_added_after_vip_free_shipping_override() {
        // Employee does not qualify for VIP free-ship, so base 900 + 500 = 1_400.
        assert_eq!(
            calculate_shipping_cents(CustomerType::Employee, "DE", "", 20_000, false),
            1_400
        );
    }
}
