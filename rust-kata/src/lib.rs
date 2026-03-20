mod customer_type;
mod discount;
mod shipping;
mod tax;

use customer_type::CustomerType;
use discount::calculate_discount_percent;
use shipping::calculate_shipping_cents;
use tax::calculate_tax_percent;

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
    let customer_type = CustomerType::from_str(&order.customer_type);
    let country = safe(&order.country);
    let coupon = safe(&order.coupon_code);

    let discount_percent =
        calculate_discount_percent(customer_type, subtotal, &coupon, order.black_friday);
    let discounted_subtotal = subtotal * (100 - discount_percent) / 100;

    let shipping_cents = calculate_shipping_cents(
        customer_type,
        &country,
        &coupon,
        discounted_subtotal,
        order.black_friday,
    );

    let tax_percent = calculate_tax_percent(customer_type, &country, &coupon);
    let tax_cents = discounted_subtotal * tax_percent / 100;
    let total = discounted_subtotal + shipping_cents + tax_cents;

    if total < 0 { 0 } else { total }
}

fn safe(value: &str) -> String {
    value.trim().to_string()
}
