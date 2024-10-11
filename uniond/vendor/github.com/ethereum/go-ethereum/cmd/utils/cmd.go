// Copyright 2014 The go-ethereum Authors
// This file is part of go-ethereum.
//
// go-ethereum is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// go-ethereum is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with go-ethereum. If not, see <http://www.gnu.org/licenses/>.

// Package utils contains internal helper functions for go-ethereum commands.
package utils

import (
	"bufio"
	"compress/gzip"
	"errors"
	"fmt"
	"io"
	"os"
	"os/signal"
	"runtime"
	"strings"
	"syscall"
	"time"

	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core"
	"github.com/ethereum/go-ethereum/core/rawdb"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/crypto"
	"github.com/ethereum/go-ethereum/eth/ethconfig"
	"github.com/ethereum/go-ethereum/ethdb"
	"github.com/ethereum/go-ethereum/internal/debug"
	"github.com/ethereum/go-ethereum/log"
	"github.com/ethereum/go-ethereum/node"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/urfave/cli/v2"
)

const (
	importBatchSize = 2500
)

// Fatalf formats a message to standard error and exits the program.
// The message is also printed to standard output if standard error
// is redirected to a different file.
func Fatalf(format string, args ...interface{}) {
	w := io.MultiWriter(os.Stdout, os.Stderr)
	if runtime.GOOS == "windows" {
		// The SameFile check below doesn't work on Windows.
		// stdout is unlikely to get redirected though, so just print there.
		w = os.Stdout
	} else {
		outf, _ := os.Stdout.Stat()
		errf, _ := os.Stderr.Stat()
		if outf != nil && errf != nil && os.SameFile(outf, errf) {
			w = os.Stderr
		}
	}
	fmt.Fprintf(w, "Fatal: "+format+"\n", args...)
	os.Exit(1)
}

func StartNode(ctx *cli.Context, stack *node.Node, isConsole bool) {
	if err := stack.Start(); err != nil {
		Fatalf("Error starting protocol stack: %v", err)
	}
	go func() {
		sigc := make(chan os.Signal, 1)
		signal.Notify(sigc, syscall.SIGINT, syscall.SIGTERM)
		defer signal.Stop(sigc)

		minFreeDiskSpace := 2 * ethconfig.Defaults.TrieDirtyCache // Default 2 * 256Mb
		if ctx.IsSet(MinFreeDiskSpaceFlag.Name) {
			minFreeDiskSpace = ctx.Int(MinFreeDiskSpaceFlag.Name)
		} else if ctx.IsSet(CacheFlag.Name) || ctx.IsSet(CacheGCFlag.Name) {
			minFreeDiskSpace = 2 * ctx.Int(CacheFlag.Name) * ctx.Int(CacheGCFlag.Name) / 100
		}
		if minFreeDiskSpace > 0 {
			go monitorFreeDiskSpace(sigc, stack.InstanceDir(), uint64(minFreeDiskSpace)*1024*1024)
		}

		shutdown := func() {
			log.Info("Got interrupt, shutting down...")
			go stack.Close()
			for i := 10; i > 0; i-- {
				<-sigc
				if i > 1 {
					log.Warn("Already shutting down, interrupt more to panic.", "times", i-1)
				}
			}
			debug.Exit() // ensure trace and CPU profile data is flushed.
			debug.LoudPanic("boom")
		}

		if isConsole {
			// In JS console mode, SIGINT is ignored because it's handled by the console.
			// However, SIGTERM still shuts down the node.
			for {
				sig := <-sigc
				if sig == syscall.SIGTERM {
					shutdown()
					return
				}
			}
		} else {
			<-sigc
			shutdown()
		}
	}()
}

func monitorFreeDiskSpace(sigc chan os.Signal, path string, freeDiskSpaceCritical uint64) {
	if path == "" {
		return
	}
	for {
		freeSpace, err := getFreeDiskSpace(path)
		if err != nil {
			log.Warn("Failed to get free disk space", "path", path, "err", err)
			break
		}
		if freeSpace < freeDiskSpaceCritical {
			log.Error("Low disk space. Gracefully shutting down Geth to prevent database corruption.", "available", common.StorageSize(freeSpace), "path", path)
			sigc <- syscall.SIGTERM
			break
		} else if freeSpace < 2*freeDiskSpaceCritical {
			log.Warn("Disk space is running low. Geth will shutdown if disk space runs below critical level.", "available", common.StorageSize(freeSpace), "critical_level", common.StorageSize(freeDiskSpaceCritical), "path", path)
		}
		time.Sleep(30 * time.Second)
	}
}

func ImportChain(chain *core.BlockChain, fn string) error {
	// Watch for Ctrl-C while the import is running.
	// If a signal is received, the import will stop at the next batch.
	interrupt := make(chan os.Signal, 1)
	stop := make(chan struct{})
	signal.Notify(interrupt, syscall.SIGINT, syscall.SIGTERM)
	defer signal.Stop(interrupt)
	defer close(interrupt)
	go func() {
		if _, ok := <-interrupt; ok {
			log.Info("Interrupted during import, stopping at next batch")
		}
		close(stop)
	}()
	checkInterrupt := func() bool {
		select {
		case <-stop:
			return true
		default:
			return false
		}
	}

	log.Info("Importing blockchain", "file", fn)

	// Open the file handle and potentially unwrap the gzip stream
	fh, err := os.Open(fn)
	if err != nil {
		return err
	}
	defer fh.Close()

	var reader io.Reader = fh
	if strings.HasSuffix(fn, ".gz") {
		if reader, err = gzip.NewReader(reader); err != nil {
			return err
		}
	}
	stream := rlp.NewStream(reader, 0)

	// Run actual the import.
	blocks := make(types.Blocks, importBatchSize)
	n := 0
	for batch := 0; ; batch++ {
		// Load a batch of RLP blocks.
		if checkInterrupt() {
			return errors.New("interrupted")
		}
		i := 0
		for ; i < importBatchSize; i++ {
			var b types.Block
			if err := stream.Decode(&b); err == io.EOF {
				break
			} else if err != nil {
				return fmt.Errorf("at block %d: %v", n, err)
			}
			// don't import first block
			if b.NumberU64() == 0 {
				i--
				continue
			}
			blocks[i] = &b
			n++
		}
		if i == 0 {
			break
		}
		// Import the batch.
		if checkInterrupt() {
			return errors.New("interrupted")
		}
		missing := missingBlocks(chain, blocks[:i])
		if len(missing) == 0 {
			log.Info("Skipping batch as all blocks present", "batch", batch, "first", blocks[0].Hash(), "last", blocks[i-1].Hash())
			continue
		}
		if failindex, err := chain.InsertChain(missing); err != nil {
			var failnumber uint64
			if failindex > 0 && failindex < len(missing) {
				failnumber = missing[failindex].NumberU64()
			} else {
				failnumber = missing[0].NumberU64()
			}
			return fmt.Errorf("invalid block %d: %v", failnumber, err)
		}
	}
	return nil
}

func missingBlocks(chain *core.BlockChain, blocks []*types.Block) []*types.Block {
	head := chain.CurrentBlock()
	for i, block := range blocks {
		// If we're behind the chain head, only check block, state is available at head
		if head.Number.Uint64() > block.NumberU64() {
			if !chain.HasBlock(block.Hash(), block.NumberU64()) {
				return blocks[i:]
			}
			continue
		}
		// If we're above the chain head, state availability is a must
		if !chain.HasBlockAndState(block.Hash(), block.NumberU64()) {
			return blocks[i:]
		}
	}
	return nil
}

// ExportChain exports a blockchain into the specified file, truncating any data
// already present in the file.
func ExportChain(blockchain *core.BlockChain, fn string) error {
	log.Info("Exporting blockchain", "file", fn)

	// Open the file handle and potentially wrap with a gzip stream
	fh, err := os.OpenFile(fn, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, os.ModePerm)
	if err != nil {
		return err
	}
	defer fh.Close()

	var writer io.Writer = fh
	if strings.HasSuffix(fn, ".gz") {
		writer = gzip.NewWriter(writer)
		defer writer.(*gzip.Writer).Close()
	}
	// Iterate over the blocks and export them
	if err := blockchain.Export(writer); err != nil {
		return err
	}
	log.Info("Exported blockchain", "file", fn)

	return nil
}

// ExportAppendChain exports a blockchain into the specified file, appending to
// the file if data already exists in it.
func ExportAppendChain(blockchain *core.BlockChain, fn string, first uint64, last uint64) error {
	log.Info("Exporting blockchain", "file", fn)

	// Open the file handle and potentially wrap with a gzip stream
	fh, err := os.OpenFile(fn, os.O_CREATE|os.O_APPEND|os.O_WRONLY, os.ModePerm)
	if err != nil {
		return err
	}
	defer fh.Close()

	var writer io.Writer = fh
	if strings.HasSuffix(fn, ".gz") {
		writer = gzip.NewWriter(writer)
		defer writer.(*gzip.Writer).Close()
	}
	// Iterate over the blocks and export them
	if err := blockchain.ExportN(writer, first, last); err != nil {
		return err
	}
	log.Info("Exported blockchain to", "file", fn)
	return nil
}

// ImportPreimages imports a batch of exported hash preimages into the database.
// It's a part of the deprecated functionality, should be removed in the future.
func ImportPreimages(db ethdb.Database, fn string) error {
	log.Info("Importing preimages", "file", fn)

	// Open the file handle and potentially unwrap the gzip stream
	fh, err := os.Open(fn)
	if err != nil {
		return err
	}
	defer fh.Close()

	var reader io.Reader = bufio.NewReader(fh)
	if strings.HasSuffix(fn, ".gz") {
		if reader, err = gzip.NewReader(reader); err != nil {
			return err
		}
	}
	stream := rlp.NewStream(reader, 0)

	// Import the preimages in batches to prevent disk thrashing
	preimages := make(map[common.Hash][]byte)

	for {
		// Read the next entry and ensure it's not junk
		var blob []byte

		if err := stream.Decode(&blob); err != nil {
			if err == io.EOF {
				break
			}
			return err
		}
		// Accumulate the preimages and flush when enough ws gathered
		preimages[crypto.Keccak256Hash(blob)] = common.CopyBytes(blob)
		if len(preimages) > 1024 {
			rawdb.WritePreimages(db, preimages)
			preimages = make(map[common.Hash][]byte)
		}
	}
	// Flush the last batch preimage data
	if len(preimages) > 0 {
		rawdb.WritePreimages(db, preimages)
	}
	return nil
}

// ExportPreimages exports all known hash preimages into the specified file,
// truncating any data already present in the file.
// It's a part of the deprecated functionality, should be removed in the future.
func ExportPreimages(db ethdb.Database, fn string) error {
	log.Info("Exporting preimages", "file", fn)

	// Open the file handle and potentially wrap with a gzip stream
	fh, err := os.OpenFile(fn, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, os.ModePerm)
	if err != nil {
		return err
	}
	defer fh.Close()

	var writer io.Writer = fh
	if strings.HasSuffix(fn, ".gz") {
		writer = gzip.NewWriter(writer)
		defer writer.(*gzip.Writer).Close()
	}
	// Iterate over the preimages and export them
	it := db.NewIterator([]byte("secure-key-"), nil)
	defer it.Release()

	for it.Next() {
		if err := rlp.Encode(writer, it.Value()); err != nil {
			return err
		}
	}
	log.Info("Exported preimages", "file", fn)
	return nil
}

// exportHeader is used in the export/import flow. When we do an export,
// the first element we output is the exportHeader.
// Whenever a backwards-incompatible change is made, the Version header
// should be bumped.
// If the importer sees a higher version, it should reject the import.
type exportHeader struct {
	Magic    string // Always set to 'gethdbdump' for disambiguation
	Version  uint64
	Kind     string
	UnixTime uint64
}

const exportMagic = "gethdbdump"
const (
	OpBatchAdd = 0
	OpBatchDel = 1
)

// ImportLDBData imports a batch of snapshot data into the database
func ImportLDBData(db ethdb.Database, f string, startIndex int64, interrupt chan struct{}) error {
	log.Info("Importing leveldb data", "file", f)

	// Open the file handle and potentially unwrap the gzip stream
	fh, err := os.Open(f)
	if err != nil {
		return err
	}
	defer fh.Close()

	var reader io.Reader = bufio.NewReader(fh)
	if strings.HasSuffix(f, ".gz") {
		if reader, err = gzip.NewReader(reader); err != nil {
			return err
		}
	}
	stream := rlp.NewStream(reader, 0)

	// Read the header
	var header exportHeader
	if err := stream.Decode(&header); err != nil {
		return fmt.Errorf("could not decode header: %v", err)
	}
	if header.Magic != exportMagic {
		return errors.New("incompatible data, wrong magic")
	}
	if header.Version != 0 {
		return fmt.Errorf("incompatible version %d, (support only 0)", header.Version)
	}
	log.Info("Importing data", "file", f, "type", header.Kind, "data age",
		common.PrettyDuration(time.Since(time.Unix(int64(header.UnixTime), 0))))

	// Import the snapshot in batches to prevent disk thrashing
	var (
		count  int64
		start  = time.Now()
		logged = time.Now()
		batch  = db.NewBatch()
	)
	for {
		// Read the next entry
		var (
			op       byte
			key, val []byte
		)
		if err := stream.Decode(&op); err != nil {
			if err == io.EOF {
				break
			}
			return err
		}
		if err := stream.Decode(&key); err != nil {
			return err
		}
		if err := stream.Decode(&val); err != nil {
			return err
		}
		if count < startIndex {
			count++
			continue
		}
		switch op {
		case OpBatchDel:
			batch.Delete(key)
		case OpBatchAdd:
			batch.Put(key, val)
		default:
			return fmt.Errorf("unknown op %d\n", op)
		}
		if batch.ValueSize() > ethdb.IdealBatchSize {
			if err := batch.Write(); err != nil {
				return err
			}
			batch.Reset()
		}
		// Check interruption emitted by ctrl+c
		if count%1000 == 0 {
			select {
			case <-interrupt:
				if err := batch.Write(); err != nil {
					return err
				}
				log.Info("External data import interrupted", "file", f, "count", count, "elapsed", common.PrettyDuration(time.Since(start)))
				return nil
			default:
			}
		}
		if count%1000 == 0 && time.Since(logged) > 8*time.Second {
			log.Info("Importing external data", "file", f, "count", count, "elapsed", common.PrettyDuration(time.Since(start)))
			logged = time.Now()
		}
		count += 1
	}
	// Flush the last batch snapshot data
	if batch.ValueSize() > 0 {
		if err := batch.Write(); err != nil {
			return err
		}
	}
	log.Info("Imported chain data", "file", f, "count", count,
		"elapsed", common.PrettyDuration(time.Since(start)))
	return nil
}

// ChainDataIterator is an interface wraps all necessary functions to iterate
// the exporting chain data.
type ChainDataIterator interface {
	// Next returns the key-value pair for next exporting entry in the iterator.
	// When the end is reached, it will return (0, nil, nil, false).
	Next() (byte, []byte, []byte, bool)

	// Release releases associated resources. Release should always succeed and can
	// be called multiple times without causing error.
	Release()
}

// ExportChaindata exports the given data type (truncating any data already present)
// in the file. If the suffix is 'gz', gzip compression is used.
func ExportChaindata(fn string, kind string, iter ChainDataIterator, interrupt chan struct{}) error {
	log.Info("Exporting chain data", "file", fn, "kind", kind)
	defer iter.Release()

	// Open the file handle and potentially wrap with a gzip stream
	fh, err := os.OpenFile(fn, os.O_CREATE|os.O_WRONLY|os.O_TRUNC, os.ModePerm)
	if err != nil {
		return err
	}
	defer fh.Close()

	var writer io.Writer = fh
	if strings.HasSuffix(fn, ".gz") {
		writer = gzip.NewWriter(writer)
		defer writer.(*gzip.Writer).Close()
	}
	// Write the header
	if err := rlp.Encode(writer, &exportHeader{
		Magic:    exportMagic,
		Version:  0,
		Kind:     kind,
		UnixTime: uint64(time.Now().Unix()),
	}); err != nil {
		return err
	}
	// Extract data from source iterator and dump them out to file
	var (
		count  int64
		start  = time.Now()
		logged = time.Now()
	)
	for {
		op, key, val, ok := iter.Next()
		if !ok {
			break
		}
		if err := rlp.Encode(writer, op); err != nil {
			return err
		}
		if err := rlp.Encode(writer, key); err != nil {
			return err
		}
		if err := rlp.Encode(writer, val); err != nil {
			return err
		}
		if count%1000 == 0 {
			// Check interruption emitted by ctrl+c
			select {
			case <-interrupt:
				log.Info("Chain data exporting interrupted", "file", fn,
					"kind", kind, "count", count, "elapsed", common.PrettyDuration(time.Since(start)))
				return nil
			default:
			}
			if time.Since(logged) > 8*time.Second {
				log.Info("Exporting chain data", "file", fn, "kind", kind,
					"count", count, "elapsed", common.PrettyDuration(time.Since(start)))
				logged = time.Now()
			}
		}
		count++
	}
	log.Info("Exported chain data", "file", fn, "kind", kind, "count", count,
		"elapsed", common.PrettyDuration(time.Since(start)))
	return nil
}
