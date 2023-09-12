package ratelimit

import (
	"strings"

	"golang.org/x/text/cases"
	"golang.org/x/text/language"
)

// Reference:
// https://github.com/getsentry/relay/blob/0424a2e017d193a93918053c90cdae9472d164bf/relay-common/src/constants.rs#L116-L127

// Category classifies supported payload types that can be ingested by Sentry
// and, therefore, rate limited.
type Category string

// Known rate limit categories. As a special case, the CategoryAll applies to
// all known payload types.
const (
	CategoryAll         Category = ""
	CategoryError       Category = "error"
	CategoryTransaction Category = "transaction"
)

// knownCategories is the set of currently known categories. Other categories
// are ignored for the purpose of rate-limiting.
var knownCategories = map[Category]struct{}{
	CategoryAll:         {},
	CategoryError:       {},
	CategoryTransaction: {},
}

// String returns the category formatted for debugging.
func (c Category) String() string {
	switch c {
	case "":
		return "CategoryAll"
	default:
		caser := cases.Title(language.English)
		rv := "Category"
		for _, w := range strings.Fields(string(c)) {
			rv += caser.String(w)
		}
		return rv
	}
}
