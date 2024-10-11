package ratelimit

import "testing"

func TestCategoryString(t *testing.T) {
	tests := []struct {
		Category
		want string
	}{
		{CategoryAll, "CategoryAll"},
		{CategoryError, "CategoryError"},
		{CategoryTransaction, "CategoryTransaction"},
		{Category("unknown"), "CategoryUnknown"},
		{Category("two words"), "CategoryTwoWords"},
	}
	for _, tt := range tests {
		tt := tt
		t.Run(tt.want, func(t *testing.T) {
			got := tt.Category.String()
			if got != tt.want {
				t.Errorf("got %q, want %q", got, tt.want)
			}
		})
	}
}
