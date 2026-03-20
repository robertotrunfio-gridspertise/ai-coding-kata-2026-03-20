use crate::customer_type::CustomerType;

pub(crate) fn calculate_tax_percent(
    customer_type: CustomerType,
    country: &str,
    coupon: &str,
) -> i32 {
    // ── 1. Base rate by country ───────────────────────────────────────────────
    let mut tax_percent = match country {
        "IT" => 22,
        "DE" => 19,
        "US" => 7,
        _ => 0,
    };

    // ── 2. VIP-in-IT override (reduced rate) ─────────────────────────────────
    if customer_type == CustomerType::Vip && country == "IT" {
        tax_percent = 20;
    }

    // ── 3. TAXFREE coupon override (non-IT only) ──────────────────────────────
    if coupon == "TAXFREE" && country != "IT" {
        tax_percent = 0;
    }

    tax_percent
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::customer_type::CustomerType;

    // ── Base rates by country ─────────────────────────────────────────────────

    #[test]
    fn tax_rate_in_it_is_22_percent() {
        assert_eq!(calculate_tax_percent(CustomerType::Regular, "IT", ""), 22);
    }

    #[test]
    fn tax_rate_in_de_is_19_percent() {
        assert_eq!(calculate_tax_percent(CustomerType::Regular, "DE", ""), 19);
    }

    #[test]
    fn tax_rate_in_us_is_7_percent() {
        assert_eq!(calculate_tax_percent(CustomerType::Regular, "US", ""), 7);
    }

    #[test]
    fn tax_rate_in_unknown_country_is_0_percent() {
        assert_eq!(calculate_tax_percent(CustomerType::Regular, "FR", ""), 0);
        assert_eq!(calculate_tax_percent(CustomerType::Regular, "JP", ""), 0);
    }

    // ── VIP-in-IT override ────────────────────────────────────────────────────

    #[test]
    fn vip_in_it_gets_reduced_rate_of_20_percent() {
        assert_eq!(calculate_tax_percent(CustomerType::Vip, "IT", ""), 20);
    }

    #[test]
    fn vip_outside_it_uses_standard_country_rate() {
        assert_eq!(calculate_tax_percent(CustomerType::Vip, "DE", ""), 19);
        assert_eq!(calculate_tax_percent(CustomerType::Vip, "US", ""), 7);
    }

    #[test]
    fn non_vip_in_it_gets_standard_22_percent() {
        assert_eq!(calculate_tax_percent(CustomerType::Premium, "IT", ""), 22);
        assert_eq!(calculate_tax_percent(CustomerType::Employee, "IT", ""), 22);
        assert_eq!(calculate_tax_percent(CustomerType::Regular, "IT", ""), 22);
    }

    // ── TAXFREE coupon ────────────────────────────────────────────────────────

    #[test]
    fn taxfree_coupon_sets_rate_to_zero_in_non_it_countries() {
        assert_eq!(
            calculate_tax_percent(CustomerType::Regular, "US", "TAXFREE"),
            0
        );
        assert_eq!(
            calculate_tax_percent(CustomerType::Regular, "DE", "TAXFREE"),
            0
        );
        assert_eq!(
            calculate_tax_percent(CustomerType::Regular, "FR", "TAXFREE"),
            0
        );
    }

    #[test]
    fn taxfree_coupon_has_no_effect_in_it() {
        // IT is explicitly excluded from the TAXFREE coupon
        assert_eq!(
            calculate_tax_percent(CustomerType::Regular, "IT", "TAXFREE"),
            22
        );
    }

    #[test]
    fn taxfree_coupon_has_no_effect_on_vip_in_it() {
        // VIP override gives 20 %, TAXFREE still cannot apply in IT
        assert_eq!(
            calculate_tax_percent(CustomerType::Vip, "IT", "TAXFREE"),
            20
        );
    }
}
