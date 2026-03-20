// --- Domain types ---

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CustomerType {
    Regular,
    New,
    Vip,
    Premium,
    Employee,
    Partner,
}

impl CustomerType {
    fn parse(s: &str) -> Self {
        match s.trim() {
            "vip" => Self::Vip,
            "premium" => Self::Premium,
            "employee" => Self::Employee,
            "partner" => Self::Partner,
            "new" => Self::New,
            _ => Self::Regular,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Country {
    IT,
    DE,
    US,
    Other,
}

impl Country {
    fn parse(s: &str) -> Self {
        match s.trim() {
            "IT" => Self::IT,
            "DE" => Self::DE,
            "US" => Self::US,
            _ => Self::Other,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum CouponCode {
    None,
    Save10,
    VipOnly,
    Bulk,
    FreeShip,
    TaxFree,
    Partner5,
}

impl CouponCode {
    fn parse(s: &str) -> Self {
        match s.trim() {
            "SAVE10" => Self::Save10,
            "VIPONLY" => Self::VipOnly,
            "BULK" => Self::Bulk,
            "FREESHIP" => Self::FreeShip,
            "TAXFREE" => Self::TaxFree,
            "PARTNER5" => Self::Partner5,
            _ => Self::None,
        }
    }
}

// --- Constants ---

const VIP_DISCOUNT: i32 = 15;
const PREMIUM_HIGH_DISCOUNT: i32 = 10;
const PREMIUM_LOW_DISCOUNT: i32 = 5;
const EMPLOYEE_DISCOUNT: i32 = 30;
const PARTNER_DISCOUNT: i32 = 12;
const SAVE10_DISCOUNT: i32 = 10;
const VIPONLY_DISCOUNT: i32 = 5;
const BULK_DISCOUNT: i32 = 7;
const PARTNER5_DISCOUNT: i32 = 5;
const BLACK_FRIDAY_DISCOUNT: i32 = 5;
const PARTNER_BLACK_FRIDAY_DISCOUNT: i32 = 3;
const MAX_DISCOUNT: i32 = 40;

const PREMIUM_HIGH_DISCOUNT_THRESHOLD: i32 = 10_000;
const SAVE10_THRESHOLD: i32 = 5_000;
const BULK_THRESHOLD: i32 = 20_000;
const PARTNER5_THRESHOLD: i32 = 12_000;

const SHIPPING_IT: i32 = 700;
const SHIPPING_DE: i32 = 900;
const SHIPPING_US: i32 = 1_500;
const SHIPPING_DEFAULT: i32 = 2_500;
const BLACK_FRIDAY_US_SURCHARGE: i32 = 300;
const EMPLOYEE_ABROAD_SURCHARGE: i32 = 500;
const FREESHIP_THRESHOLD: i32 = 8_000;
const VIP_FREE_SHIPPING_THRESHOLD: i32 = 15_000;
const PREMIUM_FREE_SHIPPING_THRESHOLD: i32 = 20_000;
const PARTNER_FREE_SHIPPING_THRESHOLD: i32 = 15_000;

const TAX_IT: i32 = 22;
const TAX_DE: i32 = 19;
const TAX_US: i32 = 7;
const TAX_DEFAULT: i32 = 0;
const VIP_TAX_IT: i32 = 20;

// --- Public API (unchanged) ---

#[derive(Debug, Clone)]
pub struct Order {
    pub customer_type: String,
    pub subtotal_cents: i32,
    pub country: String,
    pub coupon_code: String,
    pub black_friday: bool,
}

pub fn calculate_total_cents(order: &Order) -> i32 {
    let customer = CustomerType::parse(&order.customer_type);
    let country = Country::parse(&order.country);
    let coupon = CouponCode::parse(&order.coupon_code);
    let subtotal = order.subtotal_cents;

    let discount_percent = calculate_discount_percent(customer, subtotal, coupon, order.black_friday);
    let discounted_subtotal = subtotal * (100 - discount_percent) / 100;

    let shipping = calculate_shipping_cents(customer, country, coupon, discounted_subtotal, order.black_friday);
    let tax = calculate_tax_cents(customer, country, coupon, discounted_subtotal);

    let total = discounted_subtotal + shipping + tax;
    total.max(0)
}

// --- Discount logic ---

fn calculate_discount_percent(
    customer: CustomerType,
    subtotal: i32,
    coupon: CouponCode,
    black_friday: bool,
) -> i32 {
    let customer_discount = customer_type_discount(customer, subtotal);
    let coupon_discount = coupon_discount(customer, subtotal, coupon);
    let bf_discount = black_friday_discount(customer, black_friday);

    (customer_discount + coupon_discount + bf_discount).min(MAX_DISCOUNT)
}

fn customer_type_discount(customer: CustomerType, subtotal: i32) -> i32 {
    match customer {
        CustomerType::Vip => VIP_DISCOUNT,
        CustomerType::Premium if subtotal >= PREMIUM_HIGH_DISCOUNT_THRESHOLD => PREMIUM_HIGH_DISCOUNT,
        CustomerType::Premium => PREMIUM_LOW_DISCOUNT,
        CustomerType::Employee => EMPLOYEE_DISCOUNT,
        CustomerType::Partner => PARTNER_DISCOUNT,
        CustomerType::Regular | CustomerType::New => 0,
    }
}

fn coupon_discount(customer: CustomerType, subtotal: i32, coupon: CouponCode) -> i32 {
    match coupon {
        CouponCode::Save10 if subtotal >= SAVE10_THRESHOLD => SAVE10_DISCOUNT,
        CouponCode::VipOnly if customer == CustomerType::Vip => VIPONLY_DISCOUNT,
        CouponCode::Bulk if subtotal >= BULK_THRESHOLD => BULK_DISCOUNT,
        CouponCode::Partner5 if customer == CustomerType::Partner && subtotal >= PARTNER5_THRESHOLD => PARTNER5_DISCOUNT,
        _ => 0,
    }
}

fn black_friday_discount(customer: CustomerType, black_friday: bool) -> i32 {
    match (black_friday, customer) {
        (true, CustomerType::Employee) => 0,
        (true, CustomerType::Partner) => PARTNER_BLACK_FRIDAY_DISCOUNT,
        (true, _) => BLACK_FRIDAY_DISCOUNT,
        _ => 0,
    }
}

// --- Shipping logic ---

fn calculate_shipping_cents(
    customer: CustomerType,
    country: Country,
    coupon: CouponCode,
    discounted_subtotal: i32,
    black_friday: bool,
) -> i32 {
    let mut shipping = base_shipping(country);

    if black_friday && country == Country::US {
        shipping += BLACK_FRIDAY_US_SURCHARGE;
    }

    if coupon == CouponCode::FreeShip && discounted_subtotal >= FREESHIP_THRESHOLD {
        shipping = 0;
    }

    if customer == CustomerType::Vip && discounted_subtotal >= VIP_FREE_SHIPPING_THRESHOLD {
        shipping = 0;
    }

    if customer == CustomerType::Premium && discounted_subtotal >= PREMIUM_FREE_SHIPPING_THRESHOLD {
        shipping = 0;
    }

    if customer == CustomerType::Partner && discounted_subtotal >= PARTNER_FREE_SHIPPING_THRESHOLD {
        shipping = 0;
    }

    if customer == CustomerType::Employee && country != Country::IT {
        shipping += EMPLOYEE_ABROAD_SURCHARGE;
    }

    shipping
}

fn base_shipping(country: Country) -> i32 {
    match country {
        Country::IT => SHIPPING_IT,
        Country::DE => SHIPPING_DE,
        Country::US => SHIPPING_US,
        Country::Other => SHIPPING_DEFAULT,
    }
}

// --- Tax logic ---

fn calculate_tax_cents(
    customer: CustomerType,
    country: Country,
    coupon: CouponCode,
    discounted_subtotal: i32,
) -> i32 {
    let tax_percent = calculate_tax_percent(customer, country, coupon);
    discounted_subtotal * tax_percent / 100
}

fn calculate_tax_percent(customer: CustomerType, country: Country, coupon: CouponCode) -> i32 {
    if coupon == CouponCode::TaxFree && country != Country::IT {
        return TAX_DEFAULT;
    }

    if customer == CustomerType::Vip && country == Country::IT {
        return VIP_TAX_IT;
    }

    match country {
        Country::IT => TAX_IT,
        Country::DE => TAX_DE,
        Country::US => TAX_US,
        Country::Other => TAX_DEFAULT,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn order(customer_type: &str, subtotal: i32, country: &str, coupon: &str, black_friday: bool) -> Order {
        Order {
            customer_type: customer_type.to_string(),
            subtotal_cents: subtotal,
            country: country.to_string(),
            coupon_code: coupon.to_string(),
            black_friday,
        }
    }

    // --- Customer type discounts ---

    #[test]
    fn regular_customer_no_extras() {
        assert_eq!(calculate_total_cents(&order("regular", 10000, "IT", "", false)), 12900);
    }

    #[test]
    fn new_customer_same_as_regular() {
        assert_eq!(calculate_total_cents(&order("new", 10000, "IT", "", false)), 12900);
    }

    #[test]
    fn vip_discount_and_reduced_tax_in_italy() {
        // VIP: 15% discount, 20% tax in IT
        assert_eq!(calculate_total_cents(&order("vip", 10000, "IT", "", false)), 10900);
    }

    #[test]
    fn premium_low_discount_below_threshold() {
        // Premium below 100€: 5% discount
        assert_eq!(calculate_total_cents(&order("premium", 9999, "IT", "", false)), 12288);
    }

    #[test]
    fn premium_high_discount_at_threshold() {
        // Premium at 100€: 10% discount
        assert_eq!(calculate_total_cents(&order("premium", 10000, "IT", "", false)), 11680);
    }

    #[test]
    fn employee_discount() {
        assert_eq!(calculate_total_cents(&order("employee", 10000, "IT", "", false)), 9240);
    }

    // --- Coupon codes ---

    #[test]
    fn save10_coupon_above_threshold() {
        assert_eq!(calculate_total_cents(&order("regular", 5000, "IT", "SAVE10", false)), 6190);
    }

    #[test]
    fn save10_coupon_below_threshold_no_effect() {
        assert_eq!(calculate_total_cents(&order("regular", 4999, "IT", "SAVE10", false)), 6798);
    }

    #[test]
    fn viponly_coupon_for_vip() {
        assert_eq!(calculate_total_cents(&order("vip", 10000, "IT", "VIPONLY", false)), 10300);
    }

    #[test]
    fn viponly_coupon_for_non_vip_no_effect() {
        assert_eq!(calculate_total_cents(&order("regular", 10000, "IT", "VIPONLY", false)), 12900);
    }

    #[test]
    fn bulk_coupon_above_threshold() {
        assert_eq!(calculate_total_cents(&order("regular", 20000, "IT", "BULK", false)), 23392);
    }

    #[test]
    fn bulk_coupon_below_threshold_no_effect() {
        assert_eq!(calculate_total_cents(&order("regular", 19999, "IT", "BULK", false)), 25098);
    }

    // --- Black Friday ---

    #[test]
    fn black_friday_regular_customer() {
        assert_eq!(calculate_total_cents(&order("regular", 10000, "IT", "", true)), 12290);
    }

    #[test]
    fn black_friday_employee_no_extra_discount() {
        assert_eq!(calculate_total_cents(&order("employee", 10000, "IT", "", true)), 9240);
    }

    // --- Discount cap ---

    #[test]
    fn employee_save10_reaches_discount_cap() {
        // employee(30) + SAVE10(10) = 40%, exactly at cap
        assert_eq!(calculate_total_cents(&order("employee", 20000, "IT", "SAVE10", true)), 15340);
    }

    // --- Shipping by country ---

    #[test]
    fn shipping_germany() {
        assert_eq!(calculate_total_cents(&order("regular", 10000, "DE", "", false)), 12800);
    }

    #[test]
    fn shipping_us() {
        assert_eq!(calculate_total_cents(&order("regular", 10000, "US", "", false)), 12200);
    }

    #[test]
    fn shipping_other_country() {
        assert_eq!(calculate_total_cents(&order("regular", 10000, "FR", "", false)), 12500);
    }

    // --- Shipping modifiers ---

    #[test]
    fn black_friday_us_shipping_surcharge() {
        assert_eq!(calculate_total_cents(&order("regular", 10000, "US", "", true)), 11965);
    }

    #[test]
    fn freeship_coupon_above_threshold() {
        assert_eq!(calculate_total_cents(&order("regular", 10000, "IT", "FREESHIP", false)), 12200);
    }

    #[test]
    fn freeship_coupon_below_threshold_no_effect() {
        assert_eq!(calculate_total_cents(&order("regular", 7999, "IT", "FREESHIP", false)), 10458);
    }

    #[test]
    fn vip_free_shipping_above_threshold() {
        assert_eq!(calculate_total_cents(&order("vip", 20000, "IT", "", false)), 20400);
    }

    #[test]
    fn vip_shipping_below_threshold() {
        assert_eq!(calculate_total_cents(&order("vip", 17000, "IT", "", false)), 18040);
    }

    #[test]
    fn premium_free_shipping_above_threshold() {
        assert_eq!(calculate_total_cents(&order("premium", 25000, "IT", "", false)), 27450);
    }

    #[test]
    fn employee_abroad_shipping_surcharge() {
        assert_eq!(calculate_total_cents(&order("employee", 10000, "DE", "", false)), 9730);
    }

    #[test]
    fn employee_freeship_still_gets_abroad_surcharge() {
        // FREESHIP zeros shipping, but employee abroad surcharge applies after
        assert_eq!(calculate_total_cents(&order("employee", 15000, "US", "FREESHIP", false)), 11735);
    }

    #[test]
    fn employee_black_friday_us_full_shipping_stack() {
        // base 1500 + BF surcharge 300 + employee abroad 500 = 2300
        assert_eq!(calculate_total_cents(&order("employee", 10000, "US", "", true)), 9790);
    }

    // --- Tax ---

    #[test]
    fn taxfree_coupon_in_germany() {
        assert_eq!(calculate_total_cents(&order("regular", 10000, "DE", "TAXFREE", false)), 10900);
    }

    #[test]
    fn taxfree_coupon_in_italy_no_effect() {
        assert_eq!(calculate_total_cents(&order("regular", 10000, "IT", "TAXFREE", false)), 12900);
    }

    #[test]
    fn taxfree_coupon_in_us() {
        assert_eq!(calculate_total_cents(&order("regular", 10000, "US", "TAXFREE", false)), 11500);
    }

    // --- Edge cases ---

    #[test]
    fn zero_subtotal() {
        assert_eq!(calculate_total_cents(&order("regular", 0, "IT", "", false)), 700);
    }

    // --- Partner customer type ---

    #[test]
    fn partner_base_discount() {
        // 12% discount on 10000 = 8800, shipping IT 700, tax 22% of 8800 = 1936
        assert_eq!(calculate_total_cents(&order("partner", 10000, "IT", "", false)), 11436);
    }

    #[test]
    fn partner_free_shipping_above_threshold() {
        // 12% discount on 20000 = 17600 (>= 15000 → free shipping), tax 22% of 17600 = 3872
        assert_eq!(calculate_total_cents(&order("partner", 20000, "IT", "", false)), 21472);
    }

    #[test]
    fn partner_no_free_shipping_below_threshold() {
        // 12% discount on 16000 = 14080 (< 15000 → pays shipping)
        // shipping IT 700, tax 22% of 14080 = 3097
        assert_eq!(calculate_total_cents(&order("partner", 16000, "IT", "", false)), 17877);
    }

    #[test]
    fn partner5_coupon_above_threshold() {
        // partner(12%) + PARTNER5(5%) = 17% on 12000 = 9960
        // shipping IT 700, tax 22% of 9960 = 2191
        assert_eq!(calculate_total_cents(&order("partner", 12000, "IT", "PARTNER5", false)), 12851);
    }

    #[test]
    fn partner5_coupon_below_threshold_no_effect() {
        // subtotal 11999 < 12000 → PARTNER5 not applied, only 12% discount
        // 12% on 11999 = 10559, shipping IT 700, tax 22% of 10559 = 2322
        assert_eq!(calculate_total_cents(&order("partner", 11999, "IT", "PARTNER5", false)), 13581);
    }

    #[test]
    fn partner5_coupon_for_non_partner_no_effect() {
        // PARTNER5 only works for partner customers
        assert_eq!(calculate_total_cents(&order("regular", 15000, "IT", "PARTNER5", false)), 19000);
    }

    #[test]
    fn partner_black_friday_gets_3_percent() {
        // partner(12%) + BF(3%) = 15% on 10000 = 8500
        // shipping IT 700, tax 22% of 8500 = 1870
        assert_eq!(calculate_total_cents(&order("partner", 10000, "IT", "", true)), 11070);
    }

    #[test]
    fn partner_black_friday_with_partner5() {
        // partner(12%) + PARTNER5(5%) + BF(3%) = 20% on 15000 = 12000 (< 15000 → pays shipping)
        // shipping IT 700, tax 22% of 12000 = 2640
        assert_eq!(calculate_total_cents(&order("partner", 15000, "IT", "PARTNER5", true)), 15340);
    }

    #[test]
    fn partner_shipping_germany() {
        // 12% on 10000 = 8800, shipping DE 900, tax 19% of 8800 = 1672
        assert_eq!(calculate_total_cents(&order("partner", 10000, "DE", "", false)), 11372);
    }

    #[test]
    fn partner_taxfree_coupon_outside_italy() {
        // 12% on 10000 = 8800, shipping DE 900, tax 0%
        assert_eq!(calculate_total_cents(&order("partner", 10000, "DE", "TAXFREE", false)), 9700);
    }
}
