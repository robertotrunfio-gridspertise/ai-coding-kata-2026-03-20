use crate::customer_type::CustomerType;

pub(crate) fn calculate_discount_percent(
    customer_type: CustomerType,
    subtotal: i32,
    coupon: &str,
    black_friday: bool,
) -> i32 {
    let mut discount_percent = 0;

    // ── Customer-type base discount ───────────────────────────────────────────
    match customer_type {
        CustomerType::Vip => discount_percent += 15,
        CustomerType::Premium => {
            if subtotal >= 10_000 {
                discount_percent += 10;
            } else {
                discount_percent += 5;
            }
        }
        CustomerType::Employee => discount_percent += 30,
        CustomerType::Partner => discount_percent += 12,
        CustomerType::Regular | CustomerType::New | CustomerType::Unknown => {}
    }

    // ── Coupon discount (mutually exclusive: only first match applies) ─────────
    if coupon == "SAVE10" {
        if subtotal >= 5_000 {
            discount_percent += 10;
        }
    } else if coupon == "VIPONLY" {
        if customer_type == CustomerType::Vip {
            discount_percent += 5;
        }
    } else if coupon == "BULK" {
        if subtotal >= 20_000 {
            discount_percent += 7;
        }
    } else if coupon == "PARTNER5" {
        if customer_type == CustomerType::Partner && subtotal >= 12_000 {
            discount_percent += 5;
        }
    }

    // ── Black Friday bonus ────────────────────────────────────────────────────
    // Employee: exempt (0 %). Partner: reduced rate (3 %). Everyone else: 5 %.
    if black_friday {
        discount_percent += match customer_type {
            CustomerType::Employee => 0,
            CustomerType::Partner => 3,
            _ => 5,
        };
    }

    // ── Global cap ───────────────────────────────────────────────────────────
    if discount_percent > 40 {
        discount_percent = 40;
    }

    discount_percent
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::customer_type::CustomerType;

    // ── Customer-type base discounts ──────────────────────────────────────────

    #[test]
    fn vip_gets_15_percent() {
        assert_eq!(
            calculate_discount_percent(CustomerType::Vip, 10_000, "", false),
            15
        );
    }

    #[test]
    fn premium_high_subtotal_gets_10_percent() {
        assert_eq!(
            calculate_discount_percent(CustomerType::Premium, 10_000, "", false),
            10
        );
    }

    #[test]
    fn premium_low_subtotal_gets_5_percent() {
        assert_eq!(
            calculate_discount_percent(CustomerType::Premium, 9_999, "", false),
            5
        );
    }

    #[test]
    fn employee_gets_30_percent() {
        assert_eq!(
            calculate_discount_percent(CustomerType::Employee, 10_000, "", false),
            30
        );
    }

    #[test]
    fn partner_gets_12_percent() {
        assert_eq!(
            calculate_discount_percent(CustomerType::Partner, 10_000, "", false),
            12
        );
    }

    #[test]
    fn regular_gets_0_percent() {
        assert_eq!(
            calculate_discount_percent(CustomerType::Regular, 10_000, "", false),
            0
        );
    }

    #[test]
    fn new_gets_0_percent() {
        assert_eq!(
            calculate_discount_percent(CustomerType::New, 10_000, "", false),
            0
        );
    }

    #[test]
    fn unknown_gets_0_percent() {
        assert_eq!(
            calculate_discount_percent(CustomerType::Unknown, 10_000, "", false),
            0
        );
    }

    // ── Coupon codes ──────────────────────────────────────────────────────────

    #[test]
    fn save10_applies_at_threshold() {
        assert_eq!(
            calculate_discount_percent(CustomerType::Regular, 5_000, "SAVE10", false),
            10
        );
    }

    #[test]
    fn save10_does_not_apply_below_threshold() {
        assert_eq!(
            calculate_discount_percent(CustomerType::Regular, 4_999, "SAVE10", false),
            0
        );
    }

    #[test]
    fn viponly_adds_discount_for_vip() {
        // 15 (vip) + 5 (VIPONLY) = 20
        assert_eq!(
            calculate_discount_percent(CustomerType::Vip, 10_000, "VIPONLY", false),
            20
        );
    }

    #[test]
    fn viponly_has_no_effect_for_non_vip() {
        assert_eq!(
            calculate_discount_percent(CustomerType::Regular, 10_000, "VIPONLY", false),
            0
        );
    }

    #[test]
    fn bulk_applies_at_threshold() {
        assert_eq!(
            calculate_discount_percent(CustomerType::Regular, 20_000, "BULK", false),
            7
        );
    }

    #[test]
    fn bulk_does_not_apply_below_threshold() {
        assert_eq!(
            calculate_discount_percent(CustomerType::Regular, 19_999, "BULK", false),
            0
        );
    }

    #[test]
    fn coupons_are_mutually_exclusive_only_first_match_applies() {
        // "SAVE10" matches first; "BULK" branch is never reached even if subtotal qualifies
        assert_eq!(
            calculate_discount_percent(CustomerType::Regular, 20_000, "SAVE10", false),
            10 // only SAVE10 (+10), not SAVE10 (+10) + BULK (+7)
        );
    }

    #[test]
    fn partner5_applies_to_partner_at_threshold() {
        // 12 (partner) + 5 (PARTNER5) = 17
        assert_eq!(
            calculate_discount_percent(CustomerType::Partner, 12_000, "PARTNER5", false),
            17
        );
    }

    #[test]
    fn partner5_does_not_apply_below_threshold() {
        // subtotal 11_999 < 12_000: PARTNER5 condition fails, only base 12 %
        assert_eq!(
            calculate_discount_percent(CustomerType::Partner, 11_999, "PARTNER5", false),
            12
        );
    }

    #[test]
    fn partner5_has_no_effect_for_non_partner() {
        assert_eq!(
            calculate_discount_percent(CustomerType::Regular, 12_000, "PARTNER5", false),
            0
        );
        assert_eq!(
            calculate_discount_percent(CustomerType::Vip, 12_000, "PARTNER5", false),
            15 // only VIP base discount, PARTNER5 condition fails
        );
    }

    // ── Black Friday ──────────────────────────────────────────────────────────

    #[test]
    fn black_friday_adds_5_for_standard_customers() {
        assert_eq!(
            calculate_discount_percent(CustomerType::Regular, 10_000, "", true),
            5
        );
        assert_eq!(
            calculate_discount_percent(CustomerType::Vip, 10_000, "", true),
            20 // 15 + 5
        );
    }

    #[test]
    fn black_friday_adds_3_for_partner() {
        // Partner gets a reduced BF bonus of 3 %, not the standard 5 %
        assert_eq!(
            calculate_discount_percent(CustomerType::Partner, 10_000, "", true),
            15 // 12 + 3
        );
    }

    #[test]
    fn black_friday_does_not_add_for_employee() {
        // Employee is exempt from the BF bonus; base discount unchanged
        assert_eq!(
            calculate_discount_percent(CustomerType::Employee, 10_000, "", true),
            30
        );
    }

    // ── Discount cap ──────────────────────────────────────────────────────────

    #[test]
    fn employee_save10_reaches_exactly_40_percent_boundary() {
        // 30 (employee) + 10 (SAVE10) = 40: at the cap, no reduction applied
        assert_eq!(
            calculate_discount_percent(CustomerType::Employee, 10_000, "SAVE10", false),
            40
        );
    }
}
