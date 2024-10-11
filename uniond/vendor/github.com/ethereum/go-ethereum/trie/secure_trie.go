// Copyright 2015 The go-ethereum Authors
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

package trie

import (
	"github.com/ethereum/go-ethereum/common"
	"github.com/ethereum/go-ethereum/core/types"
	"github.com/ethereum/go-ethereum/rlp"
	"github.com/ethereum/go-ethereum/trie/trienode"
)

// SecureTrie is the old name of StateTrie.
// Deprecated: use StateTrie.
type SecureTrie = StateTrie

// NewSecure creates a new StateTrie.
// Deprecated: use NewStateTrie.
func NewSecure(stateRoot common.Hash, owner common.Hash, root common.Hash, db *Database) (*SecureTrie, error) {
	id := &ID{
		StateRoot: stateRoot,
		Owner:     owner,
		Root:      root,
	}
	return NewStateTrie(id, db)
}

// StateTrie wraps a trie with key hashing. In a stateTrie trie, all
// access operations hash the key using keccak256. This prevents
// calling code from creating long chains of nodes that
// increase the access time.
//
// Contrary to a regular trie, a StateTrie can only be created with
// New and must have an attached database. The database also stores
// the preimage of each key if preimage recording is enabled.
//
// StateTrie is not safe for concurrent use.
type StateTrie struct {
	trie             Trie
	preimages        *preimageStore
	hashKeyBuf       [common.HashLength]byte
	secKeyCache      map[string][]byte
	secKeyCacheOwner *StateTrie // Pointer to self, replace the key cache on mismatch
}

// NewStateTrie creates a trie with an existing root node from a backing database.
//
// If root is the zero hash or the sha3 hash of an empty string, the
// trie is initially empty. Otherwise, New will panic if db is nil
// and returns MissingNodeError if the root node cannot be found.
func NewStateTrie(id *ID, db *Database) (*StateTrie, error) {
	if db == nil {
		panic("trie.NewStateTrie called without a database")
	}
	trie, err := New(id, db)
	if err != nil {
		return nil, err
	}
	return &StateTrie{trie: *trie, preimages: db.preimages}, nil
}

// MustGet returns the value for key stored in the trie.
// The value bytes must not be modified by the caller.
//
// This function will omit any encountered error but just
// print out an error message.
func (t *StateTrie) MustGet(key []byte) []byte {
	return t.trie.MustGet(t.hashKey(key))
}

// GetStorage attempts to retrieve a storage slot with provided account address
// and slot key. The value bytes must not be modified by the caller.
// If the specified storage slot is not in the trie, nil will be returned.
// If a trie node is not found in the database, a MissingNodeError is returned.
func (t *StateTrie) GetStorage(_ common.Address, key []byte) ([]byte, error) {
	enc, err := t.trie.Get(t.hashKey(key))
	if err != nil || len(enc) == 0 {
		return nil, err
	}
	_, content, _, err := rlp.Split(enc)
	return content, err
}

// GetAccount attempts to retrieve an account with provided account address.
// If the specified account is not in the trie, nil will be returned.
// If a trie node is not found in the database, a MissingNodeError is returned.
func (t *StateTrie) GetAccount(address common.Address) (*types.StateAccount, error) {
	res, err := t.trie.Get(t.hashKey(address.Bytes()))
	if res == nil || err != nil {
		return nil, err
	}
	ret := new(types.StateAccount)
	err = rlp.DecodeBytes(res, ret)
	return ret, err
}

// GetAccountByHash does the same thing as GetAccount, however it expects an
// account hash that is the hash of address. This constitutes an abstraction
// leak, since the client code needs to know the key format.
func (t *StateTrie) GetAccountByHash(addrHash common.Hash) (*types.StateAccount, error) {
	res, err := t.trie.Get(addrHash.Bytes())
	if res == nil || err != nil {
		return nil, err
	}
	ret := new(types.StateAccount)
	err = rlp.DecodeBytes(res, ret)
	return ret, err
}

// GetNode attempts to retrieve a trie node by compact-encoded path. It is not
// possible to use keybyte-encoding as the path might contain odd nibbles.
// If the specified trie node is not in the trie, nil will be returned.
// If a trie node is not found in the database, a MissingNodeError is returned.
func (t *StateTrie) GetNode(path []byte) ([]byte, int, error) {
	return t.trie.GetNode(path)
}

// MustUpdate associates key with value in the trie. Subsequent calls to
// Get will return value. If value has length zero, any existing value
// is deleted from the trie and calls to Get will return nil.
//
// The value bytes must not be modified by the caller while they are
// stored in the trie.
//
// This function will omit any encountered error but just print out an
// error message.
func (t *StateTrie) MustUpdate(key, value []byte) {
	hk := t.hashKey(key)
	t.trie.MustUpdate(hk, value)
	t.getSecKeyCache()[string(hk)] = common.CopyBytes(key)
}

// UpdateStorage associates key with value in the trie. Subsequent calls to
// Get will return value. If value has length zero, any existing value
// is deleted from the trie and calls to Get will return nil.
//
// The value bytes must not be modified by the caller while they are
// stored in the trie.
//
// If a node is not found in the database, a MissingNodeError is returned.
func (t *StateTrie) UpdateStorage(_ common.Address, key, value []byte) error {
	hk := t.hashKey(key)
	v, _ := rlp.EncodeToBytes(value)
	err := t.trie.Update(hk, v)
	if err != nil {
		return err
	}
	t.getSecKeyCache()[string(hk)] = common.CopyBytes(key)
	return nil
}

// UpdateAccount will abstract the write of an account to the secure trie.
func (t *StateTrie) UpdateAccount(address common.Address, acc *types.StateAccount) error {
	hk := t.hashKey(address.Bytes())
	data, err := rlp.EncodeToBytes(acc)
	if err != nil {
		return err
	}
	if err := t.trie.Update(hk, data); err != nil {
		return err
	}
	t.getSecKeyCache()[string(hk)] = address.Bytes()
	return nil
}

func (t *StateTrie) UpdateContractCode(_ common.Address, _ common.Hash, _ []byte) error {
	return nil
}

// MustDelete removes any existing value for key from the trie. This function
// will omit any encountered error but just print out an error message.
func (t *StateTrie) MustDelete(key []byte) {
	hk := t.hashKey(key)
	delete(t.getSecKeyCache(), string(hk))
	t.trie.MustDelete(hk)
}

// DeleteStorage removes any existing storage slot from the trie.
// If the specified trie node is not in the trie, nothing will be changed.
// If a node is not found in the database, a MissingNodeError is returned.
func (t *StateTrie) DeleteStorage(_ common.Address, key []byte) error {
	hk := t.hashKey(key)
	delete(t.getSecKeyCache(), string(hk))
	return t.trie.Delete(hk)
}

// DeleteAccount abstracts an account deletion from the trie.
func (t *StateTrie) DeleteAccount(address common.Address) error {
	hk := t.hashKey(address.Bytes())
	delete(t.getSecKeyCache(), string(hk))
	return t.trie.Delete(hk)
}

// GetKey returns the sha3 preimage of a hashed key that was
// previously used to store a value.
func (t *StateTrie) GetKey(shaKey []byte) []byte {
	if key, ok := t.getSecKeyCache()[string(shaKey)]; ok {
		return key
	}
	if t.preimages == nil {
		return nil
	}
	return t.preimages.preimage(common.BytesToHash(shaKey))
}

// Commit collects all dirty nodes in the trie and replaces them with the
// corresponding node hash. All collected nodes (including dirty leaves if
// collectLeaf is true) will be encapsulated into a nodeset for return.
// The returned nodeset can be nil if the trie is clean (nothing to commit).
// All cached preimages will be also flushed if preimages recording is enabled.
// Once the trie is committed, it's not usable anymore. A new trie must
// be created with new root and updated trie database for following usage
func (t *StateTrie) Commit(collectLeaf bool) (common.Hash, *trienode.NodeSet, error) {
	// Write all the pre-images to the actual disk database
	if len(t.getSecKeyCache()) > 0 {
		if t.preimages != nil {
			preimages := make(map[common.Hash][]byte)
			for hk, key := range t.secKeyCache {
				preimages[common.BytesToHash([]byte(hk))] = key
			}
			t.preimages.insertPreimage(preimages)
		}
		t.secKeyCache = make(map[string][]byte)
	}
	// Commit the trie and return its modified nodeset.
	return t.trie.Commit(collectLeaf)
}

// Hash returns the root hash of StateTrie. It does not write to the
// database and can be used even if the trie doesn't have one.
func (t *StateTrie) Hash() common.Hash {
	return t.trie.Hash()
}

// Copy returns a copy of StateTrie.
func (t *StateTrie) Copy() *StateTrie {
	return &StateTrie{
		trie:        *t.trie.Copy(),
		preimages:   t.preimages,
		secKeyCache: t.secKeyCache,
	}
}

// NodeIterator returns an iterator that returns nodes of the underlying trie.
// Iteration starts at the key after the given start key.
func (t *StateTrie) NodeIterator(start []byte) (NodeIterator, error) {
	return t.trie.NodeIterator(start)
}

// MustNodeIterator is a wrapper of NodeIterator and will omit any encountered
// error but just print out an error message.
func (t *StateTrie) MustNodeIterator(start []byte) NodeIterator {
	return t.trie.MustNodeIterator(start)
}

// hashKey returns the hash of key as an ephemeral buffer.
// The caller must not hold onto the return value because it will become
// invalid on the next call to hashKey or secKey.
func (t *StateTrie) hashKey(key []byte) []byte {
	h := newHasher(false)
	h.sha.Reset()
	h.sha.Write(key)
	h.sha.Read(t.hashKeyBuf[:])
	returnHasherToPool(h)
	return t.hashKeyBuf[:]
}

// getSecKeyCache returns the current secure key cache, creating a new one if
// ownership changed (i.e. the current secure trie is a copy of another owning
// the actual cache).
func (t *StateTrie) getSecKeyCache() map[string][]byte {
	if t != t.secKeyCacheOwner {
		t.secKeyCacheOwner = t
		t.secKeyCache = make(map[string][]byte)
	}
	return t.secKeyCache
}
