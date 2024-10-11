package term_test

import (
	"errors"
	"os"

	"github.com/go-kit/kit/log"
	"github.com/go-kit/kit/log/term"
)

func ExampleNewLogger_redErrors() {
	// Color errors red
	colorFn := func(keyvals ...interface{}) term.FgBgColor {
		for i := 1; i < len(keyvals); i += 2 {
			if _, ok := keyvals[i].(error); ok {
				return term.FgBgColor{Fg: term.White, Bg: term.Red}
			}
		}
		return term.FgBgColor{}
	}

	logger := term.NewLogger(os.Stdout, log.NewLogfmtLogger, colorFn)

	logger.Log("msg", "default color", "err", nil)
	logger.Log("msg", "colored because of error", "err", errors.New("coloring error"))
}

func ExampleNewLogger_levelColors() {
	// Color by level value
	colorFn := func(keyvals ...interface{}) term.FgBgColor {
		for i := 0; i < len(keyvals)-1; i += 2 {
			if keyvals[i] != "level" {
				continue
			}
			switch keyvals[i+1] {
			case "debug":
				return term.FgBgColor{Fg: term.DarkGray}
			case "info":
				return term.FgBgColor{Fg: term.Gray}
			case "warn":
				return term.FgBgColor{Fg: term.Yellow}
			case "error":
				return term.FgBgColor{Fg: term.Red}
			case "crit":
				return term.FgBgColor{Fg: term.Gray, Bg: term.DarkRed}
			default:
				return term.FgBgColor{}
			}
		}
		return term.FgBgColor{}
	}

	logger := term.NewLogger(os.Stdout, log.NewJSONLogger, colorFn)

	logger.Log("level", "warn", "msg", "yellow")
	logger.Log("level", "debug", "msg", "dark gray")
}
