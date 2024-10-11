//go:build clock_skew
// +build clock_skew

package time

import (
	"fmt"
	"os"
	"time"
)

var clockSkew time.Duration

// Now returns the current time in UTC with no monotonic component.
func Now() time.Time {
	return Canonical(time.Now().Add(clockSkew))
}

func init() {
	skewStr := os.Getenv("COMETBFT_CLOCK_SKEW")
	if len(skewStr) == 0 {
		return
	}
	skew, err := time.ParseDuration(skewStr)
	if err != nil {
		panic(fmt.Sprintf("contents of env variable COMETBFT_CLOCK_SKEW (%q) must be empty or a duration expression", skewStr))
	}
	clockSkew = skew
}
