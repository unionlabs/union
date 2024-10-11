package metamorphic

import (
	"fmt"
	"sort"

	"github.com/cockroachdb/pebble"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/testkeys"
	"github.com/stretchr/testify/require"
)

// objKey is a tuple of (objID, key). This struct is used primarily as a map
// key for keyManager. Only writer objTags can occur here, i.e., dbTag and
// batchTag, since this is used for tracking the keys in a writer.
type objKey struct {
	id  objID
	key []byte
}

// makeObjKey returns a new objKey given and id and key.
func makeObjKey(id objID, key []byte) objKey {
	if id.tag() != dbTag && id.tag() != batchTag {
		panic("unexpected non-writer tag")
	}
	return objKey{id, key}
}

// String implements fmt.Stringer, returning a stable string representation of
// the objKey. This string is used as map key.
func (o objKey) String() string {
	return fmt.Sprintf("%s:%s", o.id, o.key)
}

type keyUpdate struct {
	deleted bool
	// metaTimestamp at which the write or delete op occurred.
	metaTimestamp int
}

// keyMeta is metadata associated with an (objID, key) pair, where objID is
// a writer containing the key.
type keyMeta struct {
	objKey

	// The number of Sets of the key in this writer.
	sets int
	// The number of Merges of the key in this writer.
	merges int
	// singleDel can be true only if sets <= 1 && merges == 0 and the
	// SingleDelete was added to this writer after the set.
	singleDel bool
	// The number of Deletes of the key in this writer.
	dels int
	// del can be true only if a Delete was added to this writer after the
	// Sets and Merges counted above.
	del bool

	// updateOps should always be ordered by non-decreasing metaTimestamp.
	// updateOps will not be updated if the key is range deleted. Therefore, it
	// is a best effort sequence of updates to the key. updateOps is used to
	// determine if an iterator created on the DB can read a certain key.
	updateOps []keyUpdate
}

func (m *keyMeta) clear() {
	m.sets = 0
	m.merges = 0
	m.singleDel = false
	m.del = false
	m.dels = 0
	m.updateOps = nil
}

// mergeInto merges this metadata this into the metadata for other.
func (m *keyMeta) mergeInto(keyManager *keyManager, other *keyMeta) {
	if other.del && !m.del {
		// m's Sets and Merges are later.
		if m.sets > 0 || m.merges > 0 {
			other.del = false
		}
	} else {
		other.del = m.del
	}
	// Sets, merges, dels are additive.
	other.sets += m.sets
	other.merges += m.merges
	other.dels += m.dels

	// Single deletes are preserved. This is valid since we are also
	// maintaining a global invariant that SingleDelete will only be added for
	// a key that has no inflight Sets or Merges (Sets have made their way to
	// the DB), and no subsequent Sets or Merges will happen until the
	// SingleDelete makes its way to the DB.
	other.singleDel = other.singleDel || m.singleDel
	if other.singleDel {
		if other.sets > 1 || other.merges > 0 || other.dels > 0 {
			panic(fmt.Sprintf("invalid sets %d or merges %d or dels %d",
				other.sets, other.merges, other.dels))
		}
	}

	// Determine if the key is visible or not after the keyMetas are merged.
	// TODO(bananabrick): We currently only care about key updates which make it
	// to the DB, since we only use key updates to determine if an iterator
	// can read a key in the DB. We could extend the timestamp system to add
	// support for iterators created on batches.
	if other.del || other.singleDel {
		other.updateOps = append(
			other.updateOps, keyUpdate{true, keyManager.nextMetaTimestamp()},
		)
	} else {
		other.updateOps = append(
			other.updateOps, keyUpdate{false, keyManager.nextMetaTimestamp()},
		)
	}
}

// keyManager tracks the write operations performed on keys in the generation
// phase of the metamorphic test. It makes the assumption that write
// operations do not fail, since that can cause the keyManager state to be not
// in-sync with the actual state of the writers. This assumption is needed to
// correctly decide when it is safe to generate a SingleDelete. This
// assumption is violated in a single place in the metamorphic test: ingestion
// of multiple batches. We sidestep this issue in a narrow way in
// generator.writerIngest by not ingesting multiple batches that contain
// deletes or single deletes, since loss of those specific operations on a key
// are what we cannot tolerate (doing SingleDelete on a key that has not been
// written to because the Set was lost is harmless).
type keyManager struct {
	comparer *base.Comparer

	// metaTimestamp is used to provide a ordering over certain operations like
	// iter creation, updates to keys. Keeping track of the timestamp allows us
	// to make determinations such as whether a key will be visible to an
	// iterator.
	metaTimestamp int

	// byObjKey tracks the state for each (writer, key) pair. It refers to the
	// same *keyMeta as in the byObj slices. Using a map allows for fast state
	// lookups when changing the state based on a writer operation on the key.
	byObjKey map[string]*keyMeta
	// List of keys per writer, and what has happened to it in that writer.
	// Will be transferred when needed.
	byObj map[objID][]*keyMeta

	// globalKeys represents all the keys that have been generated so far. Not
	// all these keys have been written to. globalKeys is sorted.
	globalKeys [][]byte
	// globalKeysMap contains the same keys as globalKeys. It ensures no
	// duplication, and contains the aggregate state of the key across all
	// writers, including inflight state that has not made its way to the DB
	// yet.The keyMeta.objKey is uninitialized.
	globalKeysMap map[string]*keyMeta
	// globalKeyPrefixes contains all the key prefixes (as defined by the
	// comparer's Split) generated so far. globalKeyPrefixes is sorted.
	globalKeyPrefixes [][]byte
	// globalKeyPrefixesMap contains the same keys as globalKeyPrefixes. It
	// ensures no duplication.
	globalKeyPrefixesMap map[string]struct{}

	// Using SingleDeletes imposes some constraints on the above state, and
	// causes some state transitions that help with generating complex but
	// correct sequences involving SingleDeletes.
	// - Generating a SingleDelete requires for that key: global.merges==0 &&
	//   global.sets==1 && global.dels==0 && !global.singleDel && (db.sets==1
	//   || writer.sets==1), where global represents the entry in
	//   globalKeysMap[key] and db represents the entry in
	//   byObjKey[makeObjKey(makeObjID(dbTag, 0), key)], and writer is the
	//   entry in byObjKey[makeObjKey(writerID, key)].
	//
	// - We do not track state changes due to range deletes, so one should
	//   think of these counts as upper bounds. Also we are not preventing
	//   interactions caused by concurrently in-flight range deletes and
	//   SingleDelete. This is acceptable since it does not cause
	//   non-determinism.
	//
	// - When the SingleDelete is generated, it is recorded as
	//   writer.singleDel=true and global.singleDel=true. No more write
	//   operations are permitted on this key until db.singleDel transitions
	//   to true.
	//
	// - When db.singleDel transitions to true, we are guaranteed that no
	//   writer other than the DB has any writes for this key. We set
	//   db.singleDel and global.singleDel to false and the corresponding sets
	//   and merges counts in global and db also to 0. This allows this key to
	//   fully participate again in write operations. This means we can
	//   generate sequences of the form:
	//   SET => SINGLEDEL => SET* => MERGE* => DEL
	//   SET => SINGLEDEL => SET => SINGLEDEL, among others.
	//
	// - The above logic is insufficient to generate sequences of the form
	//   SET => DEL => SET => SINGLEDEL
	//   To do this we need to track Deletes. When db.del transitions to true,
	//   we check if db.sets==global.sets && db.merges==global.merges &&
	//   db.dels==global.dels. If true, there are no in-flight
	//   sets/merges/deletes to this key. We then default initialize the
	//   global and db entries since one can behave as if this key was never
	//   written in this system. This enables the above sequence, among
	//   others.
}

func (k *keyManager) nextMetaTimestamp() int {
	ret := k.metaTimestamp
	k.metaTimestamp++
	return ret
}

var dbObjID objID = makeObjID(dbTag, 0)

// newKeyManager returns a pointer to a new keyManager. Callers should
// interact with this using addNewKey, eligible*Keys, update,
// canTolerateApplyFailure methods only.
func newKeyManager() *keyManager {
	m := &keyManager{
		comparer:             testkeys.Comparer,
		byObjKey:             make(map[string]*keyMeta),
		byObj:                make(map[objID][]*keyMeta),
		globalKeysMap:        make(map[string]*keyMeta),
		globalKeyPrefixesMap: make(map[string]struct{}),
	}
	m.byObj[dbObjID] = []*keyMeta{}
	return m
}

// addNewKey adds the given key to the key manager for global key tracking.
// Returns false iff this is not a new key.
func (k *keyManager) addNewKey(key []byte) bool {
	_, ok := k.globalKeysMap[string(key)]
	if ok {
		return false
	}
	keyString := string(key)
	insertSorted(k.comparer.Compare, &k.globalKeys, key)
	k.globalKeysMap[keyString] = &keyMeta{objKey: objKey{key: key}}

	prefixLen := k.comparer.Split(key)
	if _, ok := k.globalKeyPrefixesMap[keyString[:prefixLen]]; !ok {
		insertSorted(k.comparer.Compare, &k.globalKeyPrefixes, key[:prefixLen])
		k.globalKeyPrefixesMap[keyString[:prefixLen]] = struct{}{}
	}
	return true
}

// getOrInit returns the keyMeta for the (objID, key) pair, if it exists, else
// allocates, initializes and returns a new value.
func (k *keyManager) getOrInit(id objID, key []byte) *keyMeta {
	o := makeObjKey(id, key)
	m, ok := k.byObjKey[o.String()]
	if ok {
		return m
	}
	m = &keyMeta{objKey: makeObjKey(id, key)}
	// Initialize the key-to-meta index.
	k.byObjKey[o.String()] = m
	// Add to the id-to-metas slide.
	k.byObj[o.id] = append(k.byObj[o.id], m)
	return m
}

// contains returns true if the (objID, key) pair is tracked by the keyManager.
func (k *keyManager) contains(id objID, key []byte) bool {
	_, ok := k.byObjKey[makeObjKey(id, key).String()]
	return ok
}

// mergeKeysInto merges all metadata for all keys associated with the "from" ID
// with the metadata for keys associated with the "to" ID.
func (k *keyManager) mergeKeysInto(from, to objID) {
	msFrom, ok := k.byObj[from]
	if !ok {
		msFrom = []*keyMeta{}
		k.byObj[from] = msFrom
	}

	msTo, ok := k.byObj[to]
	if !ok {
		msTo = []*keyMeta{}
		k.byObj[to] = msTo
	}

	// Sort to facilitate a merge.
	sort.Slice(msFrom, func(i, j int) bool {
		return msFrom[i].String() < msFrom[j].String()
	})
	sort.Slice(msTo, func(i, j int) bool {
		return msTo[i].String() < msTo[j].String()
	})

	var msNew []*keyMeta
	var iTo int
	for _, m := range msFrom {
		// Move cursor on mTo forward.
		for iTo < len(msTo) && string(msTo[iTo].key) < string(m.key) {
			msNew = append(msNew, msTo[iTo])
			iTo++
		}

		var mTo *keyMeta
		if iTo < len(msTo) && string(msTo[iTo].key) == string(m.key) {
			mTo = msTo[iTo]
			iTo++
		} else {
			mTo = &keyMeta{objKey: makeObjKey(to, m.key)}
			k.byObjKey[mTo.String()] = mTo
		}

		m.mergeInto(k, mTo)
		msNew = append(msNew, mTo)

		delete(k.byObjKey, m.String()) // Unlink "from".
	}

	// Add any remaining items from the "to" set.
	for iTo < len(msTo) {
		msNew = append(msNew, msTo[iTo])
		iTo++
	}

	k.byObj[to] = msNew   // Update "to".
	delete(k.byObj, from) // Unlink "from".
}

func (k *keyManager) checkForDelOrSingleDelTransition(dbMeta *keyMeta, globalMeta *keyMeta) {
	if dbMeta.singleDel {
		if !globalMeta.singleDel {
			panic("inconsistency with globalMeta")
		}
		if dbMeta.del || globalMeta.del || dbMeta.dels > 0 || globalMeta.dels > 0 ||
			dbMeta.merges > 0 || globalMeta.merges > 0 || dbMeta.sets != 1 || globalMeta.sets != 1 {
			panic("inconsistency in metas when SingleDelete applied to DB")
		}
		dbMeta.clear()
		globalMeta.clear()
		return
	}
	if dbMeta.del && globalMeta.sets == dbMeta.sets && globalMeta.merges == dbMeta.merges &&
		globalMeta.dels == dbMeta.dels {
		if dbMeta.singleDel || globalMeta.singleDel {
			panic("Delete should not have happened given SingleDelete")
		}
		dbMeta.clear()
		globalMeta.clear()
	}
}

func (k *keyManager) checkForDelOrSingleDelTransitionInDB() {
	keys := k.byObj[dbObjID]
	for _, dbMeta := range keys {
		globalMeta := k.globalKeysMap[string(dbMeta.key)]
		k.checkForDelOrSingleDelTransition(dbMeta, globalMeta)
	}
}

// update updates the internal state of the keyManager according to the given
// op.
func (k *keyManager) update(o op) {
	switch s := o.(type) {
	case *setOp:
		meta := k.getOrInit(s.writerID, s.key)
		globalMeta := k.globalKeysMap[string(s.key)]
		meta.sets++ // Update the set count on this specific (id, key) pair.
		meta.del = false
		globalMeta.sets++
		meta.updateOps = append(meta.updateOps, keyUpdate{false, k.nextMetaTimestamp()})
		if meta.singleDel || globalMeta.singleDel {
			panic("setting a key that has in-flight SingleDelete")
		}
	case *mergeOp:
		meta := k.getOrInit(s.writerID, s.key)
		globalMeta := k.globalKeysMap[string(s.key)]
		meta.merges++
		meta.del = false
		globalMeta.merges++
		meta.updateOps = append(meta.updateOps, keyUpdate{false, k.nextMetaTimestamp()})
		if meta.singleDel || globalMeta.singleDel {
			panic("merging a key that has in-flight SingleDelete")
		}
	case *deleteOp:
		meta := k.getOrInit(s.writerID, s.key)
		globalMeta := k.globalKeysMap[string(s.key)]
		meta.del = true
		globalMeta.del = true
		meta.dels++
		globalMeta.dels++
		meta.updateOps = append(meta.updateOps, keyUpdate{true, k.nextMetaTimestamp()})
		if s.writerID == dbObjID {
			k.checkForDelOrSingleDelTransition(meta, globalMeta)
		}
	case *singleDeleteOp:
		if !k.globalStateIndicatesEligibleForSingleDelete(s.key) {
			panic("key ineligible for SingleDelete")
		}
		meta := k.getOrInit(s.writerID, s.key)
		globalMeta := k.globalKeysMap[string(s.key)]
		meta.singleDel = true
		globalMeta.singleDel = true
		meta.updateOps = append(meta.updateOps, keyUpdate{true, k.nextMetaTimestamp()})
		if s.writerID == dbObjID {
			k.checkForDelOrSingleDelTransition(meta, globalMeta)
		}
	case *ingestOp:
		// For each batch, merge all keys with the keys in the DB.
		for _, batchID := range s.batchIDs {
			k.mergeKeysInto(batchID, dbObjID)
		}
		k.checkForDelOrSingleDelTransitionInDB()
	case *applyOp:
		// Merge the keys from this writer into the parent writer.
		k.mergeKeysInto(s.batchID, s.writerID)
		if s.writerID == dbObjID {
			k.checkForDelOrSingleDelTransitionInDB()
		}
	case *batchCommitOp:
		// Merge the keys from the batch with the keys from the DB.
		k.mergeKeysInto(s.batchID, dbObjID)
		k.checkForDelOrSingleDelTransitionInDB()
	}
}

func (k *keyManager) eligibleReadKeys() (keys [][]byte) {
	return k.globalKeys
}

// eligibleReadKeysInRange returns all eligible read keys within the range
// [start,end). The returned slice is owned by the keyManager and must not be
// retained.
func (k *keyManager) eligibleReadKeysInRange(kr pebble.KeyRange) (keys [][]byte) {
	s := sort.Search(len(k.globalKeys), func(i int) bool {
		return k.comparer.Compare(k.globalKeys[i], kr.Start) >= 0
	})
	e := sort.Search(len(k.globalKeys), func(i int) bool {
		return k.comparer.Compare(k.globalKeys[i], kr.End) >= 0
	})
	if s >= e {
		return nil
	}
	return k.globalKeys[s:e]
}

func (k *keyManager) prefixes() (prefixes [][]byte) {
	return k.globalKeyPrefixes
}

// prefixExists returns true if a key has been generated with the provided
// prefix before.
func (k *keyManager) prefixExists(prefix []byte) bool {
	_, exists := k.globalKeyPrefixesMap[string(prefix)]
	return exists
}

func (k *keyManager) eligibleWriteKeys() (keys [][]byte) {
	// Creating and sorting this slice of keys is wasteful given that the
	// caller will pick one, but makes it simpler for unit testing.
	for _, v := range k.globalKeysMap {
		if v.singleDel {
			continue
		}
		keys = append(keys, v.key)
	}
	sort.Slice(keys, func(i, j int) bool {
		return k.comparer.Compare(keys[i], keys[j]) < 0
	})
	return keys
}

// eligibleSingleDeleteKeys returns a slice of keys that can be safely single
// deleted, given the writer id.
func (k *keyManager) eligibleSingleDeleteKeys(id objID) (keys [][]byte) {
	// Creating and sorting this slice of keys is wasteful given that the
	// caller will pick one, but makes it simpler for unit testing.
	addForObjID := func(id objID) {
		for _, m := range k.byObj[id] {
			if m.sets == 1 && k.globalStateIndicatesEligibleForSingleDelete(m.key) {
				keys = append(keys, m.key)
			}
		}
	}
	addForObjID(id)
	if id != dbObjID {
		addForObjID(dbObjID)
	}
	sort.Slice(keys, func(i, j int) bool {
		return k.comparer.Compare(keys[i], keys[j]) < 0
	})
	return keys
}

func (k *keyManager) globalStateIndicatesEligibleForSingleDelete(key []byte) bool {
	m := k.globalKeysMap[string(key)]
	return m.merges == 0 && m.sets == 1 && m.dels == 0 && !m.singleDel
}

// canTolerateApplyFailure is called with a batch ID and returns true iff a
// failure to apply this batch to the DB can be tolerated.
func (k *keyManager) canTolerateApplyFailure(id objID) bool {
	if id.tag() != batchTag {
		panic("called with an objID that is not a batch")
	}
	ms, ok := k.byObj[id]
	if !ok {
		return true
	}
	for _, m := range ms {
		if m.singleDel || m.del {
			return false
		}
	}
	return true
}

func opWrittenKeys(untypedOp op) [][]byte {
	switch t := untypedOp.(type) {
	case *applyOp:
	case *batchCommitOp:
	case *checkpointOp:
	case *closeOp:
	case *compactOp:
	case *dbRestartOp:
	case *deleteOp:
		return [][]byte{t.key}
	case *deleteRangeOp:
		return [][]byte{t.start, t.end}
	case *flushOp:
	case *getOp:
	case *ingestOp:
	case *initOp:
	case *iterFirstOp:
	case *iterLastOp:
	case *iterNextOp:
	case *iterNextPrefixOp:
	case *iterCanSingleDelOp:
	case *iterPrevOp:
	case *iterSeekGEOp:
	case *iterSeekLTOp:
	case *iterSeekPrefixGEOp:
	case *iterSetBoundsOp:
	case *iterSetOptionsOp:
	case *mergeOp:
		return [][]byte{t.key}
	case *newBatchOp:
	case *newIndexedBatchOp:
	case *newIterOp:
	case *newIterUsingCloneOp:
	case *newSnapshotOp:
	case *rangeKeyDeleteOp:
	case *rangeKeySetOp:
	case *rangeKeyUnsetOp:
	case *setOp:
		return [][]byte{t.key}
	case *singleDeleteOp:
		return [][]byte{t.key}
	}
	return nil
}

func loadPrecedingKeys(t TestingT, ops []op, cfg *config, m *keyManager) {
	for _, op := range ops {
		// Pretend we're generating all the operation's keys as potential new
		// key, so that we update the key manager's keys and prefix sets.
		for _, k := range opWrittenKeys(op) {
			m.addNewKey(k)

			// If the key has a suffix, ratchet up the suffix distribution if
			// necessary.
			if s := m.comparer.Split(k); s < len(k) {
				suffix, err := testkeys.ParseSuffix(k[s:])
				require.NoError(t, err)
				if uint64(suffix) > cfg.writeSuffixDist.Max() {
					diff := int(uint64(suffix) - cfg.writeSuffixDist.Max())
					cfg.writeSuffixDist.IncMax(diff)
				}
			}
		}

		// Update key tracking state.
		m.update(op)
	}
}

func insertSorted(cmp base.Compare, dst *[][]byte, k []byte) {
	s := *dst
	i := sort.Search(len(*dst), func(i int) bool {
		return cmp((*dst)[i], k) >= 0
	})
	if i == len(s) {
		*dst = append(*dst, k)
		return
	}
	*dst = append((*dst)[:i+1], (*dst)[i:]...)
	(*dst)[i] = k
}
