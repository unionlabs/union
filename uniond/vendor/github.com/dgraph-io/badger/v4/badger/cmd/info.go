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
	"bytes"
	"encoding/hex"
	"fmt"
	"io/fs"
	"os"
	"path/filepath"
	"sort"
	"strings"
	"time"

	humanize "github.com/dustin/go-humanize"
	"github.com/pkg/errors"
	"github.com/spf13/cobra"

	"github.com/dgraph-io/badger/v4"
	"github.com/dgraph-io/badger/v4/options"
	"github.com/dgraph-io/badger/v4/table"
	"github.com/dgraph-io/badger/v4/y"
)

type flagOptions struct {
	showTables               bool
	showHistogram            bool
	showKeys                 bool
	withPrefix               string
	keyLookup                string
	itemMeta                 bool
	keyHistory               bool
	showInternal             bool
	readOnly                 bool
	truncate                 bool
	encryptionKey            string
	checksumVerificationMode string
	discard                  bool
	externalMagicVersion     uint16
}

var (
	opt flagOptions
)

func init() {
	RootCmd.AddCommand(infoCmd)
	infoCmd.Flags().BoolVarP(&opt.showTables, "show-tables", "s", false,
		"If set to true, show tables as well.")
	infoCmd.Flags().BoolVar(&opt.showHistogram, "histogram", false,
		"Show a histogram of the key and value sizes.")
	infoCmd.Flags().BoolVar(&opt.showKeys, "show-keys", false, "Show keys stored in Badger")
	infoCmd.Flags().StringVar(&opt.withPrefix, "with-prefix", "",
		"Consider only the keys with specified prefix")
	infoCmd.Flags().StringVarP(&opt.keyLookup, "lookup", "l", "", "Hex of the key to lookup")
	infoCmd.Flags().BoolVar(&opt.itemMeta, "show-meta", true, "Output item meta data as well")
	infoCmd.Flags().BoolVar(&opt.keyHistory, "history", false, "Show all versions of a key")
	infoCmd.Flags().BoolVar(
		&opt.showInternal, "show-internal", false, "Show internal keys along with other keys."+
			" This option should be used along with --show-key option")
	infoCmd.Flags().BoolVar(&opt.readOnly, "read-only", true, "If set to true, DB will be opened "+
		"in read only mode. If DB has not been closed properly, this option can be set to false "+
		"to open DB.")
	infoCmd.Flags().BoolVar(&opt.truncate, "truncate", false, "If set to true, it allows "+
		"truncation of value log files if they have corrupt data.")
	infoCmd.Flags().StringVar(&opt.encryptionKey, "enc-key", "", "Use the provided encryption key")
	infoCmd.Flags().StringVar(&opt.checksumVerificationMode, "cv-mode", "none",
		"[none, table, block, tableAndBlock] Specifies when the db should verify checksum for SST.")
	infoCmd.Flags().BoolVar(&opt.discard, "discard", false,
		"Parse and print DISCARD file from value logs.")
	infoCmd.Flags().Uint16Var(&opt.externalMagicVersion, "external-magic", 0,
		"External magic number")
}

var infoCmd = &cobra.Command{
	Use:   "info",
	Short: "Health info about Badger database.",
	Long: `
This command prints information about the badger key-value store.  It reads MANIFEST and prints its
info. It also prints info about missing/extra files, and general information about the value log
files (which are not referenced by the manifest).  Use this tool to report any issues about Badger
to the Dgraph team.
`,
	RunE: handleInfo,
}

func handleInfo(cmd *cobra.Command, args []string) error {
	cvMode := checksumVerificationMode(opt.checksumVerificationMode)
	bopt := badger.DefaultOptions(sstDir).
		WithValueDir(vlogDir).
		WithReadOnly(opt.readOnly).
		WithBlockCacheSize(100 << 20).
		WithIndexCacheSize(200 << 20).
		WithEncryptionKey([]byte(opt.encryptionKey)).
		WithChecksumVerificationMode(cvMode).
		WithExternalMagic(opt.externalMagicVersion)

	if opt.discard {
		ds, err := badger.InitDiscardStats(bopt)
		y.Check(err)
		ds.Iterate(func(fid, stats uint64) {
			fmt.Printf("Value Log Fid: %5d. Stats: %10d [ %s ]\n",
				fid, stats, humanize.IBytes(stats))
		})
		fmt.Println("DONE")
		return nil
	}

	if err := printInfo(sstDir, vlogDir); err != nil {
		return y.Wrap(err, "failed to print information in MANIFEST file")
	}

	// Open DB
	db, err := badger.Open(bopt)
	if err != nil {
		return y.Wrap(err, "failed to open database")
	}
	defer db.Close()

	if opt.showTables {
		tableInfo(sstDir, vlogDir, db)
	}

	prefix, err := hex.DecodeString(opt.withPrefix)
	if err != nil {
		return y.Wrapf(err, "failed to decode hex prefix: %s", opt.withPrefix)
	}
	if opt.showHistogram {
		db.PrintHistogram(prefix)
	}

	if opt.showKeys {
		if err := showKeys(db, prefix); err != nil {
			return err
		}
	}

	if len(opt.keyLookup) > 0 {
		if err := lookup(db); err != nil {
			return y.Wrapf(err, "failed to perform lookup for the key: %x", opt.keyLookup)
		}
	}
	return nil
}

func showKeys(db *badger.DB, prefix []byte) error {
	if len(prefix) > 0 {
		fmt.Printf("Only choosing keys with prefix: \n%s", hex.Dump(prefix))
	}
	txn := db.NewTransaction(false)
	defer txn.Discard()

	iopt := badger.DefaultIteratorOptions
	iopt.Prefix = prefix
	iopt.PrefetchValues = false
	iopt.AllVersions = opt.keyHistory
	iopt.InternalAccess = opt.showInternal
	it := txn.NewIterator(iopt)
	defer it.Close()

	totalKeys := 0
	for it.Rewind(); it.Valid(); it.Next() {
		item := it.Item()
		if err := printKey(item, false); err != nil {
			return y.Wrapf(err, "failed to print information about key: %x(%d)",
				item.Key(), item.Version())
		}
		totalKeys++
	}
	fmt.Print("\n[Summary]\n")
	fmt.Println("Total Number of keys:", totalKeys)
	return nil

}

func lookup(db *badger.DB) error {
	txn := db.NewTransaction(false)
	defer txn.Discard()

	key, err := hex.DecodeString(opt.keyLookup)
	if err != nil {
		return y.Wrapf(err, "failed to decode key: %q", opt.keyLookup)
	}

	iopts := badger.DefaultIteratorOptions
	iopts.AllVersions = opt.keyHistory
	iopts.PrefetchValues = opt.keyHistory
	itr := txn.NewKeyIterator(key, iopts)
	defer itr.Close()

	itr.Rewind()
	if !itr.Valid() {
		return errors.Errorf("Unable to rewind to key:\n%s", hex.Dump(key))
	}
	fmt.Println()
	item := itr.Item()
	if err := printKey(item, true); err != nil {
		return y.Wrapf(err, "failed to print information about key: %x(%d)",
			item.Key(), item.Version())
	}

	if !opt.keyHistory {
		return nil
	}

	itr.Next() // Move to the next key
	for ; itr.Valid(); itr.Next() {
		item := itr.Item()
		if !bytes.Equal(key, item.Key()) {
			break
		}
		if err := printKey(item, true); err != nil {
			return y.Wrapf(err, "failed to print information about key: %x(%d)",
				item.Key(), item.Version())
		}
	}
	return nil
}

func printKey(item *badger.Item, showValue bool) error {
	var buf bytes.Buffer
	fmt.Fprintf(&buf, "Key: %x\tversion: %d", item.Key(), item.Version())
	if opt.itemMeta {
		fmt.Fprintf(&buf, "\tsize: %d\tmeta: b%04b", item.EstimatedSize(), item.UserMeta())
	}
	if item.IsDeletedOrExpired() {
		buf.WriteString("\t{deleted}")
	}
	if item.DiscardEarlierVersions() {
		buf.WriteString("\t{discard}")
	}
	if showValue {
		val, err := item.ValueCopy(nil)
		if err != nil {
			return y.Wrapf(err,
				"failed to copy value of the key: %x(%d)", item.Key(), item.Version())
		}
		fmt.Fprintf(&buf, "\n\tvalue: %v", val)
	}
	fmt.Println(buf.String())
	return nil
}

func hbytes(sz int64) string {
	return humanize.IBytes(uint64(sz))
}

func dur(src, dst time.Time) string {
	return humanize.RelTime(dst, src, "earlier", "later")
}

func getInfo(fileInfos []os.FileInfo, tid uint64) int64 {
	fileName := table.IDToFilename(tid)
	for _, fi := range fileInfos {
		if filepath.Base(fi.Name()) == fileName {
			return fi.Size()
		}
	}
	return 0
}

func tableInfo(dir, valueDir string, db *badger.DB) {
	// we want all tables with keys count here.
	tables := db.Tables()
	fileInfos, err := readDir(dir)
	y.Check(err)

	fmt.Println()
	// Total keys includes the internal keys as well.
	fmt.Println("SSTable [Li, Id, Total Keys] " +
		"[Compression Ratio, StaleData Ratio, Uncompressed Size, Index Size, BF Size] " +
		"[Left Key, Version -> Right Key, Version]")
	totalIndex := uint64(0)
	totalBloomFilter := uint64(0)
	totalCompressionRatio := float64(0.0)
	for _, t := range tables {
		lk, lt := y.ParseKey(t.Left), y.ParseTs(t.Left)
		rk, rt := y.ParseKey(t.Right), y.ParseTs(t.Right)

		compressionRatio := float64(t.UncompressedSize) /
			float64(getInfo(fileInfos, t.ID)-int64(t.IndexSz))
		staleDataRatio := float64(t.StaleDataSize) / float64(t.UncompressedSize)
		fmt.Printf("SSTable [L%d, %03d, %07d] [%.2f, %.2f, %s, %s, %s] [%20X, v%d -> %20X, v%d]\n",
			t.Level, t.ID, t.KeyCount, compressionRatio, staleDataRatio,
			hbytes(int64(t.UncompressedSize)), hbytes(int64(t.IndexSz)),
			hbytes(int64(t.BloomFilterSize)), lk, lt, rk, rt)
		totalIndex += uint64(t.IndexSz)
		totalBloomFilter += uint64(t.BloomFilterSize)
		totalCompressionRatio += compressionRatio
	}
	fmt.Println()
	fmt.Printf("Total Index Size: %s\n", hbytes(int64(totalIndex)))
	fmt.Printf("Total BloomFilter Size: %s\n", hbytes(int64(totalIndex)))
	fmt.Printf("Mean Compression Ratio: %.2f\n", totalCompressionRatio/float64(len(tables)))
	fmt.Println()
}

func readDir(dir string) ([]fs.FileInfo, error) {
	entries, err := os.ReadDir(dir)
	if err != nil {
		return nil, err
	}
	infos := make([]fs.FileInfo, 0, len(entries))
	for _, entry := range entries {
		var info fs.FileInfo
		info, err = entry.Info()
		if err != nil {
			return nil, err
		}
		infos = append(infos, info)
	}
	return infos, err
}

func printInfo(dir, valueDir string) error {
	if dir == "" {
		return fmt.Errorf("--dir not supplied")
	}
	if valueDir == "" {
		valueDir = dir
	}
	fp, err := os.Open(filepath.Join(dir, badger.ManifestFilename))
	if err != nil {
		return err
	}
	defer func() {
		if fp != nil {
			fp.Close()
		}
	}()
	manifest, truncOffset, err := badger.ReplayManifestFile(fp, opt.externalMagicVersion)
	if err != nil {
		return err
	}
	fp.Close()
	fp = nil

	fileinfos, err := readDir(dir)
	if err != nil {
		return err
	}

	fileinfoByName := make(map[string]os.FileInfo)
	fileinfoMarked := make(map[string]bool)
	for _, info := range fileinfos {
		fileinfoByName[info.Name()] = info
		fileinfoMarked[info.Name()] = false
	}

	fmt.Println()
	var baseTime time.Time
	manifestTruncated := false
	manifestInfo, ok := fileinfoByName[badger.ManifestFilename]
	if ok {
		fileinfoMarked[badger.ManifestFilename] = true
		truncatedString := ""
		if truncOffset != manifestInfo.Size() {
			truncatedString = fmt.Sprintf(" [TRUNCATED to %d]", truncOffset)
			manifestTruncated = true
		}

		baseTime = manifestInfo.ModTime()
		fmt.Printf("[%25s] %-12s %6s MA%s\n", manifestInfo.ModTime().Format(time.RFC3339),
			manifestInfo.Name(), hbytes(manifestInfo.Size()), truncatedString)
	} else {
		fmt.Printf("%s [MISSING]\n", manifestInfo.Name())
	}

	numMissing := 0
	numEmpty := 0

	levelSizes := make([]int64, len(manifest.Levels))
	for level, lm := range manifest.Levels {
		// fmt.Printf("\n[Level %d]\n", level)
		// We create a sorted list of table ID's so that output is in consistent order.
		tableIDs := make([]uint64, 0, len(lm.Tables))
		for id := range lm.Tables {
			tableIDs = append(tableIDs, id)
		}
		sort.Slice(tableIDs, func(i, j int) bool {
			return tableIDs[i] < tableIDs[j]
		})
		for _, tableID := range tableIDs {
			tableFile := table.IDToFilename(tableID)
			_, ok1 := manifest.Tables[tableID]
			file, ok2 := fileinfoByName[tableFile]
			if ok1 && ok2 {
				fileinfoMarked[tableFile] = true
				emptyString := ""
				fileSize := file.Size()
				if fileSize == 0 {
					emptyString = " [EMPTY]"
					numEmpty++
				}
				levelSizes[level] += fileSize
				// (Put level on every line to make easier to process with sed/perl.)
				fmt.Printf("[%25s] %-12s %6s L%d %s\n", dur(baseTime, file.ModTime()),
					tableFile, hbytes(fileSize), level, emptyString)
			} else {
				fmt.Printf("%s [MISSING]\n", tableFile)
				numMissing++
			}
		}
	}

	valueDirFileinfos := fileinfos
	if valueDir != dir {
		valueDirFileinfos, err = readDir(valueDir)
		if err != nil {
			return err
		}

	}

	// If valueDir is different from dir, holds extra files in the value dir.
	valueDirExtras := []os.FileInfo{}

	valueLogSize := int64(0)
	// fmt.Print("\n[Value Log]\n")
	for _, file := range valueDirFileinfos {
		if !strings.HasSuffix(file.Name(), ".vlog") {
			if valueDir != dir {
				valueDirExtras = append(valueDirExtras, file)
			}
			continue
		}

		fileSize := file.Size()
		emptyString := ""
		if fileSize == 0 {
			emptyString = " [EMPTY]"
			numEmpty++
		}
		valueLogSize += fileSize
		fmt.Printf("[%25s] %-12s %6s VL%s\n", dur(baseTime, file.ModTime()), file.Name(),
			hbytes(fileSize), emptyString)

		fileinfoMarked[file.Name()] = true
	}

	numExtra := 0
	for _, file := range fileinfos {
		if fileinfoMarked[file.Name()] {
			continue
		}
		if numExtra == 0 {
			fmt.Print("\n[EXTRA]\n")
		}
		fmt.Printf("[%s] %-12s %6s\n", file.ModTime().Format(time.RFC3339),
			file.Name(), hbytes(file.Size()))
		numExtra++
	}

	numValueDirExtra := 0
	for _, file := range valueDirExtras {
		if numValueDirExtra == 0 {
			fmt.Print("\n[ValueDir EXTRA]\n")
		}
		fmt.Printf("[%s] %-12s %6s\n", file.ModTime().Format(time.RFC3339),
			file.Name(), hbytes(file.Size()))
		numValueDirExtra++
	}

	fmt.Print("\n[Summary]\n")
	totalSSTSize := int64(0)
	for i, sz := range levelSizes {
		fmt.Printf("Level %d size: %12s\n", i, hbytes(sz))
		totalSSTSize += sz
	}

	fmt.Printf("Total SST size: %10s\n", hbytes(totalSSTSize))
	fmt.Printf("Value log size: %10s\n", hbytes(valueLogSize))
	fmt.Println()
	totalExtra := numExtra + numValueDirExtra
	if totalExtra == 0 && numMissing == 0 && numEmpty == 0 && !manifestTruncated {
		fmt.Println("Abnormalities: None.")
	} else {
		fmt.Println("Abnormalities:")
	}
	fmt.Printf("%d extra %s.\n", totalExtra, pluralFiles(totalExtra))
	fmt.Printf("%d missing %s.\n", numMissing, pluralFiles(numMissing))
	fmt.Printf("%d empty %s.\n", numEmpty, pluralFiles(numEmpty))
	fmt.Printf("%d truncated %s.\n", boolToNum(manifestTruncated),
		pluralManifest(manifestTruncated))

	return nil
}

func boolToNum(x bool) int {
	if x {
		return 1
	}
	return 0
}

func pluralManifest(manifestTruncated bool) string {
	if manifestTruncated {
		return "manifest"
	}
	return "manifests"
}

func pluralFiles(count int) string {
	if count == 1 {
		return "file"
	}
	return "files"
}

func checksumVerificationMode(cvMode string) options.ChecksumVerificationMode {
	switch cvMode {
	case "none":
		return options.NoVerification
	case "table":
		return options.OnTableRead
	case "block":
		return options.OnBlockRead
	case "tableAndblock":
		return options.OnTableAndBlockRead
	default:
		fmt.Printf("Invalid checksum verification mode: %s\n", cvMode)
		os.Exit(1)
	}
	return options.NoVerification
}
