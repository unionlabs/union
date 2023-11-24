// TODO: Can we delete this package. Honestly pretty insane this is in our database.
package color

import (
	"fmt"
	"os"
	"strings"
)

const (
	ANSIReset  = "\x1b[0m"
	ANSIBright = "\x1b[1m"

	ANSIFgGreen = "\x1b[32m"
	ANSIFgBlue  = "\x1b[34m"
	ANSIFgCyan  = "\x1b[36m"
)

// color the string s with color 'color'
// unless s is already colored
func treat(s string, color string) string {
	if len(s) > 2 && s[:2] == "\x1b[" {
		return s
	}
	return color + s + ANSIReset
}

func treatAll(color string, args ...interface{}) string {
	parts := make([]string, 0, len(args))
	for _, arg := range args {
		parts = append(parts, treat(fmt.Sprintf("%v", arg), color))
	}
	return strings.Join(parts, "")
}

func Green(args ...interface{}) string {
	return treatAll(ANSIFgGreen, args...)
}

func Blue(args ...interface{}) string {
	return treatAll(ANSIFgBlue, args...)
}

func Cyan(args ...interface{}) string {
	return treatAll(ANSIFgCyan, args...)
}

// ColoredBytes takes in the byte that you would like to show as a string and byte
// and will display them in a human readable format.
// If the environment variable TENDERMINT_IAVL_COLORS_ON is set to a non-empty string then different colors will be used for bytes and strings.
func ColoredBytes(data []byte, textColor, bytesColor func(...interface{}) string) string {
	colors := os.Getenv("TENDERMINT_IAVL_COLORS_ON")
	if colors == "" {
		for _, b := range data {
			return string(b)
		}
	}
	s := ""
	for _, b := range data {
		if 0x21 <= b && b < 0x7F {
			s += textColor(string(b))
		} else {
			s += bytesColor(fmt.Sprintf("%02X", b))
		}
	}
	return s
}
