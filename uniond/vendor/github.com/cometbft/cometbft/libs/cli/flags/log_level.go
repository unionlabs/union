package flags

import (
	"errors"
	"fmt"
	"strings"

	"github.com/cometbft/cometbft/libs/log"
)

const (
	defaultLogLevelKey = "*"
)

// ParseLogLevel parses complex log level - comma-separated
// list of module:level pairs with an optional *:level pair (* means
// all other modules).
//
// Example:
//
//	ParseLogLevel("consensus:debug,mempool:debug,*:error", log.NewTMLogger(os.Stdout), "info")
func ParseLogLevel(lvl string, logger log.Logger, defaultLogLevelValue string) (log.Logger, error) {
	if lvl == "" {
		return nil, errors.New("empty log level")
	}

	l := lvl

	// prefix simple one word levels (e.g. "info") with "*"
	if !strings.Contains(l, ":") {
		l = defaultLogLevelKey + ":" + l
	}

	options := make([]log.Option, 0)

	isDefaultLogLevelSet := false
	var option log.Option
	var err error

	list := strings.Split(l, ",")
	for _, item := range list {
		moduleAndLevel := strings.Split(item, ":")

		if len(moduleAndLevel) != 2 {
			return nil, fmt.Errorf("expected list in a form of \"module:level\" pairs, given pair %s, list %s", item, list)
		}

		module := moduleAndLevel[0]
		level := moduleAndLevel[1]

		if module == defaultLogLevelKey {
			option, err = log.AllowLevel(level)
			if err != nil {
				return nil, fmt.Errorf("failed to parse default log level (pair %s, list %s): %w", item, l, err)
			}
			options = append(options, option)
			isDefaultLogLevelSet = true
		} else {
			switch level {
			case "debug":
				option = log.AllowDebugWith("module", module)
			case "info":
				option = log.AllowInfoWith("module", module)
			case "error":
				option = log.AllowErrorWith("module", module)
			case "none":
				option = log.AllowNoneWith("module", module)
			default:
				return nil,
					fmt.Errorf("expected either \"info\", \"debug\", \"error\" or \"none\" log level, given %s (pair %s, list %s)",
						level,
						item,
						list)
			}
			options = append(options, option)

		}
	}

	// if "*" is not provided, set default global level
	if !isDefaultLogLevelSet {
		option, err = log.AllowLevel(defaultLogLevelValue)
		if err != nil {
			return nil, err
		}
		options = append(options, option)
	}

	return log.NewFilter(logger, options...), nil
}
