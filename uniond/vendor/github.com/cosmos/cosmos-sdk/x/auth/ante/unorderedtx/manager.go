package unorderedtx

import (
	"bufio"
	"bytes"
	"context"
	"encoding/binary"
	"errors"
	"fmt"
	"io"
	"maps"
	"os"
	"path/filepath"
	"slices"
	"sync"
	"time"
)

const (
	// DefaultmaxTimeoutDuration defines the default maximum duration an un-ordered transaction
	// can set.
	DefaultMaxTimeoutDuration = time.Minute * 40

	dirName  = "unordered_txs"
	fileName = "data"
)

// TxHash defines a transaction hash type alias, which is a fixed array of 32 bytes.
type TxHash [32]byte

// Manager contains the tx hash dictionary for duplicates checking, and expire
// them when block production progresses.
type Manager struct {
	// blockCh defines a channel to receive newly committed block time
	blockCh chan time.Time
	// doneCh allows us to ensure the purgeLoop has gracefully terminated prior to closing
	doneCh chan struct{}

	// dataDir defines the directory to store unexpired unordered transactions
	//
	// XXX: Note, ideally we avoid the need to store unexpired unordered transactions
	// directly to file. However, store v1 does not allow such a primitive. But,
	// once store v2 is fully integrated, we can remove manual file handling and
	// store the unexpired unordered transactions directly to SS.
	//
	// Ref: https://github.com/cosmos/cosmos-sdk/issues/18467
	dataDir string

	mu sync.RWMutex

	// txHashes defines a map from tx hash -> TTL value defined as block time, which is used for duplicate
	// checking and replay protection, as well as purging the map when the TTL is
	// expired.
	txHashes map[TxHash]time.Time
}

func NewManager(dataDir string) *Manager {
	path := filepath.Join(dataDir, dirName)
	if _, err := os.Stat(path); errors.Is(err, os.ErrNotExist) {
		if err = os.MkdirAll(path, os.ModePerm); err != nil {
			panic(fmt.Errorf("failed to create unordered txs directory: %w", err))
		}
	}

	m := &Manager{
		dataDir:  dataDir,
		blockCh:  make(chan time.Time, 16),
		doneCh:   make(chan struct{}),
		txHashes: make(map[TxHash]time.Time),
	}

	return m
}

func (m *Manager) Start() {
	go m.purgeLoop()
}

// Close must be called when a node gracefully shuts down. Typically, this should
// be called in an application's Close() function, which is called by the server.
// Note, Start() must be called in order for Close() to not hang.
//
// It will free all necessary resources as well as writing all unexpired unordered
// transactions along with their TTL values to file.
func (m *Manager) Close() error {
	close(m.blockCh)
	<-m.doneCh
	m.blockCh = nil

	return m.flushToFile()
}

func (m *Manager) Contains(hash TxHash) bool {
	m.mu.RLock()
	defer m.mu.RUnlock()
	_, ok := m.txHashes[hash]
	return ok
}

func (m *Manager) Size() int {
	m.mu.RLock()
	defer m.mu.RUnlock()

	return len(m.txHashes)
}

func (m *Manager) Add(txHash TxHash, timestamp time.Time) {
	m.mu.Lock()
	defer m.mu.Unlock()

	m.txHashes[txHash] = timestamp
}

// OnInit must be called when a node starts up. Typically, this should be called
// in an application's constructor, which is called by the server.
func (m *Manager) OnInit() error {
	f, err := os.Open(filepath.Join(m.dataDir, dirName, fileName))
	if err != nil {
		if errors.Is(err, os.ErrNotExist) {
			// File does not exist, which we can assume that there are no unexpired
			// unordered transactions.
			return nil
		}

		return fmt.Errorf("failed to open unconfirmed txs file: %w", err)
	}
	defer f.Close()

	var (
		r   = bufio.NewReader(f)
		buf = make([]byte, chunkSize)
	)
	for {
		n, err := io.ReadFull(r, buf)
		if err != nil {
			if errors.Is(err, io.EOF) {
				break
			} else {
				return fmt.Errorf("failed to read unconfirmed txs file: %w", err)
			}
		}
		if n != 32+8 {
			return fmt.Errorf("read unexpected number of bytes from unconfirmed txs file: %d", n)
		}

		var txHash TxHash
		copy(txHash[:], buf[:txHashSize])

		timeStamp := binary.BigEndian.Uint64(buf[txHashSize:])
		m.Add(txHash, time.Unix(int64(timeStamp), 0))
	}

	return nil
}

// OnNewBlock sends the latest block time to the background purge loop, which
// should be called in ABCI Commit event.
func (m *Manager) OnNewBlock(blockTime time.Time) {
	m.blockCh <- blockTime
}

func (m *Manager) exportSnapshot(_ uint64, snapshotWriter func([]byte) error) error {
	var buf bytes.Buffer
	w := bufio.NewWriter(&buf)

	keys := slices.SortedFunc(maps.Keys(m.txHashes), func(i, j TxHash) int { return bytes.Compare(i[:], j[:]) })
	for _, txHash := range keys {
		timeoutTime := m.txHashes[txHash]

		// right now we dont have access block time at this flow, so we would just include the expired txs
		// and let it be purge during purge loop
		chunk := unorderedTxToBytes(txHash, uint64(timeoutTime.Unix()))

		if _, err := w.Write(chunk); err != nil {
			return fmt.Errorf("failed to write unordered tx to buffer: %w", err)
		}
	}

	if err := w.Flush(); err != nil {
		return fmt.Errorf("failed to flush unordered txs buffer: %w", err)
	}

	return snapshotWriter(buf.Bytes())
}

// flushToFile writes all unordered transactions (including expired if not pruned yet)
// along with their TTL to file, overwriting the existing file if it exists.
func (m *Manager) flushToFile() error {
	f, err := os.Create(filepath.Join(m.dataDir, dirName, fileName))
	if err != nil {
		return fmt.Errorf("failed to create unordered txs file: %w", err)
	}
	defer f.Close()

	w := bufio.NewWriter(f)
	for txHash, timestamp := range m.txHashes {
		chunk := unorderedTxToBytes(txHash, uint64(timestamp.Unix()))

		if _, err = w.Write(chunk); err != nil {
			return fmt.Errorf("failed to write unordered tx to buffer: %w", err)
		}
	}

	if err = w.Flush(); err != nil {
		return fmt.Errorf("failed to flush unordered txs buffer: %w", err)
	}

	return nil
}

// expiredTxs returns expired tx hashes based on the provided block time.
func (m *Manager) expiredTxs(blockTime time.Time) []TxHash {
	m.mu.RLock()
	defer m.mu.RUnlock()

	var result []TxHash
	for txHash, timestamp := range m.txHashes {
		if blockTime.After(timestamp) {
			result = append(result, txHash)
		}
	}

	return result
}

func (m *Manager) purge(txHashes []TxHash) {
	m.mu.Lock()
	defer m.mu.Unlock()

	for _, txHash := range txHashes {
		delete(m.txHashes, txHash)
	}
}

// purgeLoop removes expired tx hashes in the background
func (m *Manager) purgeLoop() {
	for {
		latestTime, ok := m.batchReceive()
		if !ok {
			// channel closed
			m.doneCh <- struct{}{}
			return
		}

		hashes := m.expiredTxs(latestTime)
		if len(hashes) > 0 {
			m.purge(hashes)
		}
	}
}

// batchReceive receives block time from the channel until the context is done
// or the channel is closed.
func (m *Manager) batchReceive() (time.Time, bool) {
	ctx, cancel := context.WithTimeout(context.Background(), 5*time.Second)
	defer cancel()

	var latestTime time.Time
	for {
		select {
		case <-ctx.Done():
			return latestTime, true

		case blockTime, ok := <-m.blockCh:
			if !ok {
				// channel is closed
				return time.Time{}, false
			}

			if blockTime.After(latestTime) {
				latestTime = blockTime
			}
		}
	}
}

func unorderedTxToBytes(txHash TxHash, ttl uint64) []byte {
	chunk := make([]byte, chunkSize)
	copy(chunk[:txHashSize], txHash[:])

	ttlBz := make([]byte, timeoutSize)
	binary.BigEndian.PutUint64(ttlBz, ttl)
	copy(chunk[txHashSize:], ttlBz)

	return chunk
}
