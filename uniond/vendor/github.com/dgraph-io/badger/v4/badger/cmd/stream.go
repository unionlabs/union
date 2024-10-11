/*
 * Copyright 2020 Dgraph Labs, Inc. and Contributors
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
	"fmt"
	"io"
	"math"
	"os"

	"github.com/pkg/errors"
	"github.com/spf13/cobra"

	"github.com/dgraph-io/badger/v4"
	"github.com/dgraph-io/badger/v4/options"
	"github.com/dgraph-io/badger/v4/y"
)

var streamCmd = &cobra.Command{
	Use:   "stream",
	Short: "Stream DB into another DB with different options",
	Long: `
This command streams the contents of this DB into another DB with the given options.
`,
	RunE: stream,
}

var so = struct {
	outDir          string
	outFile         string
	compressionType uint32
	numVersions     int
	readOnly        bool
	keyPath         string
}{}

func init() {
	// TODO: Add more options.
	RootCmd.AddCommand(streamCmd)
	streamCmd.Flags().StringVarP(&so.outDir, "out", "o", "",
		"Path to output DB. The directory should be empty.")
	streamCmd.Flags().StringVarP(&so.outFile, "", "f", "",
		"Run a backup to this file.")
	streamCmd.Flags().BoolVarP(&so.readOnly, "read_only", "", true,
		"Option to open input DB in read-only mode")
	streamCmd.Flags().IntVarP(&so.numVersions, "num_versions", "", 0,
		"Option to configure the maximum number of versions per key. "+
			"Values <= 0 will be considered to have the max number of versions.")
	streamCmd.Flags().Uint32VarP(&so.compressionType, "compression", "", 1,
		"Option to configure the compression type in output DB. "+
			"0 to disable, 1 for Snappy, and 2 for ZSTD.")
	streamCmd.Flags().StringVarP(&so.keyPath, "encryption-key-file", "e", "",
		"Path of the encryption key file.")
}

func stream(cmd *cobra.Command, args []string) error {
	// Options for input DB.
	if so.numVersions <= 0 {
		so.numVersions = math.MaxInt32
	}
	encKey, err := getKey(so.keyPath)
	if err != nil {
		return err
	}
	inOpt := badger.DefaultOptions(sstDir).
		WithReadOnly(so.readOnly).
		WithValueThreshold(1 << 10 /* 1KB */).
		WithNumVersionsToKeep(so.numVersions).
		WithBlockCacheSize(100 << 20).
		WithIndexCacheSize(200 << 20).
		WithEncryptionKey(encKey)

	// Options for output DB.
	if so.compressionType > 2 {
		return errors.Errorf(
			"compression value must be one of 0 (disabled), 1 (Snappy), or 2 (ZSTD)")
	}
	inDB, err := badger.OpenManaged(inOpt)
	if err != nil {
		return y.Wrapf(err, "cannot open DB at %s", sstDir)
	}
	defer inDB.Close()

	stream := inDB.NewStreamAt(math.MaxUint64)

	if len(so.outDir) > 0 {
		if _, err := os.Stat(so.outDir); err == nil {
			f, err := os.Open(so.outDir)
			if err != nil {
				return err
			}
			defer f.Close()

			_, err = f.Readdirnames(1)
			if err != io.EOF {
				return errors.Errorf(
					"cannot run stream tool on non-empty output directory %s", so.outDir)
			}
		}

		stream.LogPrefix = "DB.Stream"
		outOpt := inOpt.
			WithDir(so.outDir).
			WithValueDir(so.outDir).
			WithNumVersionsToKeep(so.numVersions).
			WithCompression(options.CompressionType(so.compressionType)).
			WithEncryptionKey(encKey).
			WithReadOnly(false)
		err = inDB.StreamDB(outOpt)

	} else if len(so.outFile) > 0 {
		stream.LogPrefix = "DB.Backup"
		f, err := os.OpenFile(so.outFile, os.O_RDWR|os.O_CREATE, 0666)
		y.Check(err)
		_, err = stream.Backup(f, 0)
		y.Check(err)
	}
	fmt.Println("Done.")
	return err
}
