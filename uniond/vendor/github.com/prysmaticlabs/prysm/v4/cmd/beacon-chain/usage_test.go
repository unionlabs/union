package main

import (
	"testing"

	"github.com/prysmaticlabs/prysm/v4/config/features"
	"github.com/urfave/cli/v2"
)

func TestAllFlagsExistInHelp(t *testing.T) {
	// If this test is failing, it is because you've recently added/removed a
	// flag in beacon chain main.go, but did not add/remove it to the usage.go
	// flag grouping (appHelpFlagGroups).

	var helpFlags []cli.Flag
	for _, group := range appHelpFlagGroups {
		helpFlags = append(helpFlags, group.Flags...)
	}
	helpFlags = features.ActiveFlags(helpFlags)
	appFlags = features.ActiveFlags(appFlags)

	for _, flag := range appFlags {
		if !doesFlagExist(flag, helpFlags) {
			t.Errorf("Flag %s does not exist in help/usage flags.", flag.Names()[0])
		}
	}

	for _, flag := range helpFlags {
		if !doesFlagExist(flag, appFlags) {
			t.Errorf("Flag %s does not exist in main.go, "+
				"but exists in help flags", flag.Names()[0])
		}
	}
}

func doesFlagExist(flag cli.Flag, flags []cli.Flag) bool {
	for _, f := range flags {
		if f.String() == flag.String() {
			return true
		}
	}

	return false
}
