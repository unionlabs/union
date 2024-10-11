package exec

import (
	"context"
	"fmt"
	"os"
	osexec "os/exec"
)

// Command executes a shell command.
func Command(ctx context.Context, args ...string) error {
	_, err := CommandOutput(ctx, args...)
	return err
}

// CommandOutput executes a shell command and returns the command's output.
func CommandOutput(ctx context.Context, args ...string) ([]byte, error) {
	//nolint: gosec
	// G204: Subprocess launched with a potential tainted input or cmd arguments
	cmd := osexec.CommandContext(ctx, args[0], args[1:]...)
	out, err := cmd.CombinedOutput()
	switch err := err.(type) {
	case nil:
		return out, nil
	case *osexec.ExitError:
		return nil, fmt.Errorf("failed to run %q:\n%v", args, string(out))
	default:
		return nil, err
	}
}

// CommandVerbose executes a shell command while displaying its output.
func CommandVerbose(ctx context.Context, args ...string) error {
	//nolint: gosec
	// G204: Subprocess launched with a potential tainted input or cmd arguments
	cmd := osexec.CommandContext(ctx, args[0], args[1:]...)
	cmd.Stdout = os.Stdout
	cmd.Stderr = os.Stderr
	return cmd.Run()
}
