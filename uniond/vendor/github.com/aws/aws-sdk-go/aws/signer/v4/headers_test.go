//go:build go1.7
// +build go1.7

package v4

import "testing"

func TestAllowedQueryHoisting(t *testing.T) {
	cases := map[string]struct {
		Header      string
		ExpectHoist bool
	}{
		"object-lock": {
			Header:      "X-Amz-Object-Lock-Mode",
			ExpectHoist: false,
		},
		"s3 metadata": {
			Header:      "X-Amz-Meta-SomeName",
			ExpectHoist: false,
		},
		"another header": {
			Header:      "X-Amz-SomeOtherHeader",
			ExpectHoist: true,
		},
		"non X-AMZ header": {
			Header:      "X-SomeOtherHeader",
			ExpectHoist: false,
		},
	}

	for name, c := range cases {
		t.Run(name, func(t *testing.T) {
			if e, a := c.ExpectHoist, allowedQueryHoisting.IsValid(c.Header); e != a {
				t.Errorf("expect hoist %v, was %v", e, a)
			}
		})
	}
}
