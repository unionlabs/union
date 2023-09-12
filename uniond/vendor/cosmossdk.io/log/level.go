package log

import (
	"errors"
	"fmt"
	"strings"

	"github.com/rs/zerolog"
)

const defaultLogLevelKey = "*"

// FilterFunc is a function that returns true if the log level is filtered for the given key
// When the filter returns true, the log entry is discarded.
type FilterFunc func(key, level string) bool

// ParseLogLevel parses complex log level
// A comma-separated list of module:level pairs with an optional *:level pair
// (* means all other modules).
//
// Example:
// ParseLogLevel("consensus:debug,mempool:debug,*:error")
//
// This function attempts to keep the same behavior as the CometBFT ParseLogLevel
// However the level `none` is replaced by `disabled`.
func ParseLogLevel(levelStr string) (FilterFunc, error) {
	if levelStr == "" {
		return nil, errors.New("empty log level")
	}

	// prefix simple one word levels (e.g. "info") with "*"
	l := levelStr
	if !strings.Contains(l, ":") {
		l = defaultLogLevelKey + ":" + l
	}

	// parse and validate the levels
	filterMap := make(map[string]zerolog.Level)
	list := strings.Split(l, ",")
	for _, item := range list {
		moduleAndLevel := strings.Split(item, ":")
		if len(moduleAndLevel) != 2 {
			return nil, fmt.Errorf("expected list in a form of \"module:level\" pairs, given pair %s, list %s", item, list)
		}

		module := moduleAndLevel[0]
		level := moduleAndLevel[1]

		if _, ok := filterMap[module]; ok {
			return nil, fmt.Errorf("duplicate module %s in log level list %s", module, list)
		}

		zllevel, err := zerolog.ParseLevel(level)
		if err != nil {
			return nil, fmt.Errorf("invalid log level %s in log level list %s", level, list)
		}

		filterMap[module] = zllevel
	}

	filterFunc := func(key, lvl string) bool {
		zllevel, ok := filterMap[key]
		if !ok { // no level filter for this key
			// check if there is a default level filter
			zllevel, ok = filterMap[defaultLogLevelKey]
			if !ok {
				return false
			}
		}

		zllvl, err := zerolog.ParseLevel(lvl)
		if err != nil {
			panic(err)
		}

		return zllvl < zllevel
	}

	return filterFunc, nil
}
