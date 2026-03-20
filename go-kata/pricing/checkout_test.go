package pricing

import "testing"

func TestCalculateTotalCents(t *testing.T) {
	tests := []struct {
		name     string
		order    Order
		expected int
	}{
		// README examples (behavior preservation)
		{"regular IT no coupon", Order{"regular", 10000, "IT", "", false}, 12900},
		{"premium DE SAVE10", Order{"premium", 10000, "DE", "SAVE10", false}, 10420},
		{"vip IT VIPONLY", Order{"vip", 18000, "IT", "VIPONLY", false}, 17980},

		// Customer base discounts
		{"employee DE no coupon", Order{"employee", 10000, "DE", "", false}, 9730},
		{"employee IT no coupon", Order{"employee", 10000, "IT", "", false}, 9240},
		{"new IT no coupon", Order{"new", 10000, "IT", "", false}, 12900},
		{"premium low subtotal DE", Order{"premium", 5000, "DE", "", false}, 6552},

		// VIP free shipping
		{"vip IT free shipping threshold", Order{"vip", 20000, "IT", "", false}, 20400},
		{"vip IT below free shipping", Order{"vip", 10000, "IT", "", false}, 10900},

		// Premium free shipping
		{"premium IT free shipping threshold", Order{"premium", 25000, "IT", "", false}, 27450},

		// Coupons
		{"SAVE10 below threshold no discount", Order{"premium", 4000, "DE", "SAVE10", false}, 5422},
		{"BULK large order", Order{"regular", 25000, "DE", "BULK", false}, 28567},
		{"VIPONLY on non-vip no discount", Order{"premium", 10000, "IT", "VIPONLY", false}, 11680},
		{"FREESHIP coupon", Order{"regular", 10000, "IT", "FREESHIP", false}, 12200},
		{"TAXFREE DE", Order{"regular", 10000, "DE", "TAXFREE", false}, 10900},
		{"TAXFREE IT ignored", Order{"regular", 10000, "IT", "TAXFREE", false}, 12900},

		// Black Friday
		{"regular BF US", Order{"regular", 10000, "US", "", true}, 11965},
		{"employee BF no bonus", Order{"employee", 10000, "IT", "", true}, 9240},
		{"vip BF IT", Order{"vip", 10000, "IT", "", true}, 10300},

		// Discount cap
		{"employee SAVE10 near cap", Order{"employee", 10000, "IT", "SAVE10", false}, 8020},

		// Unknown / edge cases
		{"unknown customer type", Order{"unknown", 10000, "IT", "", false}, 12900},
		{"empty country default shipping", Order{"regular", 10000, "FR", "", false}, 12500},
		{"empty strings", Order{"", 10000, "", "", false}, 12500},

		// Partner — new requirement
		{"partner IT below free ship threshold", Order{"partner", 15000, "IT", "", false}, 16804},
		{"partner IT free ship threshold", Order{"partner", 20000, "IT", "", false}, 21472},
		{"partner IT PARTNER5 coupon", Order{"partner", 15000, "IT", "PARTNER5", false}, 15889},
		{"partner IT PARTNER5 below subtotal threshold", Order{"partner", 10000, "IT", "PARTNER5", false}, 11436},
		{"partner IT BF gets +3 not +5", Order{"partner", 15000, "IT", "", true}, 16255},
		{"PARTNER5 on non-partner no discount", Order{"regular", 15000, "IT", "PARTNER5", false}, 19000},
	}

	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			got := CalculateTotalCents(tt.order)
			if got != tt.expected {
				t.Errorf("expected %d, got %d", tt.expected, got)
			}
		})
	}
}
