package genopenapi

import (
	"testing"
)

func TestCycle(t *testing.T) {
	for _, tt := range []struct {
		max     int
		attempt int
		e       bool
	}{
		{
			max:     3,
			attempt: 3,
			e:       true,
		},
		{
			max:     5,
			attempt: 6,
		},
		{
			max:     1000,
			attempt: 1001,
		},
	} {

		c := newCycleChecker(tt.max)
		var final bool
		for i := 0; i < tt.attempt; i++ {
			final = c.Check("a")
			if !final {
				break
			}
		}

		if final != tt.e {
			t.Errorf("got: %t wanted: %t", final, tt.e)
		}
	}
}
