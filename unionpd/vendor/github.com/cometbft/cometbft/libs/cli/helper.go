package cli

import (
	"bytes"
	"fmt"
	"io"
	"os"
	"path/filepath"

	"github.com/spf13/cobra"
)

// WriteConfigVals writes a toml file with the given values.
// It returns an error if writing was impossible.
func WriteConfigVals(dir string, vals map[string]string) error {
	data := ""
	for k, v := range vals {
		data += fmt.Sprintf("%s = \"%s\"\n", k, v)
	}
	cfile := filepath.Join(dir, "config.toml")
	return os.WriteFile(cfile, []byte(data), 0600)
}

// RunWithArgs executes the given command with the specified command line args
// and environmental variables set. It returns any error returned from cmd.Execute()
func RunWithArgs(cmd Executable, args []string, env map[string]string) error {
	oargs := os.Args
	oenv := map[string]string{}
	// defer returns the environment back to normal
	defer func() {
		os.Args = oargs
		for k, v := range oenv {
			os.Setenv(k, v)
		}
	}()

	// set the args and env how we want them
	os.Args = args
	for k, v := range env {
		// backup old value if there, to restore at end
		oenv[k] = os.Getenv(k)
		err := os.Setenv(k, v)
		if err != nil {
			return err
		}
	}

	// and finally run the command
	return cmd.Execute()
}

// RunCaptureWithArgs executes the given command with the specified command
// line args and environmental variables set. It returns string fields
// representing output written to stdout and stderr, additionally any error
// from cmd.Execute() is also returned
func RunCaptureWithArgs(cmd Executable, args []string, env map[string]string) (stdout, stderr string, err error) {
	oldout, olderr := os.Stdout, os.Stderr // keep backup of the real stdout
	rOut, wOut, _ := os.Pipe()
	rErr, wErr, _ := os.Pipe()
	os.Stdout, os.Stderr = wOut, wErr
	defer func() {
		os.Stdout, os.Stderr = oldout, olderr // restoring the real stdout
	}()

	// copy the output in a separate goroutine so printing can't block indefinitely
	copyStd := func(reader *os.File) *(chan string) {
		stdC := make(chan string)
		go func() {
			var buf bytes.Buffer
			// io.Copy will end when we call reader.Close() below
			io.Copy(&buf, reader) //nolint:errcheck //ignore error
			stdC <- buf.String()
		}()
		return &stdC
	}
	outC := copyStd(rOut)
	errC := copyStd(rErr)

	// now run the command
	err = RunWithArgs(cmd, args, env)

	// and grab the stdout to return
	wOut.Close()
	wErr.Close()
	stdout = <-*outC
	stderr = <-*errC
	return stdout, stderr, err
}

// NewCompletionCmd returns a cobra.Command that generates bash and zsh
// completion scripts for the given root command. If hidden is true, the
// command will not show up in the root command's list of available commands.
func NewCompletionCmd(rootCmd *cobra.Command, hidden bool) *cobra.Command {
	flagZsh := "zsh"
	cmd := &cobra.Command{
		Use:   "completion",
		Short: "Generate shell completion scripts",
		Long: fmt.Sprintf(`Generate Bash and Zsh completion scripts and print them to STDOUT.

Once saved to file, a completion script can be loaded in the shell's
current session as shown:

   $ . <(%s completion)

To configure your bash shell to load completions for each session add to
your $HOME/.bashrc or $HOME/.profile the following instruction:

   . <(%s completion)
`, rootCmd.Use, rootCmd.Use),
		RunE: func(cmd *cobra.Command, _ []string) error {
			zsh, err := cmd.Flags().GetBool(flagZsh)
			if err != nil {
				return err
			}
			if zsh {
				return rootCmd.GenZshCompletion(cmd.OutOrStdout())
			}
			return rootCmd.GenBashCompletion(cmd.OutOrStdout())
		},
		Hidden: hidden,
		Args:   cobra.NoArgs,
	}

	cmd.Flags().Bool(flagZsh, false, "Generate Zsh completion script")

	return cmd
}
