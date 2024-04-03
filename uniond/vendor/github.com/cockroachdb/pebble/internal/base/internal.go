// Copyright 2011 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package base // import "github.com/cockroachdb/pebble/internal/base"

import (
	"encoding/binary"
	"fmt"
	"strconv"
	"strings"

	"github.com/cockroachdb/redact"
)

const (
	// SeqNumZero is the zero sequence number, set by compactions if they can
	// guarantee there are no keys underneath an internal key.
	SeqNumZero = uint64(0)
	// SeqNumStart is the first sequence number assigned to a key. Sequence
	// numbers 1-9 are reserved for potential future use.
	SeqNumStart = uint64(10)
)

// InternalKeyKind enumerates the kind of key: a deletion tombstone, a set
// value, a merged value, etc.
type InternalKeyKind uint8

// These constants are part of the file format, and should not be changed.
const (
	InternalKeyKindDelete  InternalKeyKind = 0
	InternalKeyKindSet     InternalKeyKind = 1
	InternalKeyKindMerge   InternalKeyKind = 2
	InternalKeyKindLogData InternalKeyKind = 3
	//InternalKeyKindColumnFamilyDeletion     InternalKeyKind = 4
	//InternalKeyKindColumnFamilyValue        InternalKeyKind = 5
	//InternalKeyKindColumnFamilyMerge        InternalKeyKind = 6

	// InternalKeyKindSingleDelete (SINGLEDEL) is a performance optimization
	// solely for compactions (to reduce write amp and space amp). Readers other
	// than compactions should treat SINGLEDEL as equivalent to a DEL.
	// Historically, it was simpler for readers other than compactions to treat
	// SINGLEDEL as equivalent to DEL, but as of the introduction of
	// InternalKeyKindSSTableInternalObsoleteBit, this is also necessary for
	// correctness.
	InternalKeyKindSingleDelete InternalKeyKind = 7
	//InternalKeyKindColumnFamilySingleDelete InternalKeyKind = 8
	//InternalKeyKindBeginPrepareXID          InternalKeyKind = 9
	//InternalKeyKindEndPrepareXID            InternalKeyKind = 10
	//InternalKeyKindCommitXID                InternalKeyKind = 11
	//InternalKeyKindRollbackXID              InternalKeyKind = 12
	//InternalKeyKindNoop                     InternalKeyKind = 13
	//InternalKeyKindColumnFamilyRangeDelete  InternalKeyKind = 14
	InternalKeyKindRangeDelete InternalKeyKind = 15
	//InternalKeyKindColumnFamilyBlobIndex    InternalKeyKind = 16
	//InternalKeyKindBlobIndex                InternalKeyKind = 17

	// InternalKeyKindSeparator is a key used for separator / successor keys
	// written to sstable block indexes.
	//
	// NOTE: the RocksDB value has been repurposed. This was done to ensure that
	// keys written to block indexes with value "17" (when 17 happened to be the
	// max value, and InternalKeyKindMax was therefore set to 17), remain stable
	// when new key kinds are supported in Pebble.
	InternalKeyKindSeparator InternalKeyKind = 17

	// InternalKeyKindSetWithDelete keys are SET keys that have met with a
	// DELETE or SINGLEDEL key in a prior compaction. This key kind is
	// specific to Pebble. See
	// https://github.com/cockroachdb/pebble/issues/1255.
	InternalKeyKindSetWithDelete InternalKeyKind = 18

	// InternalKeyKindRangeKeyDelete removes all range keys within a key range.
	// See the internal/rangekey package for more details.
	InternalKeyKindRangeKeyDelete InternalKeyKind = 19
	// InternalKeyKindRangeKeySet and InternalKeyKindRangeUnset represent
	// keys that set and unset values associated with ranges of key
	// space. See the internal/rangekey package for more details.
	InternalKeyKindRangeKeyUnset InternalKeyKind = 20
	InternalKeyKindRangeKeySet   InternalKeyKind = 21

	// InternalKeyKindIngestSST is used to distinguish a batch that corresponds to
	// the WAL entry for ingested sstables that are added to the flushable
	// queue. This InternalKeyKind cannot appear, amongst other key kinds in a
	// batch, or in an sstable.
	InternalKeyKindIngestSST InternalKeyKind = 22

	// InternalKeyKindDeleteSized keys behave identically to
	// InternalKeyKindDelete keys, except that they hold an associated uint64
	// value indicating the (len(key)+len(value)) of the shadowed entry the
	// tombstone is expected to delete. This value is used to inform compaction
	// heuristics, but is not required to be accurate for correctness.
	InternalKeyKindDeleteSized InternalKeyKind = 23

	// This maximum value isn't part of the file format. Future extensions may
	// increase this value.
	//
	// When constructing an internal key to pass to DB.Seek{GE,LE},
	// internalKeyComparer sorts decreasing by kind (after sorting increasing by
	// user key and decreasing by sequence number). Thus, use InternalKeyKindMax,
	// which sorts 'less than or equal to' any other valid internalKeyKind, when
	// searching for any kind of internal key formed by a certain user key and
	// seqNum.
	InternalKeyKindMax InternalKeyKind = 23

	// Internal to the sstable format. Not exposed by any sstable iterator.
	// Declared here to prevent definition of valid key kinds that set this bit.
	InternalKeyKindSSTableInternalObsoleteBit  InternalKeyKind = 64
	InternalKeyKindSSTableInternalObsoleteMask InternalKeyKind = 191

	// InternalKeyZeroSeqnumMaxTrailer is the largest trailer with a
	// zero sequence number.
	InternalKeyZeroSeqnumMaxTrailer = uint64(255)

	// A marker for an invalid key.
	InternalKeyKindInvalid InternalKeyKind = InternalKeyKindSSTableInternalObsoleteMask

	// InternalKeySeqNumBatch is a bit that is set on batch sequence numbers
	// which prevents those entries from being excluded from iteration.
	InternalKeySeqNumBatch = uint64(1 << 55)

	// InternalKeySeqNumMax is the largest valid sequence number.
	InternalKeySeqNumMax = uint64(1<<56 - 1)

	// InternalKeyRangeDeleteSentinel is the marker for a range delete sentinel
	// key. This sequence number and kind are used for the upper stable boundary
	// when a range deletion tombstone is the largest key in an sstable. This is
	// necessary because sstable boundaries are inclusive, while the end key of a
	// range deletion tombstone is exclusive.
	InternalKeyRangeDeleteSentinel = (InternalKeySeqNumMax << 8) | uint64(InternalKeyKindRangeDelete)

	// InternalKeyBoundaryRangeKey is the marker for a range key boundary. This
	// sequence number and kind are used during interleaved range key and point
	// iteration to allow an iterator to stop at range key start keys where
	// there exists no point key.
	InternalKeyBoundaryRangeKey = (InternalKeySeqNumMax << 8) | uint64(InternalKeyKindRangeKeySet)
)

// Assert InternalKeyKindSSTableInternalObsoleteBit > InternalKeyKindMax
const _ = uint(InternalKeyKindSSTableInternalObsoleteBit - InternalKeyKindMax - 1)

var internalKeyKindNames = []string{
	InternalKeyKindDelete:         "DEL",
	InternalKeyKindSet:            "SET",
	InternalKeyKindMerge:          "MERGE",
	InternalKeyKindLogData:        "LOGDATA",
	InternalKeyKindSingleDelete:   "SINGLEDEL",
	InternalKeyKindRangeDelete:    "RANGEDEL",
	InternalKeyKindSeparator:      "SEPARATOR",
	InternalKeyKindSetWithDelete:  "SETWITHDEL",
	InternalKeyKindRangeKeySet:    "RANGEKEYSET",
	InternalKeyKindRangeKeyUnset:  "RANGEKEYUNSET",
	InternalKeyKindRangeKeyDelete: "RANGEKEYDEL",
	InternalKeyKindIngestSST:      "INGESTSST",
	InternalKeyKindDeleteSized:    "DELSIZED",
	InternalKeyKindInvalid:        "INVALID",
}

func (k InternalKeyKind) String() string {
	if int(k) < len(internalKeyKindNames) {
		return internalKeyKindNames[k]
	}
	return fmt.Sprintf("UNKNOWN:%d", k)
}

// SafeFormat implements redact.SafeFormatter.
func (k InternalKeyKind) SafeFormat(w redact.SafePrinter, _ rune) {
	w.Print(redact.SafeString(k.String()))
}

// InternalKey is a key used for the in-memory and on-disk partial DBs that
// make up a pebble DB.
//
// It consists of the user key (as given by the code that uses package pebble)
// followed by 8-bytes of metadata:
//   - 1 byte for the type of internal key: delete or set,
//   - 7 bytes for a uint56 sequence number, in little-endian format.
type InternalKey struct {
	UserKey []byte
	Trailer uint64
}

// InvalidInternalKey is an invalid internal key for which Valid() will return
// false.
var InvalidInternalKey = MakeInternalKey(nil, 0, InternalKeyKindInvalid)

// MakeInternalKey constructs an internal key from a specified user key,
// sequence number and kind.
func MakeInternalKey(userKey []byte, seqNum uint64, kind InternalKeyKind) InternalKey {
	return InternalKey{
		UserKey: userKey,
		Trailer: (seqNum << 8) | uint64(kind),
	}
}

// MakeTrailer constructs an internal key trailer from the specified sequence
// number and kind.
func MakeTrailer(seqNum uint64, kind InternalKeyKind) uint64 {
	return (seqNum << 8) | uint64(kind)
}

// MakeSearchKey constructs an internal key that is appropriate for searching
// for a the specified user key. The search key contain the maximal sequence
// number and kind ensuring that it sorts before any other internal keys for
// the same user key.
func MakeSearchKey(userKey []byte) InternalKey {
	return InternalKey{
		UserKey: userKey,
		Trailer: (InternalKeySeqNumMax << 8) | uint64(InternalKeyKindMax),
	}
}

// MakeRangeDeleteSentinelKey constructs an internal key that is a range
// deletion sentinel key, used as the upper boundary for an sstable when a
// range deletion is the largest key in an sstable.
func MakeRangeDeleteSentinelKey(userKey []byte) InternalKey {
	return InternalKey{
		UserKey: userKey,
		Trailer: InternalKeyRangeDeleteSentinel,
	}
}

// MakeExclusiveSentinelKey constructs an internal key that is an
// exclusive sentinel key, used as the upper boundary for an sstable
// when a ranged key is the largest key in an sstable.
func MakeExclusiveSentinelKey(kind InternalKeyKind, userKey []byte) InternalKey {
	return InternalKey{
		UserKey: userKey,
		Trailer: (InternalKeySeqNumMax << 8) | uint64(kind),
	}
}

var kindsMap = map[string]InternalKeyKind{
	"DEL":           InternalKeyKindDelete,
	"SINGLEDEL":     InternalKeyKindSingleDelete,
	"RANGEDEL":      InternalKeyKindRangeDelete,
	"LOGDATA":       InternalKeyKindLogData,
	"SET":           InternalKeyKindSet,
	"MERGE":         InternalKeyKindMerge,
	"INVALID":       InternalKeyKindInvalid,
	"SEPARATOR":     InternalKeyKindSeparator,
	"SETWITHDEL":    InternalKeyKindSetWithDelete,
	"RANGEKEYSET":   InternalKeyKindRangeKeySet,
	"RANGEKEYUNSET": InternalKeyKindRangeKeyUnset,
	"RANGEKEYDEL":   InternalKeyKindRangeKeyDelete,
	"INGESTSST":     InternalKeyKindIngestSST,
	"DELSIZED":      InternalKeyKindDeleteSized,
}

// ParseInternalKey parses the string representation of an internal key. The
// format is <user-key>.<kind>.<seq-num>. If the seq-num starts with a "b" it
// is marked as a batch-seq-num (i.e. the InternalKeySeqNumBatch bit is set).
func ParseInternalKey(s string) InternalKey {
	x := strings.Split(s, ".")
	ukey := x[0]
	kind, ok := kindsMap[x[1]]
	if !ok {
		panic(fmt.Sprintf("unknown kind: %q", x[1]))
	}
	j := 0
	if x[2][0] == 'b' {
		j = 1
	}
	seqNum, _ := strconv.ParseUint(x[2][j:], 10, 64)
	if x[2][0] == 'b' {
		seqNum |= InternalKeySeqNumBatch
	}
	return MakeInternalKey([]byte(ukey), seqNum, kind)
}

// ParseKind parses the string representation of an internal key kind.
func ParseKind(s string) InternalKeyKind {
	kind, ok := kindsMap[s]
	if !ok {
		panic(fmt.Sprintf("unknown kind: %q", s))
	}
	return kind
}

// InternalTrailerLen is the number of bytes used to encode InternalKey.Trailer.
const InternalTrailerLen = 8

// DecodeInternalKey decodes an encoded internal key. See InternalKey.Encode().
func DecodeInternalKey(encodedKey []byte) InternalKey {
	n := len(encodedKey) - InternalTrailerLen
	var trailer uint64
	if n >= 0 {
		trailer = binary.LittleEndian.Uint64(encodedKey[n:])
		encodedKey = encodedKey[:n:n]
	} else {
		trailer = uint64(InternalKeyKindInvalid)
		encodedKey = nil
	}
	return InternalKey{
		UserKey: encodedKey,
		Trailer: trailer,
	}
}

// InternalCompare compares two internal keys using the specified comparison
// function. For equal user keys, internal keys compare in descending sequence
// number order. For equal user keys and sequence numbers, internal keys
// compare in descending kind order (this may happen in practice among range
// keys).
func InternalCompare(userCmp Compare, a, b InternalKey) int {
	if x := userCmp(a.UserKey, b.UserKey); x != 0 {
		return x
	}
	if a.Trailer > b.Trailer {
		return -1
	}
	if a.Trailer < b.Trailer {
		return 1
	}
	return 0
}

// Encode encodes the receiver into the buffer. The buffer must be large enough
// to hold the encoded data. See InternalKey.Size().
func (k InternalKey) Encode(buf []byte) {
	i := copy(buf, k.UserKey)
	binary.LittleEndian.PutUint64(buf[i:], k.Trailer)
}

// EncodeTrailer returns the trailer encoded to an 8-byte array.
func (k InternalKey) EncodeTrailer() [8]byte {
	var buf [8]byte
	binary.LittleEndian.PutUint64(buf[:], k.Trailer)
	return buf
}

// Separator returns a separator key such that k <= x && x < other, where less
// than is consistent with the Compare function. The buf parameter may be used
// to store the returned InternalKey.UserKey, though it is valid to pass a
// nil. See the Separator type for details on separator keys.
func (k InternalKey) Separator(
	cmp Compare, sep Separator, buf []byte, other InternalKey,
) InternalKey {
	buf = sep(buf, k.UserKey, other.UserKey)
	if len(buf) <= len(k.UserKey) && cmp(k.UserKey, buf) < 0 {
		// The separator user key is physically shorter than k.UserKey (if it is
		// longer, we'll continue to use "k"), but logically after. Tack on the max
		// sequence number to the shortened user key. Note that we could tack on
		// any sequence number and kind here to create a valid separator key. We
		// use the max sequence number to match the behavior of LevelDB and
		// RocksDB.
		return MakeInternalKey(buf, InternalKeySeqNumMax, InternalKeyKindSeparator)
	}
	return k
}

// Successor returns a successor key such that k <= x. A simple implementation
// may return k unchanged. The buf parameter may be used to store the returned
// InternalKey.UserKey, though it is valid to pass a nil.
func (k InternalKey) Successor(cmp Compare, succ Successor, buf []byte) InternalKey {
	buf = succ(buf, k.UserKey)
	if len(buf) <= len(k.UserKey) && cmp(k.UserKey, buf) < 0 {
		// The successor user key is physically shorter that k.UserKey (if it is
		// longer, we'll continue to use "k"), but logically after. Tack on the max
		// sequence number to the shortened user key. Note that we could tack on
		// any sequence number and kind here to create a valid separator key. We
		// use the max sequence number to match the behavior of LevelDB and
		// RocksDB.
		return MakeInternalKey(buf, InternalKeySeqNumMax, InternalKeyKindSeparator)
	}
	return k
}

// Size returns the encoded size of the key.
func (k InternalKey) Size() int {
	return len(k.UserKey) + 8
}

// SetSeqNum sets the sequence number component of the key.
func (k *InternalKey) SetSeqNum(seqNum uint64) {
	k.Trailer = (seqNum << 8) | (k.Trailer & 0xff)
}

// SeqNum returns the sequence number component of the key.
func (k InternalKey) SeqNum() uint64 {
	return k.Trailer >> 8
}

// SeqNumFromTrailer returns the sequence number component of a trailer.
func SeqNumFromTrailer(t uint64) uint64 {
	return t >> 8
}

// Visible returns true if the key is visible at the specified snapshot
// sequence number.
func (k InternalKey) Visible(snapshot, batchSnapshot uint64) bool {
	return Visible(k.SeqNum(), snapshot, batchSnapshot)
}

// Visible returns true if a key with the provided sequence number is visible at
// the specified snapshot sequence numbers.
func Visible(seqNum uint64, snapshot, batchSnapshot uint64) bool {
	// There are two snapshot sequence numbers, one for committed keys and one
	// for batch keys. If a seqNum is less than `snapshot`, then seqNum
	// corresponds to a committed key that is visible. If seqNum has its batch
	// bit set, then seqNum corresponds to an uncommitted batch key. Its
	// visible if its snapshot is less than batchSnapshot.
	//
	// There's one complication. The maximal sequence number
	// (`InternalKeySeqNumMax`) is used across Pebble for exclusive sentinel
	// keys and other purposes. The maximal sequence number has its batch bit
	// set, but it can never be < `batchSnapshot`, since there is no expressible
	// larger snapshot. We dictate that the maximal sequence number is always
	// visible.
	return seqNum < snapshot ||
		((seqNum&InternalKeySeqNumBatch) != 0 && seqNum < batchSnapshot) ||
		seqNum == InternalKeySeqNumMax
}

// SetKind sets the kind component of the key.
func (k *InternalKey) SetKind(kind InternalKeyKind) {
	k.Trailer = (k.Trailer &^ 0xff) | uint64(kind)
}

// Kind returns the kind component of the key.
func (k InternalKey) Kind() InternalKeyKind {
	return TrailerKind(k.Trailer)
}

// TrailerKind returns the key kind of the key trailer.
func TrailerKind(trailer uint64) InternalKeyKind {
	return InternalKeyKind(trailer & 0xff)
}

// Valid returns true if the key has a valid kind.
func (k InternalKey) Valid() bool {
	return k.Kind() <= InternalKeyKindMax
}

// Clone clones the storage for the UserKey component of the key.
func (k InternalKey) Clone() InternalKey {
	if len(k.UserKey) == 0 {
		return k
	}
	return InternalKey{
		UserKey: append([]byte(nil), k.UserKey...),
		Trailer: k.Trailer,
	}
}

// CopyFrom converts this InternalKey into a clone of the passed-in InternalKey,
// reusing any space already used for the current UserKey.
func (k *InternalKey) CopyFrom(k2 InternalKey) {
	k.UserKey = append(k.UserKey[:0], k2.UserKey...)
	k.Trailer = k2.Trailer
}

// String returns a string representation of the key.
func (k InternalKey) String() string {
	return fmt.Sprintf("%s#%d,%d", FormatBytes(k.UserKey), k.SeqNum(), k.Kind())
}

// Pretty returns a formatter for the key.
func (k InternalKey) Pretty(f FormatKey) fmt.Formatter {
	return prettyInternalKey{k, f}
}

// IsExclusiveSentinel returns whether this internal key excludes point keys
// with the same user key if used as an end boundary. See the comment on
// InternalKeyRangeDeletionSentinel.
func (k InternalKey) IsExclusiveSentinel() bool {
	switch kind := k.Kind(); kind {
	case InternalKeyKindRangeDelete:
		return k.Trailer == InternalKeyRangeDeleteSentinel
	case InternalKeyKindRangeKeyDelete, InternalKeyKindRangeKeyUnset, InternalKeyKindRangeKeySet:
		return (k.Trailer >> 8) == InternalKeySeqNumMax
	default:
		return false
	}
}

type prettyInternalKey struct {
	InternalKey
	formatKey FormatKey
}

func (k prettyInternalKey) Format(s fmt.State, c rune) {
	if seqNum := k.SeqNum(); seqNum == InternalKeySeqNumMax {
		fmt.Fprintf(s, "%s#inf,%s", k.formatKey(k.UserKey), k.Kind())
	} else {
		fmt.Fprintf(s, "%s#%d,%s", k.formatKey(k.UserKey), k.SeqNum(), k.Kind())
	}
}

// ParsePrettyInternalKey parses the pretty string representation of an
// internal key. The format is <user-key>#<seq-num>,<kind>.
func ParsePrettyInternalKey(s string) InternalKey {
	x := strings.FieldsFunc(s, func(c rune) bool { return c == '#' || c == ',' })
	ukey := x[0]
	kind, ok := kindsMap[x[2]]
	if !ok {
		panic(fmt.Sprintf("unknown kind: %q", x[2]))
	}
	var seqNum uint64
	if x[1] == "max" || x[1] == "inf" {
		seqNum = InternalKeySeqNumMax
	} else {
		seqNum, _ = strconv.ParseUint(x[1], 10, 64)
	}
	return MakeInternalKey([]byte(ukey), seqNum, kind)
}
