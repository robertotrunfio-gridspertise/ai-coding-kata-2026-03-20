package pricing

import "strings"

type Order struct {
	CustomerType  string
	SubtotalCents int
	Country       string
	CouponCode    string
	BlackFriday   bool
}

type customerConfig struct {
	baseDiscount      func(subtotal int) int
	freeShipThreshold int                    // discounted subtotal threshold for free shipping; 0 = none
	extraShip         func(country string) int // nil = no extra shipping
	blackFridayBonus  *int                   // nil = default 5; pointer to override
}

func intPtr(n int) *int { return &n }

var customerRules = map[string]customerConfig{
	"vip": {
		baseDiscount:      func(_ int) int { return 15 },
		freeShipThreshold: 15000,
	},
	"premium": {
		baseDiscount: func(s int) int {
			if s >= 10000 {
				return 10
			}
			return 5
		},
		freeShipThreshold: 20000,
	},
	"employee": {
		baseDiscount: func(_ int) int { return 30 },
		extraShip: func(c string) int {
			if c != "IT" {
				return 500
			}
			return 0
		},
		blackFridayBonus: intPtr(0),
	},
	"regular": {baseDiscount: func(_ int) int { return 0 }},
	"new":     {baseDiscount: func(_ int) int { return 0 }},
	"partner": {
		baseDiscount:      func(_ int) int { return 12 },
		freeShipThreshold: 15000,
		blackFridayBonus:  intPtr(3),
	},
}

var couponRules = map[string]func(customerType string, subtotal int) int{
	"SAVE10":  func(_ string, s int) int { if s >= 5000 { return 10 }; return 0 },
	"VIPONLY": func(ct string, _ int) int { if ct == "vip" { return 5 }; return 0 },
	"BULK":    func(_ string, s int) int { if s >= 20000 { return 7 }; return 0 },
	"PARTNER5": func(ct string, s int) int {
		if ct == "partner" && s >= 12000 {
			return 5
		}
		return 0
	},
}

var countryShipping = map[string]int{
	"IT": 700,
	"DE": 900,
	"US": 1500,
}

var countryTax = map[string]int{
	"IT": 22,
	"DE": 19,
	"US": 7,
}

const (
	defaultShipping = 2500
	maxDiscount     = 40
	defaultBFBonus  = 5
)

func CalculateTotalCents(order Order) int {
	subtotal := order.SubtotalCents
	ct := safe(order.CustomerType)
	country := safe(order.Country)
	coupon := safe(order.CouponCode)

	customer := customerRules[ct]

	// Discount
	discount := 0
	if customer.baseDiscount != nil {
		discount = customer.baseDiscount(subtotal)
	}
	if couponFn, ok := couponRules[coupon]; ok {
		discount += couponFn(ct, subtotal)
	}
	if order.BlackFriday {
		bonus := defaultBFBonus
		if customer.blackFridayBonus != nil {
			bonus = *customer.blackFridayBonus
		}
		discount += bonus
	}
	if discount > maxDiscount {
		discount = maxDiscount
	}

	discountedSubtotal := subtotal * (100 - discount) / 100

	// Shipping
	shipping, ok := countryShipping[country]
	if !ok {
		shipping = defaultShipping
	}
	if order.BlackFriday && country == "US" {
		shipping += 300
	}
	if coupon == "FREESHIP" && discountedSubtotal >= 8000 {
		shipping = 0
	}
	if customer.freeShipThreshold > 0 && discountedSubtotal >= customer.freeShipThreshold {
		shipping = 0
	}
	if customer.extraShip != nil {
		shipping += customer.extraShip(country)
	}

	// Tax
	tax := countryTax[country]
	if ct == "vip" && country == "IT" {
		tax = 20
	}
	if coupon == "TAXFREE" && country != "IT" {
		tax = 0
	}

	taxCents := discountedSubtotal * tax / 100
	total := discountedSubtotal + shipping + taxCents

	if total < 0 {
		return 0
	}
	return total
}

func safe(value string) string {
	return strings.TrimSpace(value)
}
