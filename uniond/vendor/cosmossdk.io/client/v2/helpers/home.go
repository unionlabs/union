package helpers

import (
	"os"
	"path/filepath"
	"strings"
)

// EnvPrefix is the prefix for environment variables that are used by the CLI.
// It should match the one used for viper in the CLI.
var EnvPrefix = ""

// GetNodeHomeDirectory gets the home directory of the node (where the config is located).
// It parses the home flag if set if the `(PREFIX)_HOME` environment variable if set (and ignores name).
// When no prefix is set, it reads the `NODE_HOME` environment variable.
// Otherwise, it returns the default home directory given its name.
func GetNodeHomeDirectory(name string) (string, error) {
	// get the home directory from the flag
	args := os.Args
	for i := 0; i < len(args); i++ {
		if args[i] == "--home" && i+1 < len(args) {
			return filepath.Clean(args[i+1]), nil
		} else if strings.HasPrefix(args[i], "--home=") {
			return filepath.Clean(args[i][7:]), nil
		}
	}

	// get the home directory from the environment variable
	// to not clash with the $HOME system variable, when no prefix is set
	// we check the NODE_HOME environment variable
	homeDir, envHome := "", "HOME"
	if len(EnvPrefix) > 0 {
		homeDir = os.Getenv(EnvPrefix + "_" + envHome)
	} else {
		homeDir = os.Getenv("NODE_" + envHome)
	}
	if homeDir != "" {
		return filepath.Clean(homeDir), nil
	}

	// get user home directory
	userHomeDir, err := os.UserHomeDir()
	if err != nil {
		return "", err
	}

	return filepath.Join(userHomeDir, name), nil
}
