package math

import (
	"errors"
	"fmt"
	"math"
	"strconv"
	"strings"
)

// Fraction defined in terms of a numerator divided by a denominator in uint64
// format. Fraction must be positive.
type Fraction struct {
	// The portion of the denominator in the faction, e.g. 2 in 2/3.
	Numerator uint64 `json:"numerator"`
	// The value by which the numerator is divided, e.g. 3 in 2/3.
	Denominator uint64 `json:"denominator"`
}

func (fr Fraction) String() string {
	return fmt.Sprintf("%d/%d", fr.Numerator, fr.Denominator)
}

// ParseFractions takes the string of a fraction as input i.e "2/3" and converts this
// to the equivalent fraction else returns an error. The format of the string must be
// one number followed by a slash (/) and then the other number.
func ParseFraction(f string) (Fraction, error) {
	o := strings.Split(f, "/")
	if len(o) != 2 {
		return Fraction{}, errors.New("incorrect formating: should have a single slash i.e. \"1/3\"")
	}
	numerator, err := strconv.ParseUint(o[0], 10, 64)
	if err != nil {
		return Fraction{}, fmt.Errorf("incorrect formatting, err: %w", err)
	}

	denominator, err := strconv.ParseUint(o[1], 10, 64)
	if err != nil {
		return Fraction{}, fmt.Errorf("incorrect formatting, err: %w", err)
	}
	if denominator == 0 {
		return Fraction{}, errors.New("denominator can't be 0")
	}
	if numerator > math.MaxInt64 || denominator > math.MaxInt64 {
		return Fraction{}, fmt.Errorf("value overflow, numerator and denominator must be less than %d", int64(math.MaxInt64))
	}
	return Fraction{Numerator: numerator, Denominator: denominator}, nil
}
