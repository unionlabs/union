package log

import "fmt"

type level byte

const (
	levelDebug level = 1 << iota
	levelInfo
	levelError
)

type filter struct {
	next             Logger
	allowed          level            // XOR'd levels for default case
	initiallyAllowed level            // XOR'd levels for initial case
	allowedKeyvals   map[keyval]level // When key-value match, use this level
}

type keyval struct {
	key   interface{}
	value interface{}
}

// NewFilter wraps next and implements filtering. See the commentary on the
// Option functions for a detailed description of how to configure levels. If
// no options are provided, all leveled log events created with Debug, Info or
// Error helper methods are squelched.
func NewFilter(next Logger, options ...Option) Logger {
	l := &filter{
		next:           next,
		allowedKeyvals: make(map[keyval]level),
	}
	for _, option := range options {
		option(l)
	}
	l.initiallyAllowed = l.allowed
	return l
}

func (l *filter) Info(msg string, keyvals ...interface{}) {
	levelAllowed := l.allowed&levelInfo != 0
	if !levelAllowed {
		return
	}
	l.next.Info(msg, keyvals...)
}

func (l *filter) Debug(msg string, keyvals ...interface{}) {
	levelAllowed := l.allowed&levelDebug != 0
	if !levelAllowed {
		return
	}
	l.next.Debug(msg, keyvals...)
}

func (l *filter) Error(msg string, keyvals ...interface{}) {
	levelAllowed := l.allowed&levelError != 0
	if !levelAllowed {
		return
	}
	l.next.Error(msg, keyvals...)
}

// With implements Logger by constructing a new filter with a keyvals appended
// to the logger.
//
// If custom level was set for a keyval pair using one of the
// Allow*With methods, it is used as the logger's level.
//
// Examples:
//
//	    logger = log.NewFilter(logger, log.AllowError(), log.AllowInfoWith("module", "crypto"))
//			 logger.With("module", "crypto").Info("Hello") # produces "I... Hello module=crypto"
//
//	    logger = log.NewFilter(logger, log.AllowError(),
//					log.AllowInfoWith("module", "crypto"),
//					log.AllowNoneWith("user", "Sam"))
//			 logger.With("module", "crypto", "user", "Sam").Info("Hello") # returns nil
//
//	    logger = log.NewFilter(logger,
//					log.AllowError(),
//					log.AllowInfoWith("module", "crypto"), log.AllowNoneWith("user", "Sam"))
//			 logger.With("user", "Sam").With("module", "crypto").Info("Hello") # produces "I... Hello module=crypto user=Sam"
func (l *filter) With(keyvals ...interface{}) Logger {
	keyInAllowedKeyvals := false

	for i := len(keyvals) - 2; i >= 0; i -= 2 {
		for kv, allowed := range l.allowedKeyvals {
			if keyvals[i] == kv.key {
				keyInAllowedKeyvals = true
				// Example:
				//		logger = log.NewFilter(logger, log.AllowError(), log.AllowInfoWith("module", "crypto"))
				//		logger.With("module", "crypto")
				if keyvals[i+1] == kv.value {
					return &filter{
						next:             l.next.With(keyvals...),
						allowed:          allowed, // set the desired level
						allowedKeyvals:   l.allowedKeyvals,
						initiallyAllowed: l.initiallyAllowed,
					}
				}
			}
		}
	}

	// Example:
	//		logger = log.NewFilter(logger, log.AllowError(), log.AllowInfoWith("module", "crypto"))
	//		logger.With("module", "main")
	if keyInAllowedKeyvals {
		return &filter{
			next:             l.next.With(keyvals...),
			allowed:          l.initiallyAllowed, // return back to initially allowed
			allowedKeyvals:   l.allowedKeyvals,
			initiallyAllowed: l.initiallyAllowed,
		}
	}

	return &filter{
		next:             l.next.With(keyvals...),
		allowed:          l.allowed, // simply continue with the current level
		allowedKeyvals:   l.allowedKeyvals,
		initiallyAllowed: l.initiallyAllowed,
	}
}

//--------------------------------------------------------------------------------

// Option sets a parameter for the filter.
type Option func(*filter)

// AllowLevel returns an option for the given level or error if no option exist
// for such level.
func AllowLevel(lvl string) (Option, error) {
	switch lvl {
	case "debug":
		return AllowDebug(), nil
	case "info":
		return AllowInfo(), nil
	case "error":
		return AllowError(), nil
	case "none":
		return AllowNone(), nil
	default:
		return nil, fmt.Errorf("expected either \"info\", \"debug\", \"error\" or \"none\" level, given %s", lvl)
	}
}

// AllowAll is an alias for AllowDebug.
func AllowAll() Option {
	return AllowDebug()
}

// AllowDebug allows error, info and debug level log events to pass.
func AllowDebug() Option {
	return allowed(levelError | levelInfo | levelDebug)
}

// AllowInfo allows error and info level log events to pass.
func AllowInfo() Option {
	return allowed(levelError | levelInfo)
}

// AllowError allows only error level log events to pass.
func AllowError() Option {
	return allowed(levelError)
}

// AllowNone allows no leveled log events to pass.
func AllowNone() Option {
	return allowed(0)
}

func allowed(allowed level) Option {
	return func(l *filter) { l.allowed = allowed }
}

// AllowDebugWith allows error, info and debug level log events to pass for a specific key value pair.
func AllowDebugWith(key interface{}, value interface{}) Option {
	return func(l *filter) { l.allowedKeyvals[keyval{key, value}] = levelError | levelInfo | levelDebug }
}

// AllowInfoWith allows error and info level log events to pass for a specific key value pair.
func AllowInfoWith(key interface{}, value interface{}) Option {
	return func(l *filter) { l.allowedKeyvals[keyval{key, value}] = levelError | levelInfo }
}

// AllowErrorWith allows only error level log events to pass for a specific key value pair.
func AllowErrorWith(key interface{}, value interface{}) Option {
	return func(l *filter) { l.allowedKeyvals[keyval{key, value}] = levelError }
}

// AllowNoneWith allows no leveled log events to pass for a specific key value pair.
func AllowNoneWith(key interface{}, value interface{}) Option {
	return func(l *filter) { l.allowedKeyvals[keyval{key, value}] = 0 }
}
