package db

import (
	"encoding/binary"
	"fmt"
	"regexp"
	"strconv"

	dbm "github.com/cometbft/cometbft-db"

	cmtsync "github.com/cometbft/cometbft/libs/sync"
	"github.com/cometbft/cometbft/light/store"
	cmtproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/cometbft/cometbft/types"
)

var sizeKey = []byte("size")

type dbs struct {
	db     dbm.DB
	prefix string

	mtx  cmtsync.RWMutex
	size uint16
}

// New returns a Store that wraps any DB (with an optional prefix in case you
// want to use one DB with many light clients).
func New(db dbm.DB, prefix string) store.Store {
	size := uint16(0)
	bz, err := db.Get(sizeKey)
	if err == nil && len(bz) > 0 {
		size = unmarshalSize(bz)
	}

	return &dbs{db: db, prefix: prefix, size: size}
}

// SaveLightBlock persists LightBlock to the db.
//
// Safe for concurrent use by multiple goroutines.
func (s *dbs) SaveLightBlock(lb *types.LightBlock) error {
	if lb.Height <= 0 {
		panic("negative or zero height")
	}

	lbpb, err := lb.ToProto()
	if err != nil {
		return fmt.Errorf("unable to convert light block to protobuf: %w", err)
	}

	lbBz, err := lbpb.Marshal()
	if err != nil {
		return fmt.Errorf("marshaling LightBlock: %w", err)
	}

	s.mtx.Lock()
	defer s.mtx.Unlock()

	b := s.db.NewBatch()
	defer b.Close()
	if err = b.Set(s.lbKey(lb.Height), lbBz); err != nil {
		return err
	}
	if err = b.Set(sizeKey, marshalSize(s.size+1)); err != nil {
		return err
	}
	if err = b.WriteSync(); err != nil {
		return err
	}
	s.size++

	return nil
}

// DeleteLightBlockAndValidatorSet deletes the LightBlock from
// the db.
//
// Safe for concurrent use by multiple goroutines.
func (s *dbs) DeleteLightBlock(height int64) error {
	if height <= 0 {
		panic("negative or zero height")
	}

	s.mtx.Lock()
	defer s.mtx.Unlock()

	b := s.db.NewBatch()
	defer b.Close()
	if err := b.Delete(s.lbKey(height)); err != nil {
		return err
	}
	if err := b.Set(sizeKey, marshalSize(s.size-1)); err != nil {
		return err
	}
	if err := b.WriteSync(); err != nil {
		return err
	}
	s.size--

	return nil
}

// LightBlock retrieves the LightBlock at the given height.
//
// Safe for concurrent use by multiple goroutines.
func (s *dbs) LightBlock(height int64) (*types.LightBlock, error) {
	if height <= 0 {
		panic("negative or zero height")
	}

	bz, err := s.db.Get(s.lbKey(height))
	if err != nil {
		panic(err)
	}
	if len(bz) == 0 {
		return nil, store.ErrLightBlockNotFound
	}

	var lbpb cmtproto.LightBlock
	err = lbpb.Unmarshal(bz)
	if err != nil {
		return nil, fmt.Errorf("unmarshal error: %w", err)
	}

	lightBlock, err := types.LightBlockFromProto(&lbpb)
	if err != nil {
		return nil, fmt.Errorf("proto conversion error: %w", err)
	}

	return lightBlock, err
}

// LastLightBlockHeight returns the last LightBlock height stored.
//
// Safe for concurrent use by multiple goroutines.
func (s *dbs) LastLightBlockHeight() (int64, error) {
	itr, err := s.db.ReverseIterator(
		s.lbKey(1),
		append(s.lbKey(1<<63-1), byte(0x00)),
	)
	if err != nil {
		panic(err)
	}
	defer itr.Close()

	for itr.Valid() {
		key := itr.Key()
		_, height, ok := parseLbKey(key)
		if ok {
			return height, nil
		}
		itr.Next()
	}

	return -1, itr.Error()
}

// FirstLightBlockHeight returns the first LightBlock height stored.
//
// Safe for concurrent use by multiple goroutines.
func (s *dbs) FirstLightBlockHeight() (int64, error) {
	itr, err := s.db.Iterator(
		s.lbKey(1),
		append(s.lbKey(1<<63-1), byte(0x00)),
	)
	if err != nil {
		panic(err)
	}
	defer itr.Close()

	for itr.Valid() {
		key := itr.Key()
		_, height, ok := parseLbKey(key)
		if ok {
			return height, nil
		}
		itr.Next()
	}

	return -1, itr.Error()
}

// LightBlockBefore iterates over light blocks until it finds a block before
// the given height. It returns ErrLightBlockNotFound if no such block exists.
//
// Safe for concurrent use by multiple goroutines.
func (s *dbs) LightBlockBefore(height int64) (*types.LightBlock, error) {
	if height <= 0 {
		panic("negative or zero height")
	}

	itr, err := s.db.ReverseIterator(
		s.lbKey(1),
		s.lbKey(height),
	)
	if err != nil {
		panic(err)
	}
	defer itr.Close()

	for itr.Valid() {
		key := itr.Key()
		_, existingHeight, ok := parseLbKey(key)
		if ok {
			return s.LightBlock(existingHeight)
		}
		itr.Next()
	}
	if err = itr.Error(); err != nil {
		return nil, err
	}

	return nil, store.ErrLightBlockNotFound
}

// Prune prunes header & validator set pairs until there are only size pairs
// left.
//
// Safe for concurrent use by multiple goroutines.
func (s *dbs) Prune(size uint16) error {
	// 1) Check how many we need to prune.
	s.mtx.RLock()
	sSize := s.size
	s.mtx.RUnlock()

	if sSize <= size { // nothing to prune
		return nil
	}
	numToPrune := sSize - size

	// 2) Iterate over headers and perform a batch operation.
	itr, err := s.db.Iterator(
		s.lbKey(1),
		append(s.lbKey(1<<63-1), byte(0x00)),
	)
	if err != nil {
		return err
	}
	defer itr.Close()

	b := s.db.NewBatch()
	defer b.Close()

	pruned := 0
	for itr.Valid() && numToPrune > 0 {
		key := itr.Key()
		_, height, ok := parseLbKey(key)
		if ok {
			if err = b.Delete(s.lbKey(height)); err != nil {
				return err
			}
		}
		itr.Next()
		numToPrune--
		pruned++
	}
	if err = itr.Error(); err != nil {
		return err
	}

	err = b.WriteSync()
	if err != nil {
		return err
	}

	// 3) Update size.
	s.mtx.Lock()
	defer s.mtx.Unlock()

	s.size -= uint16(pruned)

	if wErr := s.db.SetSync(sizeKey, marshalSize(s.size)); wErr != nil {
		return fmt.Errorf("failed to persist size: %w", wErr)
	}

	return nil
}

// Size returns the number of header & validator set pairs.
//
// Safe for concurrent use by multiple goroutines.
func (s *dbs) Size() uint16 {
	s.mtx.RLock()
	defer s.mtx.RUnlock()
	return s.size
}

func (s *dbs) lbKey(height int64) []byte {
	return []byte(fmt.Sprintf("lb/%s/%020d", s.prefix, height))
}

var keyPattern = regexp.MustCompile(`^(lb)/([^/]*)/([0-9]+)$`)

func parseKey(key []byte) (part string, prefix string, height int64, ok bool) {
	submatch := keyPattern.FindSubmatch(key)
	if submatch == nil {
		return "", "", 0, false
	}
	part = string(submatch[1])
	prefix = string(submatch[2])
	height, err := strconv.ParseInt(string(submatch[3]), 10, 64)
	if err != nil {
		return "", "", 0, false
	}
	ok = true // good!
	return
}

func parseLbKey(key []byte) (prefix string, height int64, ok bool) {
	var part string
	part, prefix, height, ok = parseKey(key)
	if part != "lb" {
		return "", 0, false
	}
	return
}

func marshalSize(size uint16) []byte {
	bs := make([]byte, 2)
	binary.LittleEndian.PutUint16(bs, size)
	return bs
}

func unmarshalSize(bz []byte) uint16 {
	return binary.LittleEndian.Uint16(bz)
}
