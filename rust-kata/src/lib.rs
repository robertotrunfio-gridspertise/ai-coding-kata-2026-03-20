#[derive(Debug, Clone)]
pub struct Order {
    pub customer_type: String,
    pub subtotal_cents: i32,
    pub country: String,
    pub coupon_code: String,
    pub black_friday: bool,
}

struct CustomerConfig {
    base_discount: fn(i32) -> i32,
    free_ship_threshold: i32,       // 0 = no threshold
    extra_ship: fn(&str) -> i32,    // returns 0 if no extra shipping
    black_friday_bonus: Option<i32>, // None = default 5
}

fn customer_config(customer_type: &str) -> CustomerConfig {
    match customer_type {
        "vip" => CustomerConfig {
            base_discount: |_| 15,
            free_ship_threshold: 15000,
            extra_ship: |_| 0,
            black_friday_bonus: None,
        },
        "premium" => CustomerConfig {
            base_discount: |s| if s >= 10000 { 10 } else { 5 },
            free_ship_threshold: 20000,
            extra_ship: |_| 0,
            black_friday_bonus: None,
        },
        "employee" => CustomerConfig {
            base_discount: |_| 30,
            free_ship_threshold: 0,
            extra_ship: |c| if c != "IT" { 500 } else { 0 },
            black_friday_bonus: Some(0),
        },
        "partner" => CustomerConfig {
            base_discount: |_| 12,
            free_ship_threshold: 15000,
            extra_ship: |_| 0,
            black_friday_bonus: Some(3),
        },
        _ => CustomerConfig {
            base_discount: |_| 0,
            free_ship_threshold: 0,
            extra_ship: |_| 0,
            black_friday_bonus: None,
        },
    }
}

fn coupon_discount(coupon: &str, customer_type: &str, subtotal: i32) -> i32 {
    match coupon {
        "SAVE10"   => if subtotal >= 5000 { 10 } else { 0 },
        "VIPONLY"  => if customer_type == "vip" { 5 } else { 0 },
        "BULK"     => if subtotal >= 20000 { 7 } else { 0 },
        "PARTNER5" => if customer_type == "partner" && subtotal >= 12000 { 5 } else { 0 },
        _          => 0,
    }
}

fn base_shipping(country: &str) -> i32 {
    match country {
        "IT" => 700,
        "DE" => 900,
        "US" => 1500,
        _    => 2500,
    }
}

fn country_tax(country: &str) -> i32 {
    match country {
        "IT" => 22,
        "DE" => 19,
        "US" => 7,
        _    => 0,
    }
}

const MAX_DISCOUNT: i32    = 40;
const DEFAULT_BF_BONUS: i32 = 5;

pub fn calculate_total_cents(order: &Order) -> i32 {
    let subtotal       = order.subtotal_cents;
    let customer_type  = safe(&order.customer_type);
    let country        = safe(&order.country);
    let coupon         = safe(&order.coupon_code);

    let customer = customer_config(&customer_type);

    // Discount
    let mut discount = (customer.base_discount)(subtotal);
    discount += coupon_discount(&coupon, &customer_type, subtotal);
    if order.black_friday {
        discount += customer.black_friday_bonus.unwrap_or(DEFAULT_BF_BONUS);
    }
    if discount > MAX_DISCOUNT {
        discount = MAX_DISCOUNT;
    }

    let discounted_subtotal = subtotal * (100 - discount) / 100;

    // Shipping
    let mut shipping = base_shipping(&country);
    if order.black_friday && country == "US" {
        shipping += 300;
    }
    if coupon == "FREESHIP" && discounted_subtotal >= 8000 {
        shipping = 0;
    }
    if customer.free_ship_threshold > 0 && discounted_subtotal >= customer.free_ship_threshold {
        shipping = 0;
    }
    shipping += (customer.extra_ship)(&country);

    // Tax
    let mut tax = country_tax(&country);
    if customer_type == "vip" && country == "IT" {
        tax = 20;
    }
    if coupon == "TAXFREE" && country != "IT" {
        tax = 0;
    }

    let tax_cents = discounted_subtotal * tax / 100;
    let total = discounted_subtotal + shipping + tax_cents;

    if total < 0 { 0 } else { total }
}

fn safe(value: &str) -> String {
    value.trim().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn order(customer_type: &str, subtotal: i32, country: &str, coupon: &str, bf: bool) -> Order {
        Order {
            customer_type: customer_type.to_string(),
            subtotal_cents: subtotal,
            country: country.to_string(),
            coupon_code: coupon.to_string(),
            black_friday: bf,
        }
    }

    // README examples — behavior preservation
    #[test] fn regular_it_no_coupon()  { assert_eq!(12900, calculate_total_cents(&order("regular", 10000, "IT", "", false))); }
    #[test] fn premium_de_save10()     { assert_eq!(10420, calculate_total_cents(&order("premium", 10000, "DE", "SAVE10", false))); }
    #[test] fn vip_it_viponly()        { assert_eq!(17980, calculate_total_cents(&order("vip", 18000, "IT", "VIPONLY", false))); }

    // Customer base discounts
    #[test] fn employee_de_shipping_surcharge()      { assert_eq!(9730,  calculate_total_cents(&order("employee", 10000, "DE", "", false))); }
    #[test] fn employee_it_no_surcharge()            { assert_eq!(9240,  calculate_total_cents(&order("employee", 10000, "IT", "", false))); }
    #[test] fn premium_low_subtotal_5pct_discount()  { assert_eq!(6552,  calculate_total_cents(&order("premium",  5000,  "DE", "", false))); }

    // Free shipping thresholds
    #[test] fn vip_it_free_shipping()            { assert_eq!(20400, calculate_total_cents(&order("vip",     20000, "IT", "", false))); }
    #[test] fn vip_it_below_free_shipping()      { assert_eq!(10900, calculate_total_cents(&order("vip",     10000, "IT", "", false))); }
    #[test] fn premium_it_free_shipping()        { assert_eq!(27450, calculate_total_cents(&order("premium", 25000, "IT", "", false))); }

    // Coupon rules
    #[test] fn save10_below_threshold_not_applied() { assert_eq!(5422,  calculate_total_cents(&order("premium", 4000,  "DE", "SAVE10",  false))); }
    #[test] fn bulk_large_order()                   { assert_eq!(28567, calculate_total_cents(&order("regular", 25000, "DE", "BULK",    false))); }
    #[test] fn viponly_on_non_vip_no_effect()        { assert_eq!(11680, calculate_total_cents(&order("premium", 10000, "IT", "VIPONLY", false))); }
    #[test] fn freeship_coupon()                    { assert_eq!(12200, calculate_total_cents(&order("regular", 10000, "IT", "FREESHIP",false))); }
    #[test] fn taxfree_de()                         { assert_eq!(10900, calculate_total_cents(&order("regular", 10000, "DE", "TAXFREE", false))); }
    #[test] fn taxfree_it_not_applied()             { assert_eq!(12900, calculate_total_cents(&order("regular", 10000, "IT", "TAXFREE", false))); }

    // Black Friday
    #[test] fn regular_bf_us_shipping_surcharge()  { assert_eq!(11965, calculate_total_cents(&order("regular",  10000, "US", "", true))); }
    #[test] fn employee_bf_no_bonus()              { assert_eq!(9240,  calculate_total_cents(&order("employee", 10000, "IT", "", true))); }
    #[test] fn vip_bf_it()                         { assert_eq!(10300, calculate_total_cents(&order("vip",      10000, "IT", "", true))); }

    // Edge cases
    #[test] fn unknown_customer_type()         { assert_eq!(12900, calculate_total_cents(&order("unknown", 10000, "IT", "", false))); }
    #[test] fn unknown_country_default_ship()  { assert_eq!(12500, calculate_total_cents(&order("regular", 10000, "FR", "", false))); }
    #[test] fn empty_strings()                 { assert_eq!(12500, calculate_total_cents(&order("", 10000, "", "", false))); }

    // Partner — new requirement
    #[test] fn partner_it_below_free_ship()              { assert_eq!(16804, calculate_total_cents(&order("partner", 15000, "IT", "",         false))); }
    #[test] fn partner_it_free_shipping()                { assert_eq!(21472, calculate_total_cents(&order("partner", 20000, "IT", "",         false))); }
    #[test] fn partner_it_partner5_coupon()              { assert_eq!(15889, calculate_total_cents(&order("partner", 15000, "IT", "PARTNER5", false))); }
    #[test] fn partner_partner5_below_subtotal_threshold(){ assert_eq!(11436, calculate_total_cents(&order("partner", 10000, "IT", "PARTNER5", false))); }
    #[test] fn partner_bf_gets_plus3_not_plus5()         { assert_eq!(16255, calculate_total_cents(&order("partner", 15000, "IT", "",         true))); }
    #[test] fn partner5_on_non_partner_no_discount()     { assert_eq!(19000, calculate_total_cents(&order("regular", 15000, "IT", "PARTNER5", false))); }
}
