use legacy_checkout_kata::{calculate_total_cents, Order};

struct TestCase {
    name: &'static str,
    order: Order,
    expected_total_cents: i32,
}

#[test]
fn calculates_expected_totals_for_safety_net_cases() {
    let cases = vec![
        case("regular_it_baseline", "regular", 10_000, "IT", "", false, 12_900),
        case("new_de_baseline", "new", 10_000, "DE", "", false, 12_800),
        case("unknown_customer_other_country", "guest", 10_000, "FR", "", false, 12_500),
        case("premium_below_10000", "premium", 9_999, "US", "", false, 11_663),
        case("premium_at_10000", "premium", 10_000, "DE", "", false, 11_610),
        case("save10_at_threshold", "regular", 5_000, "DE", "SAVE10", false, 6_255),
        case("save10_below_threshold", "regular", 4_999, "DE", "SAVE10", false, 6_848),
        case("viponly_true_with_trim", " vip ", 18_000, " IT ", " VIPONLY ", false, 17_980),
        case("viponly_false_for_non_vip", "premium", 10_000, "DE", "VIPONLY", false, 11_610),
        case("bulk_at_threshold", "regular", 20_000, "FR", "BULK", false, 21_100),
        case("bulk_below_threshold", "regular", 19_999, "FR", "BULK", false, 22_499),
        case("black_friday_regular_us", "regular", 10_000, "US", "", true, 11_965),
        case("black_friday_employee_us", "employee", 10_000, "US", "", true, 9_790),
        case("black_friday_non_us_no_shipping_surcharge", "regular", 10_000, "DE", "", true, 12_205),
        case("freeship_at_threshold", "regular", 8_000, "DE", "FREESHIP", false, 9_520),
        case("freeship_below_threshold", "regular", 7_999, "DE", "FREESHIP", false, 10_418),
        case("freeship_overrides_black_friday_us_surcharge", "regular", 9_000, "US", "FREESHIP", true, 9_148),
        case("vip_free_shipping_exact_threshold", "vip", 17_648, "DE", "", false, 17_850),
        case("premium_free_shipping_exact_threshold", "premium", 22_223, "DE", "", false, 23_800),
        case("premium_free_shipping_just_below", "premium", 22_222, "DE", "", false, 24_698),
        case("employee_freeship_still_pays_surcharge", "employee", 12_000, "DE", "FREESHIP", false, 10_496),
        case("employee_it_no_shipping_surcharge", "employee", 10_000, "IT", "", false, 9_240),
        case("taxfree_non_it", "regular", 10_000, "DE", "TAXFREE", false, 10_900),
        case("taxfree_it_has_no_effect", "regular", 10_000, "IT", "TAXFREE", false, 12_900),
        case("vip_it_tax_override", "vip", 10_000, "IT", "", false, 10_900),
        case("tax_us_standard_rate", "regular", 10_000, "US", "", false, 12_200),
        case("employee_save10_hits_40_percent_boundary", "employee", 5_000, "DE", "SAVE10", false, 4_970),
        case("negative_total_clamped_to_zero", "regular", -10_000, "FR", "", false, 0),
    ];

    for test_case in cases {
        assert_eq!(
            test_case.expected_total_cents,
            calculate_total_cents(&test_case.order),
            "{}",
            test_case.name
        );
    }
}

fn case(
    name: &'static str,
    customer_type: &str,
    subtotal_cents: i32,
    country: &str,
    coupon_code: &str,
    black_friday: bool,
    expected_total_cents: i32,
) -> TestCase {
    TestCase {
        name,
        order: Order {
            customer_type: customer_type.to_string(),
            subtotal_cents,
            country: country.to_string(),
            coupon_code: coupon_code.to_string(),
            black_friday,
        },
        expected_total_cents,
    }
}