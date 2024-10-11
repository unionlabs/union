package types

import (
	"regexp"
)

// IsAlphaNumeric defines a regular expression for matching against alphanumeric
// values.
var IsAlphaNumeric = regexp.MustCompile(`^[a-zA-Z0-9]+$`).MatchString
