package testdata

// Equal --
func Equal() {
	x := []string{"a"}
	if len(x) == len(x) { // want "Boolean expression has identical expressions on both sides. The result is always true."
	}
}

// NotEqual --
func NotEqual() {
	x := []string{"a"}
	if len(x) != len(x) { // want "Boolean expression has identical expressions on both sides. The result is always true."
	}
}

// GreaterThanOrEqual --
func GreaterThanOrEqual() {
	x := []string{"a"}
	if len(x) >= len(x) { // want "Boolean expression has identical expressions on both sides. The result is always true."
	}
}

// LessThanOrEqual --
func LessThanOrEqual() {
	x := []string{"a"}
	if len(x) <= len(x) { // want "Boolean expression has identical expressions on both sides. The result is always true."
	}
}

// GreaterThan --
func GreaterThan() {
	x := []string{"a"}
	if len(x) > len(x) { // want "Boolean expression has identical expressions on both sides. The result is always false."
	}
}

// LessThan --
func LessThan() {
	x := []string{"a"}
	if len(x) < len(x) { // want "Boolean expression has identical expressions on both sides. The result is always false."
	}
}
