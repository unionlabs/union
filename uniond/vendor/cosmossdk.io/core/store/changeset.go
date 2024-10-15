package store

import (
	"bytes"
)

// Changeset is a list of changes to be written to disk
type Changeset struct {
	Changes []StateChanges
}

// StateChanges represents a set of changes to the state of an actor in storage.
type StateChanges = struct {
	Actor        []byte  // actor represents the space in storage where state is stored, previously this was called a "storekey"
	StateChanges KVPairs // StateChanges is a list of key-value pairs representing the changes to the state.
}

// KVPairs represents a set of key-value pairs.
type KVPairs = []KVPair

// KVPair represents a change in a key and value of state.
// Remove being true signals the key must be removed from state.
type KVPair = struct {
	// Key defines the key being updated.
	Key []byte
	// Value defines the value associated with the updated key.
	Value []byte
	// Remove is true when the key must be removed from state.
	Remove bool
}

func NewChangeset() *Changeset {
	return &Changeset{}
}

func NewChangesetWithPairs(pairs map[string]KVPairs) *Changeset {
	changes := make([]StateChanges, len(pairs))
	i := 0
	for storeKey, kvPairs := range pairs {
		changes[i] = StateChanges{
			Actor:        []byte(storeKey),
			StateChanges: kvPairs,
		}
		i++
	}
	return &Changeset{
		Changes: changes,
	}
}

// Size returns the number of key-value pairs in the batch.
func (cs *Changeset) Size() int {
	cnt := 0
	for _, pairs := range cs.Changes {
		cnt += len(pairs.StateChanges)
	}

	return cnt
}

// Add adds a key-value pair to the ChangeSet.
func (cs *Changeset) Add(storeKey, key, value []byte, remove bool) {
	found := false
	for i, pairs := range cs.Changes {
		if bytes.Equal(storeKey, pairs.Actor) {
			pairs.StateChanges = append(pairs.StateChanges, KVPair{
				Key:    key,
				Value:  value,
				Remove: remove,
			})
			cs.Changes[i] = pairs
			found = true
			break
		}
	}

	if !found {
		cs.Changes = append(cs.Changes, StateChanges{
			Actor:        storeKey,
			StateChanges: []KVPair{{Key: key, Value: value, Remove: remove}},
		})
	}
}

// AddKVPair adds a KVPair to the ChangeSet.
func (cs *Changeset) AddKVPair(storeKey []byte, pair KVPair) {
	found := false
	for i, pairs := range cs.Changes {
		if bytes.Equal(storeKey, pairs.Actor) {
			pairs.StateChanges = append(pairs.StateChanges, pair)
			cs.Changes[i] = pairs
			found = true
			break
		}
	}

	if !found {
		cs.Changes = append(cs.Changes, StateChanges{
			Actor:        storeKey,
			StateChanges: []KVPair{pair},
		})
	}
}
