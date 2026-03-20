#[derive(Debug, Clone)]
pub struct Order {
    pub customer_type: String,
    pub subtotal_cents: i32,
    pub country: String,
    pub coupon_code: String,
    pub black_friday: bool,
}

const VIP: &str = "vip";
const PREMIUM: &str = "premium";
const EMPLOYEE: &str = "employee";
const REGULAR: &str = "regular";
const NEW: &str = "new";

const IT: &str = "IT";
const DE: &str = "DE";
const US: &str = "US";

const SAVE10: &str = "SAVE10";
const VIPONLY: &str = "VIPONLY";
const BULK: &str = "BULK";
const FREESHIP: &str = "FREESHIP";
const TAXFREE: &str = "TAXFREE";

const MAX_DISCOUNT_PERCENT: i32 = 40;
const PREMIUM_DISCOUNT_THRESHOLD_CENTS: i32 = 10_000;
const SAVE10_THRESHOLD_CENTS: i32 = 5_000;
const BULK_THRESHOLD_CENTS: i32 = 20_000;
const FREESHIP_THRESHOLD_CENTS: i32 = 8_000;
const VIP_FREESHIP_THRESHOLD_CENTS: i32 = 15_000;
const PREMIUM_FREESHIP_THRESHOLD_CENTS: i32 = 20_000;

const IT_SHIPPING_CENTS: i32 = 700;
const DE_SHIPPING_CENTS: i32 = 900;
const US_SHIPPING_CENTS: i32 = 1_500;
const OTHER_SHIPPING_CENTS: i32 = 2_500;
const US_BLACK_FRIDAY_SURCHARGE_CENTS: i32 = 300;
const EMPLOYEE_NON_IT_SURCHARGE_CENTS: i32 = 500;

const IT_TAX_PERCENT: i32 = 22;
const VIP_IT_TAX_PERCENT: i32 = 20;
const DE_TAX_PERCENT: i32 = 19;
const US_TAX_PERCENT: i32 = 7;

#[derive(Debug)]
struct NormalizedOrder {
    customer_type: String,
    subtotal_cents: i32,
    country: String,
    coupon_code: String,
    black_friday: bool,
}

pub fn calculate_total_cents(order: &Order) -> i32 {
    let order = NormalizedOrder::from(order);
    let discount_percent = discount_percent(&order);
    let discounted_subtotal = discounted_subtotal(order.subtotal_cents, discount_percent);
    let shipping_cents = shipping_cents(&order, discounted_subtotal);
    let tax_percent = tax_percent(&order);
    let tax_cents = discounted_subtotal * tax_percent / 100;
    let total = discounted_subtotal + shipping_cents + tax_cents;

    total.max(0)
}

fn safe(value: &str) -> String {
    value.trim().to_string()
}

impl From<&Order> for NormalizedOrder {
    fn from(order: &Order) -> Self {
        Self {
            customer_type: safe(&order.customer_type),
            subtotal_cents: order.subtotal_cents,
            country: safe(&order.country),
            coupon_code: safe(&order.coupon_code),
            black_friday: order.black_friday,
        }
    }
}

fn discount_percent(order: &NormalizedOrder) -> i32 {
    let discount_percent = base_discount_percent(order)
        + coupon_discount_percent(order)
        + black_friday_discount_percent(order);

    discount_percent.min(MAX_DISCOUNT_PERCENT)
}

fn base_discount_percent(order: &NormalizedOrder) -> i32 {
    if order.customer_type == VIP {
        15
    } else if order.customer_type == PREMIUM {
        if order.subtotal_cents >= PREMIUM_DISCOUNT_THRESHOLD_CENTS {
            10
        } else {
            5
        }
    } else if order.customer_type == EMPLOYEE {
        30
    } else if order.customer_type == REGULAR || order.customer_type == NEW {
        0
    } else {
        0
    }
}

fn coupon_discount_percent(order: &NormalizedOrder) -> i32 {
    if order.coupon_code == SAVE10 {
        if order.subtotal_cents >= SAVE10_THRESHOLD_CENTS {
            10
        } else {
            0
        }
    } else if order.coupon_code == VIPONLY {
        if order.customer_type == VIP {
            5
        } else {
            0
        }
    } else if order.coupon_code == BULK {
        if order.subtotal_cents >= BULK_THRESHOLD_CENTS {
            7
        } else {
            0
        }
    } else {
        0
    }
}

fn black_friday_discount_percent(order: &NormalizedOrder) -> i32 {
    if order.black_friday && order.customer_type != EMPLOYEE {
        5
    } else {
        0
    }
}

fn discounted_subtotal(subtotal_cents: i32, discount_percent: i32) -> i32 {
    subtotal_cents * (100 - discount_percent) / 100
}

fn shipping_cents(order: &NormalizedOrder, discounted_subtotal: i32) -> i32 {
    let mut shipping_cents = base_shipping_cents(order);

    if order.black_friday && order.country == US {
        shipping_cents += US_BLACK_FRIDAY_SURCHARGE_CENTS;
    }

    if order.coupon_code == FREESHIP && discounted_subtotal >= FREESHIP_THRESHOLD_CENTS {
        shipping_cents = 0;
    }

    if order.customer_type == VIP && discounted_subtotal >= VIP_FREESHIP_THRESHOLD_CENTS {
        shipping_cents = 0;
    }

    if order.customer_type == PREMIUM && discounted_subtotal >= PREMIUM_FREESHIP_THRESHOLD_CENTS {
        shipping_cents = 0;
    }

    if order.customer_type == EMPLOYEE && order.country != IT {
        shipping_cents += EMPLOYEE_NON_IT_SURCHARGE_CENTS;
    }

    shipping_cents
}

fn base_shipping_cents(order: &NormalizedOrder) -> i32 {
    if order.country == IT {
        IT_SHIPPING_CENTS
    } else if order.country == DE {
        DE_SHIPPING_CENTS
    } else if order.country == US {
        US_SHIPPING_CENTS
    } else {
        OTHER_SHIPPING_CENTS
    }
}

fn tax_percent(order: &NormalizedOrder) -> i32 {
    let mut tax_percent = base_tax_percent(order);

    if order.customer_type == VIP && order.country == IT {
        tax_percent = VIP_IT_TAX_PERCENT;
    }

    if order.coupon_code == TAXFREE && order.country != IT {
        tax_percent = 0;
    }

    tax_percent
}

fn base_tax_percent(order: &NormalizedOrder) -> i32 {
    if order.country == IT {
        IT_TAX_PERCENT
    } else if order.country == DE {
        DE_TAX_PERCENT
    } else if order.country == US {
        US_TAX_PERCENT
    } else {
        0
    }
}
