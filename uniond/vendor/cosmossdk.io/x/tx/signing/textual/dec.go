package textual

import (
	"context"
	"fmt"
	"math/big"
	"strings"

	"google.golang.org/protobuf/reflect/protoreflect"

	"cosmossdk.io/math"
)

// NewDecValueRenderer returns a ValueRenderer for encoding math.Dec cosmos
// scalars.
func NewDecValueRenderer() ValueRenderer {
	return decValueRenderer{}
}

type decValueRenderer struct{}

func (vr decValueRenderer) Format(_ context.Context, v protoreflect.Value) ([]Screen, error) {
	decStr := v.String()

	// If the decimal doesn't contain a point, we assume it's a value formatted using the legacy
	// `math.Dec`. So we try to parse it as an integer and then convert it to a
	// decimal.
	if !strings.Contains(decStr, ".") {
		parsedInt, ok := new(big.Int).SetString(decStr, 10)
		if !ok {
			return nil, fmt.Errorf("invalid decimal: %s", decStr)
		}

		// We assume the decimal has 18 digits of precision.
		decStr = math.LegacyNewDecFromBigIntWithPrec(parsedInt, math.LegacyPrecision).String()
	}

	formatted, err := math.FormatDec(decStr)
	if err != nil {
		return nil, err
	}
	return []Screen{{Content: formatted}}, nil
}

func (vr decValueRenderer) Parse(_ context.Context, screens []Screen) (protoreflect.Value, error) {
	if n := len(screens); n != 1 {
		return nilValue, fmt.Errorf("expected 1 screen, got: %d", n)
	}

	parsed, err := parseDec(screens[0].Content)
	if err != nil {
		return nilValue, err
	}

	return protoreflect.ValueOfString(parsed), nil
}

func parseDec(v string) (string, error) {
	parts := strings.Split(v, ".")
	if len(parts) > 2 {
		return "", fmt.Errorf("invalid decimal: too many points in %s", v)
	}

	intPart, err := parseInt(parts[0])
	if err != nil {
		return "", err
	}

	if len(parts) == 1 {
		return intPart, nil
	}

	return intPart + "." + parts[1], nil
}
