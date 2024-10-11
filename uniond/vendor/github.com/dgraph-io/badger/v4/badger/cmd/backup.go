/*
 * Copyright 2017 Dgraph Labs, Inc. and Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package cmd

import (
	"bufio"
	"math"
	"os"

	"github.com/spf13/cobra"

	"github.com/dgraph-io/badger/v4"
)

var bo = struct {
	backupFile  string
	numVersions int
}{}

// backupCmd represents the backup command
var backupCmd = &cobra.Command{
	Use:   "backup",
	Short: "Backup Badger database.",
	Long: `Backup Badger database to a file in a version-agnostic manner.

Iterates over each key-value pair, encodes it along with its metadata and
version in protocol buffers and writes them to a file. This file can later be
used by the restore command to create an identical copy of the
database.`,
	RunE: doBackup,
}

func init() {
	RootCmd.AddCommand(backupCmd)
	backupCmd.Flags().StringVarP(&bo.backupFile, "backup-file", "f",
		"badger.bak", "File to backup to")
	backupCmd.Flags().IntVarP(&bo.numVersions, "num-versions", "n",
		0, "Number of versions to keep. A value <= 0 means keep all versions.")
}

func doBackup(cmd *cobra.Command, args []string) error {
	opt := badger.DefaultOptions(sstDir).
		WithValueDir(vlogDir).
		WithNumVersionsToKeep(math.MaxInt32)

	if bo.numVersions > 0 {
		opt.NumVersionsToKeep = bo.numVersions
	}

	// Open DB
	db, err := badger.Open(opt)
	if err != nil {
		return err
	}
	defer db.Close()

	// Create File
	f, err := os.Create(bo.backupFile)
	if err != nil {
		return err
	}

	bw := bufio.NewWriterSize(f, 64<<20)
	if _, err = db.Backup(bw, 0); err != nil {
		return err
	}

	if err = bw.Flush(); err != nil {
		return err
	}

	if err = f.Sync(); err != nil {
		return err
	}

	return f.Close()
}
