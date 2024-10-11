package schema

import "testing"

func TestValidateName(t *testing.T) {
	tests := []struct {
		name  string
		valid bool
	}{
		{"", false},
		{"a", true},
		{"A", true},
		{"_", true},
		{"abc123_def789", true},
		{"0", false},
		{"a0", true},
		{"a_", true},
		{"$a", false},
		{"a b", false},
		{"pretty_unnecessarily_long_but_valid_name", true},
		{"totally_unnecessarily_long_and_invalid_name_sdgkhwersdglkhweriqwery3258", false},
	}

	for _, test := range tests {
		t.Run(test.name, func(t *testing.T) {
			if ValidateName(test.name) != test.valid {
				t.Errorf("expected %v for name %q", test.valid, test.name)
			}
		})
	}
}
