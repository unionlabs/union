package cli

import (
	"flag"
	"io"
	"log"
	"os"

	"github.com/google/subcommands"
)

// Command is a base for all subcommands.
type Command struct {
	Log *log.Logger
}

// NewBaseCommand builds a new base command for the named tool.
func NewBaseCommand(name string) Command {
	return Command{
		Log: log.New(os.Stderr, name+": ", 0),
	}
}

// SetFlags is a stub implementation of the SetFlags methods that does nothing.
func (Command) SetFlags(f *flag.FlagSet) {}

// UsageError logs a usage error and returns a suitable exit code.
func (c Command) UsageError(format string, args ...interface{}) subcommands.ExitStatus {
	c.Log.Printf(format, args...)
	return subcommands.ExitUsageError
}

// Fail logs an error message and returns a failing exit code.
func (c Command) Fail(format string, args ...interface{}) subcommands.ExitStatus {
	c.Log.Printf(format, args...)
	return subcommands.ExitFailure
}

// Error logs err and returns a failing exit code.
func (c Command) Error(err error) subcommands.ExitStatus {
	return c.Fail(err.Error())
}

// CheckClose closes cl. On error it logs and writes to the status pointer.
// Intended for deferred Close() calls.
func (c Command) CheckClose(statusp *subcommands.ExitStatus, cl io.Closer) {
	if err := cl.Close(); err != nil {
		*statusp = c.Error(err)
	}
}
