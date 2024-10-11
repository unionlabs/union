package promptui

import "testing"

func TestStyler(t *testing.T) {
	t.Run("renders a single code", func(t *testing.T) {
		red := Styler(FGRed)("hi")
		expected := "\033[31mhi\033[0m"
		if red != expected {
			t.Errorf("style did not match: %s != %s", red, expected)
		}
	})

	t.Run("combines multiple codes", func(t *testing.T) {
		boldRed := Styler(FGRed, FGBold)("hi")
		expected := "\033[31;1mhi\033[0m"
		if boldRed != expected {
			t.Errorf("style did not match: %s != %s", boldRed, expected)
		}
	})

	t.Run("should not repeat reset codes for nested styles", func(t *testing.T) {
		red := Styler(FGRed)("hi")
		boldRed := Styler(FGBold)(red)
		expected := "\033[1m\033[31mhi\033[0m"
		if boldRed != expected {
			t.Errorf("style did not match: %s != %s", boldRed, expected)
		}
	})
}
