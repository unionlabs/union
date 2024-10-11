package schema

import "regexp"

// NameFormat is the regular expression that a name must match.
// A name must start with a letter or underscore and can only contain letters, numbers, and underscores.
// A name must be at least one character long and can be at most 64 characters long.
const NameFormat = `^[a-zA-Z_][a-zA-Z0-9_]{0,63}$`

var nameRegex = regexp.MustCompile(NameFormat)

// ValidateName checks if the given name is a valid name conforming to NameFormat.
func ValidateName(name string) bool {
	return nameRegex.MatchString(name)
}
