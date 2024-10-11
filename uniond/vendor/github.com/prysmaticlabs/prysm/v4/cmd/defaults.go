// Copyright 2016 The go-ethereum Authors
// This file is part of the go-ethereum library.
//
// The go-ethereum library is free software: you can redistribute it and/or modify
// it under the terms of the GNU Lesser General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// The go-ethereum library is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU Lesser General Public License for more details.
//
// You should have received a copy of the GNU Lesser General Public License
// along with the go-ethereum library. If not, see <http://www.gnu.org/licenses/>.

package cmd

import (
	"fmt"
	"os"
	"path/filepath"
	"runtime"

	"github.com/prysmaticlabs/prysm/v4/io/file"
)

// DefaultDataDir is the default data directory to use for the databases and other
// persistence requirements.
func DefaultDataDir() string {
	// Try to place the data folder in the user's home dir
	home := file.HomeDir()
	if home != "" {
		switch runtime.GOOS {
		case "darwin":
			return filepath.Join(home, "Library", "Eth2")
		case "windows":
			return filepath.Join(home, "AppData", "Local", "Eth2")
		default:
			return filepath.Join(home, ".eth2")
		}
	}
	// As we cannot guess a stable location, return empty and handle later
	return ""
}

// FixDefaultDataDir checks if previous data directory is found and can be migrated to a new path.
// This is used to resolve issue with weak default path (for Windows users) in existing installations.
// For full details see: https://github.com/prysmaticlabs/prysm/issues/5660.
func FixDefaultDataDir(prevDataDir, curDataDir string) error {
	if runtime.GOOS != "windows" {
		return nil
	}

	// Clean paths.
	prevDataDir, err := file.ExpandPath(prevDataDir)
	if err != nil {
		return err
	}
	curDataDir, err = file.ExpandPath(curDataDir)
	if err != nil {
		return err
	}

	// See if shared directory is found (if it is -- we need to move it to non-shared destination).
	prevDataDirExists, err := file.HasDir(prevDataDir)
	if err != nil {
		return err
	}
	if !prevDataDirExists {
		// If no previous "%APPDATA%\Eth2" found, nothing to patch and move to new default location.
		return nil
	}

	if curDataDir == "" {
		curDataDir = DefaultDataDir()
	}
	selectedDirExists, err := file.HasDir(curDataDir)
	if err != nil {
		return err
	}
	if selectedDirExists {
		// No need not move anything, destination directory already exists.
		log.Warnf("Outdated data directory is found: %s! The current data folder %s is not empty, "+
			"so can not copy files automatically. Either remove outdated data directory, or "+
			"consider specifying non-existent new data directory (files will be moved automatically).\n"+
			"For full details see: https://github.com/prysmaticlabs/prysm/issues/5660.",
			prevDataDir, curDataDir)
		return nil
	}

	if curDataDir == prevDataDir {
		return nil
	}

	log.Warnf("Outdated data directory is found: %s. "+
		"Copying its contents to the new data folder: %s", prevDataDir, curDataDir)
	if err := file.CopyDir(prevDataDir, curDataDir); err != nil {
		return err
	}
	log.Infof("All files from the outdated data directory %s have been moved to %s.", prevDataDir, curDataDir)

	// If directories match, previous data directory can be safely deleted.
	actionText := "The outdated directory is copied and not needed anymore, so should be deleted. " +
		"Directory %s and its contents will be removed - do you want to proceed? (Y/N)"
	deniedText := "Outdated directory will not be deleted. No changes have been made."
	removeConfirmed, err := ConfirmAction(fmt.Sprintf(actionText, prevDataDir), deniedText)
	if err != nil {
		return err
	}
	if removeConfirmed && file.DirsEqual(prevDataDir, curDataDir) {
		if err := os.RemoveAll(prevDataDir); err != nil {
			return fmt.Errorf("cannot remove outdated directory or one of its files: %w", err)
		}
		log.Infof("Successfully removed %s", prevDataDir)
	}
	return nil
}
