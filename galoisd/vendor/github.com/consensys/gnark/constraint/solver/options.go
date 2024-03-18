package solver

import (
	"fmt"
	"runtime"

	"github.com/consensys/gnark/logger"
	"github.com/rs/zerolog"
)

// Option defines option for altering the behavior of a constraint system
// solver (Solve() method). See the descriptions of functions returning instances
// of this type for implemented options.
type Option func(*Config) error

// Config is the configuration for the solver with the options applied.
type Config struct {
	HintFunctions map[HintID]Hint // defaults to all built-in hint functions
	Logger        zerolog.Logger  // defaults to gnark.Logger
	NbTasks       int             // defaults to runtime.NumCPU()
}

// WithHints is a solver option that specifies additional hint functions to be used
// by the constraint solver.
func WithHints(hintFunctions ...Hint) Option {
	log := logger.Logger()
	return func(opt *Config) error {
		// it is an error to register hint function several times, but as the
		// prover already checks it then omit here.
		for _, h := range hintFunctions {
			uuid := GetHintID(h)
			if _, ok := opt.HintFunctions[uuid]; ok {
				log.Warn().Int("hintID", int(uuid)).Str("name", GetHintName(h)).Msg("duplicate hint function")
			} else {
				opt.HintFunctions[uuid] = h
			}
		}
		return nil
	}
}

// OverrideHint forces the solver to use provided hint function for given id.
func OverrideHint(id HintID, f Hint) Option {
	return func(opt *Config) error {
		opt.HintFunctions[id] = f
		return nil
	}
}

// WithLogger is a prover option that specifies zerolog.Logger as a destination for the
// logs printed by api.Println(). By default, uses gnark/logger.
// zerolog.Nop() will disable logging
func WithLogger(l zerolog.Logger) Option {
	return func(opt *Config) error {
		opt.Logger = l
		return nil
	}
}

// WithNbTasks sets the number of parallel workers to use for the solver. If not
// set, then the number of workers is set to runtime.NumCPU().
//
// The option may be useful for debugging the solver behaviour and restricting
// the CPU usage. Note that this is not hard limit - in case the solver calls a
// hint function which may create more goroutines, the number of goroutines may
// exceed the limit.
func WithNbTasks(nbTasks int) Option {
	return func(opt *Config) error {
		if nbTasks <= 0 {
			return fmt.Errorf("invalid number of threads: %d", nbTasks)
		}
		if nbTasks > 512 {
			// limit the number of tasks to 512. This is to avoid possible
			// saturation of the runtime scheduler.
			nbTasks = 512
		}
		opt.NbTasks = nbTasks
		return nil
	}
}

// NewConfig returns a default SolverConfig with given prover options opts applied.
func NewConfig(opts ...Option) (Config, error) {
	log := logger.Logger()
	opt := Config{Logger: log}
	opt.HintFunctions = cloneHintRegistry()
	opt.NbTasks = runtime.NumCPU()
	for _, option := range opts {
		if err := option(&opt); err != nil {
			return Config{}, err
		}
	}
	return opt, nil
}
