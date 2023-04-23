package logger

import (
	"fmt"
)

var debugging = false

func Debug(format string, args ...interface{}) {
	if debugging {
		fmt.Printf(format, args...)
	}
}
