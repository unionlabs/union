package assert_test

import (
	"fmt"
	"regexp"
	"testing"

	"gotest.tools/v3/assert"
	"gotest.tools/v3/assert/cmp"
)

var t = &testing.T{}

func ExampleAssert_customComparison() {
	regexPattern := func(value string, pattern string) cmp.Comparison {
		return func() cmp.Result {
			re := regexp.MustCompile(pattern)
			if re.MatchString(value) {
				return cmp.ResultSuccess
			}
			return cmp.ResultFailure(
				fmt.Sprintf("%q did not match pattern %q", value, pattern))
		}
	}
	assert.Assert(t, regexPattern("12345.34", `\d+.\d\d`))
}
