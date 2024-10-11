package testdata

// NoIndexProvided --
func NoIndexProvided() {
	x := []byte{'f', 'o', 'o'}
	y := x[:] // want "Expression is already a slice."
	if len(y) == 3 {
	}
}

// StartindexprovidedNodiagnostic --
func StartindexprovidedNodiagnostic() {
	x := []byte{'f', 'o', 'o'}
	y := x[1:]
	if len(y) == 3 {
	}
}

// EndindexprovidedNodiagnostic --
func EndindexprovidedNodiagnostic() {
	x := []byte{'f', 'o', 'o'}
	y := x[:2]
	if len(y) == 3 {
	}
}

// BothindicesprovidedNodiagnostic --
func BothindicesprovidedNodiagnostic() {
	x := []byte{'f', 'o', 'o'}
	y := x[1:2]
	if len(y) == 3 {
	}
}

// StringSlice --
func StringSlice() {
	x := "foo"
	y := x[:] // want "Expression is already a slice."
	if len(y) == 3 {
	}
}

// SliceFromFunction --
func SliceFromFunction() {
	x := slice()[:] // want "Expression is already a slice."
	if len(x) == 3 {
	}
}

func slice() []string {
	return []string{"bar"}
}
