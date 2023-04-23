// +build !windows

package promptui

import "github.com/chzyer/readline"

var (
	// KeyBackspace is the default key for deleting input text.
	KeyBackspace rune = readline.CharBackspace
)
