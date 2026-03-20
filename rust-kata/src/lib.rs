#[derive(Debug, Clone)]
pub struct Order {
    pub customer_type: String,
    pub subtotal_cents: i32,
    pub country: String,
    pub coupon_code: String,
    pub black_friday: bool,
}

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
    customer_kind: CustomerKind,
    subtotal_cents: i32,
    country_kind: CountryKind,
    coupon_kind: CouponKind,
    black_friday: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CustomerKind {
    Vip,
    Premium,
    Employee,
    Regular,
    New,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CountryKind {
    It,
    De,
    Us,
    Other,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum CouponKind {
    Save10,
    VipOnly,
    Bulk,
    FreeShip,
    TaxFree,
    Other,
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
            customer_kind: CustomerKind::from_raw(&order.customer_type),
            subtotal_cents: order.subtotal_cents,
            country_kind: CountryKind::from_raw(&order.country),
            coupon_kind: CouponKind::from_raw(&order.coupon_code),
            black_friday: order.black_friday,
        }
    }
}

fn discount_percent(order: &NormalizedOrder) -> i32 {
    let discount_percent = order.customer_kind.base_discount_percent(order.subtotal_cents)
        + order.coupon_kind.discount_percent(order)
        + order
            .customer_kind
            .black_friday_discount_percent(order.black_friday);

    discount_percent.min(MAX_DISCOUNT_PERCENT)
}

fn discounted_subtotal(subtotal_cents: i32, discount_percent: i32) -> i32 {
    subtotal_cents * (100 - discount_percent) / 100
}

fn shipping_cents(order: &NormalizedOrder, discounted_subtotal: i32) -> i32 {
    let mut shipping_cents = order.country_kind.base_shipping_cents();

    if order.black_friday && order.country_kind == CountryKind::Us {
        shipping_cents += US_BLACK_FRIDAY_SURCHARGE_CENTS;
    }

    if order.coupon_kind.applies_free_shipping(discounted_subtotal) {
        shipping_cents = 0;
    }

    if let Some(threshold) = order.customer_kind.free_shipping_threshold() {
        if discounted_subtotal >= threshold {
            shipping_cents = 0;
        }
    }

    shipping_cents += order.customer_kind.shipping_surcharge_cents(order.country_kind);

    shipping_cents
}

fn tax_percent(order: &NormalizedOrder) -> i32 {
    let mut tax_percent = order.country_kind.base_tax_percent();

    if let Some(tax_override_percent) = order.customer_kind.tax_override_percent(order.country_kind) {
        tax_percent = tax_override_percent;
    }

    if let Some(tax_override_percent) = order.coupon_kind.tax_override_percent(order.country_kind) {
        tax_percent = tax_override_percent;
    }

    tax_percent
}

impl CustomerKind {
    fn from_raw(value: &str) -> Self {
        match safe(value).as_str() {
            "vip" => Self::Vip,
            "premium" => Self::Premium,
            "employee" => Self::Employee,
            "regular" => Self::Regular,
            "new" => Self::New,
            _ => Self::Unknown,
        }
    }

    fn base_discount_percent(self, subtotal_cents: i32) -> i32 {
        match self {
            Self::Vip => 15,
            Self::Premium if subtotal_cents >= PREMIUM_DISCOUNT_THRESHOLD_CENTS => 10,
            Self::Premium => 5,
            Self::Employee => 30,
            Self::Regular | Self::New | Self::Unknown => 0,
        }
    }

    fn black_friday_discount_percent(self, black_friday: bool) -> i32 {
        if black_friday && self != Self::Employee {
            5
        } else {
            0
        }
    }

    fn free_shipping_threshold(self) -> Option<i32> {
        match self {
            Self::Vip => Some(VIP_FREESHIP_THRESHOLD_CENTS),
            Self::Premium => Some(PREMIUM_FREESHIP_THRESHOLD_CENTS),
            Self::Employee | Self::Regular | Self::New | Self::Unknown => None,
        }
    }

    fn shipping_surcharge_cents(self, country_kind: CountryKind) -> i32 {
        if self == Self::Employee && country_kind != CountryKind::It {
            EMPLOYEE_NON_IT_SURCHARGE_CENTS
        } else {
            0
        }
    }

    fn tax_override_percent(self, country_kind: CountryKind) -> Option<i32> {
        if self == Self::Vip && country_kind == CountryKind::It {
            Some(VIP_IT_TAX_PERCENT)
        } else {
            None
        }
    }
}

impl CountryKind {
    fn from_raw(value: &str) -> Self {
        match safe(value).as_str() {
            "IT" => Self::It,
            "DE" => Self::De,
            "US" => Self::Us,
            _ => Self::Other,
        }
    }

    fn base_shipping_cents(self) -> i32 {
        match self {
            Self::It => IT_SHIPPING_CENTS,
            Self::De => DE_SHIPPING_CENTS,
            Self::Us => US_SHIPPING_CENTS,
            Self::Other => OTHER_SHIPPING_CENTS,
        }
    }

    fn base_tax_percent(self) -> i32 {
        match self {
            Self::It => IT_TAX_PERCENT,
            Self::De => DE_TAX_PERCENT,
            Self::Us => US_TAX_PERCENT,
            Self::Other => 0,
        }
    }
}

impl CouponKind {
    fn from_raw(value: &str) -> Self {
        match safe(value).as_str() {
            "SAVE10" => Self::Save10,
            "VIPONLY" => Self::VipOnly,
            "BULK" => Self::Bulk,
            "FREESHIP" => Self::FreeShip,
            "TAXFREE" => Self::TaxFree,
            _ => Self::Other,
        }
    }

    fn discount_percent(self, order: &NormalizedOrder) -> i32 {
        match self {
            Self::Save10 if order.subtotal_cents >= SAVE10_THRESHOLD_CENTS => 10,
            Self::VipOnly if order.customer_kind == CustomerKind::Vip => 5,
            Self::Bulk if order.subtotal_cents >= BULK_THRESHOLD_CENTS => 7,
            Self::Save10 | Self::VipOnly | Self::Bulk | Self::FreeShip | Self::TaxFree | Self::Other => 0,
        }
    }

    fn applies_free_shipping(self, discounted_subtotal: i32) -> bool {
        self == Self::FreeShip && discounted_subtotal >= FREESHIP_THRESHOLD_CENTS
    }

    fn tax_override_percent(self, country_kind: CountryKind) -> Option<i32> {
        if self == Self::TaxFree && country_kind != CountryKind::It {
            Some(0)
        } else {
            None
        }
    }
}
