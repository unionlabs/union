package main

import (
	"context"
	"os"

	e2e "github.com/cometbft/cometbft/test/e2e/pkg"
	"github.com/cometbft/cometbft/test/e2e/pkg/exec"
)

// Test runs test cases under tests.
func Test(testnet *e2e.Testnet, ifd *e2e.InfrastructureData) error {
	logger.Info("Running tests in ./tests/...")

	err := os.Setenv("E2E_MANIFEST", testnet.File)
	if err != nil {
		return err
	}
	err = os.Setenv("E2E_TESTNET_DIR", testnet.Dir)
	if err != nil {
		return err
	}
	if p := ifd.Path; p != "" {
		err = os.Setenv("INFRASTRUCTURE_FILE", p)
		if err != nil {
			return err
		}
	}
	err = os.Setenv("INFRASTRUCTURE_TYPE", ifd.Provider)
	if err != nil {
		return err
	}

	cmd := []string{"go", "test", "-count", "1"}
	verbose := os.Getenv("VERBOSE")
	if verbose == "1" {
		cmd = append(cmd, "-v")
	}
	cmd = append(cmd, "./tests/...")

	return exec.CommandVerbose(context.Background(), cmd...)
}
