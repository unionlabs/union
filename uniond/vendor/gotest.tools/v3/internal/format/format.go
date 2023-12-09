package format // import "gotest.tools/v3/internal/format"

import "fmt"

// Message accepts a msgAndArgs varargs and formats it using fmt.Sprintf
func Message(msgAndArgs ...interface{}) string {
	switch len(msgAndArgs) {
	case 0:
		return ""
	case 1:
		return fmt.Sprintf("%v", msgAndArgs[0])
	default:
		return fmt.Sprintf(msgAndArgs[0].(string), msgAndArgs[1:]...)
	}
}

// WithCustomMessage accepts one or two messages and formats them appropriately
func WithCustomMessage(source string, msgAndArgs ...interface{}) string {
	custom := Message(msgAndArgs...)
	switch {
	case custom == "":
		return source
	case source == "":
		return custom
	}
	return fmt.Sprintf("%s: %s", source, custom)
}
