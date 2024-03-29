// Copyright 2018 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package sstable

import (
	"bytes"
	"encoding/binary"
	"fmt"
	"math"
	"reflect"
	"sort"
	"unsafe"

	"github.com/cockroachdb/pebble/internal/intern"
)

const propertiesBlockRestartInterval = math.MaxInt32
const propGlobalSeqnumName = "rocksdb.external_sst_file.global_seqno"

var propTagMap = make(map[string]reflect.StructField)
var propBoolTrue = []byte{'1'}
var propBoolFalse = []byte{'0'}

var propOffsetTagMap = make(map[uintptr]string)

func generateTagMaps(t reflect.Type, indexPrefix []int) {
	for i := 0; i < t.NumField(); i++ {
		f := t.Field(i)
		if f.Type.Kind() == reflect.Struct {
			if tag := f.Tag.Get("prop"); i == 0 && tag == "pebble.embbeded_common_properties" {
				// CommonProperties struct embedded in Properties. Note that since
				// CommonProperties is placed at the top of properties we can use
				// the offsets of the fields within CommonProperties to determine
				// the offsets of those fields within Properties.
				generateTagMaps(f.Type, []int{i})
				continue
			}
			panic("pebble: unknown struct type in Properties")
		}
		if tag := f.Tag.Get("prop"); tag != "" {
			switch f.Type.Kind() {
			case reflect.Bool:
			case reflect.Uint32:
			case reflect.Uint64:
			case reflect.String:
			default:
				panic(fmt.Sprintf("unsupported property field type: %s %s", f.Name, f.Type))
			}
			if len(indexPrefix) > 0 {
				// Prepend the index prefix so that we can use FieldByIndex on the top-level struct.
				f.Index = append(indexPrefix[:len(indexPrefix):len(indexPrefix)], f.Index...)
			}
			propTagMap[tag] = f
			propOffsetTagMap[f.Offset] = tag
		}
	}
}

func init() {
	generateTagMaps(reflect.TypeOf(Properties{}), nil)
}

// CommonProperties holds properties for either a virtual or a physical sstable. This
// can be used by code which doesn't care to make the distinction between physical
// and virtual sstables properties.
//
// For virtual sstables, fields are constructed through extrapolation upon virtual
// reader construction. See MakeVirtualReader for implementation details.
//
// NB: The values of these properties can affect correctness. For example,
// if NumRangeKeySets == 0, but the sstable actually contains range keys, then
// the iterators will behave incorrectly.
type CommonProperties struct {
	// The number of entries in this table.
	NumEntries uint64 `prop:"rocksdb.num.entries"`
	// Total raw key size.
	RawKeySize uint64 `prop:"rocksdb.raw.key.size"`
	// Total raw value size.
	RawValueSize uint64 `prop:"rocksdb.raw.value.size"`
	// Total raw key size of point deletion tombstones. This value is comparable
	// to RawKeySize.
	RawPointTombstoneKeySize uint64 `prop:"pebble.raw.point-tombstone.key.size"`
	// Sum of the raw value sizes carried by point deletion tombstones
	// containing size estimates. See the DeleteSized key kind. This value is
	// comparable to Raw{Key,Value}Size.
	RawPointTombstoneValueSize uint64 `prop:"pebble.raw.point-tombstone.value.size"`
	// The number of point deletion entries ("tombstones") in this table that
	// carry a size hint indicating the size of the value the tombstone deletes.
	NumSizedDeletions uint64 `prop:"pebble.num.deletions.sized"`
	// The number of deletion entries in this table, including both point and
	// range deletions.
	NumDeletions uint64 `prop:"rocksdb.deleted.keys"`
	// The number of range deletions in this table.
	NumRangeDeletions uint64 `prop:"rocksdb.num.range-deletions"`
	// The number of RANGEKEYDELs in this table.
	NumRangeKeyDels uint64 `prop:"pebble.num.range-key-dels"`
	// The number of RANGEKEYSETs in this table.
	NumRangeKeySets uint64 `prop:"pebble.num.range-key-sets"`
	// Total size of value blocks and value index block. Only serialized if > 0.
	ValueBlocksSize uint64 `prop:"pebble.value-blocks.size"`
}

// String is only used for testing purposes.
func (c *CommonProperties) String() string {
	var buf bytes.Buffer
	v := reflect.ValueOf(*c)
	loaded := make(map[uintptr]struct{})
	writeProperties(loaded, v, &buf)
	return buf.String()
}

// NumPointDeletions is the number of point deletions in the sstable. For virtual
// sstables, this is an estimate.
func (c *CommonProperties) NumPointDeletions() uint64 {
	return c.NumDeletions - c.NumRangeDeletions
}

// Properties holds the sstable property values. The properties are
// automatically populated during sstable creation and load from the properties
// meta block when an sstable is opened.
type Properties struct {
	// CommonProperties needs to be at the top of the Properties struct so that the
	// offsets of the fields in CommonProperties match the offsets of the embedded
	// fields of CommonProperties in Properties.
	CommonProperties `prop:"pebble.embbeded_common_properties"`

	// The name of the comparer used in this table.
	ComparerName string `prop:"rocksdb.comparator"`
	// The compression algorithm used to compress blocks.
	CompressionName string `prop:"rocksdb.compression"`
	// The compression options used to compress blocks.
	CompressionOptions string `prop:"rocksdb.compression_options"`
	// The total size of all data blocks.
	DataSize uint64 `prop:"rocksdb.data.size"`
	// The external sstable version format. Version 2 is the one RocksDB has been
	// using since 5.13. RocksDB only uses the global sequence number for an
	// sstable if this property has been set.
	ExternalFormatVersion uint32 `prop:"rocksdb.external_sst_file.version"`
	// The name of the filter policy used in this table. Empty if no filter
	// policy is used.
	FilterPolicyName string `prop:"rocksdb.filter.policy"`
	// The size of filter block.
	FilterSize uint64 `prop:"rocksdb.filter.size"`
	// The global sequence number to use for all entries in the table. Present if
	// the table was created externally and ingested whole.
	GlobalSeqNum uint64 `prop:"rocksdb.external_sst_file.global_seqno"`
	// Total number of index partitions if kTwoLevelIndexSearch is used.
	IndexPartitions uint64 `prop:"rocksdb.index.partitions"`
	// The size of index block.
	IndexSize uint64 `prop:"rocksdb.index.size"`
	// The index type. TODO(peter): add a more detailed description.
	IndexType uint32 `prop:"rocksdb.block.based.table.index.type"`
	// For formats >= TableFormatPebblev4, this is set to true if the obsolete
	// bit is strict for all the point keys.
	IsStrictObsolete bool `prop:"pebble.obsolete.is_strict"`
	// The name of the merger used in this table. Empty if no merger is used.
	MergerName string `prop:"rocksdb.merge.operator"`
	// The number of blocks in this table.
	NumDataBlocks uint64 `prop:"rocksdb.num.data.blocks"`
	// The number of merge operands in the table.
	NumMergeOperands uint64 `prop:"rocksdb.merge.operands"`
	// The number of RANGEKEYUNSETs in this table.
	NumRangeKeyUnsets uint64 `prop:"pebble.num.range-key-unsets"`
	// The number of value blocks in this table. Only serialized if > 0.
	NumValueBlocks uint64 `prop:"pebble.num.value-blocks"`
	// The number of values stored in value blocks. Only serialized if > 0.
	NumValuesInValueBlocks uint64 `prop:"pebble.num.values.in.value-blocks"`
	// The name of the prefix extractor used in this table. Empty if no prefix
	// extractor is used.
	PrefixExtractorName string `prop:"rocksdb.prefix.extractor.name"`
	// If filtering is enabled, was the filter created on the key prefix.
	PrefixFiltering bool `prop:"rocksdb.block.based.table.prefix.filtering"`
	// A comma separated list of names of the property collectors used in this
	// table.
	PropertyCollectorNames string `prop:"rocksdb.property.collectors"`
	// Total raw rangekey key size.
	RawRangeKeyKeySize uint64 `prop:"pebble.raw.range-key.key.size"`
	// Total raw rangekey value size.
	RawRangeKeyValueSize uint64 `prop:"pebble.raw.range-key.value.size"`
	// The total number of keys in this table that were pinned by open snapshots.
	SnapshotPinnedKeys uint64 `prop:"pebble.num.snapshot-pinned-keys"`
	// The cumulative bytes of keys in this table that were pinned by
	// open snapshots. This value is comparable to RawKeySize.
	SnapshotPinnedKeySize uint64 `prop:"pebble.raw.snapshot-pinned-keys.size"`
	// The cumulative bytes of values in this table that were pinned by
	// open snapshots. This value is comparable to RawValueSize.
	SnapshotPinnedValueSize uint64 `prop:"pebble.raw.snapshot-pinned-values.size"`
	// Size of the top-level index if kTwoLevelIndexSearch is used.
	TopLevelIndexSize uint64 `prop:"rocksdb.top-level.index.size"`
	// User collected properties.
	UserProperties map[string]string
	// If filtering is enabled, was the filter created on the whole key.
	WholeKeyFiltering bool `prop:"rocksdb.block.based.table.whole.key.filtering"`

	// Loaded set indicating which fields have been loaded from disk. Indexed by
	// the field's byte offset within the struct
	// (reflect.StructField.Offset). Only set if the properties have been loaded
	// from a file. Only exported for testing purposes.
	Loaded map[uintptr]struct{}
}

// NumPointDeletions returns the number of point deletions in this table.
func (p *Properties) NumPointDeletions() uint64 {
	return p.NumDeletions - p.NumRangeDeletions
}

// NumRangeKeys returns a count of the number of range keys in this table.
func (p *Properties) NumRangeKeys() uint64 {
	return p.NumRangeKeyDels + p.NumRangeKeySets + p.NumRangeKeyUnsets
}

func writeProperties(loaded map[uintptr]struct{}, v reflect.Value, buf *bytes.Buffer) {
	vt := v.Type()
	for i := 0; i < v.NumField(); i++ {
		ft := vt.Field(i)
		if ft.Type.Kind() == reflect.Struct {
			// Embedded struct within the properties.
			writeProperties(loaded, v.Field(i), buf)
			continue
		}
		tag := ft.Tag.Get("prop")
		if tag == "" {
			continue
		}

		f := v.Field(i)
		// TODO(peter): Use f.IsZero() when we can rely on go1.13.
		if zero := reflect.Zero(f.Type()); zero.Interface() == f.Interface() {
			// Skip printing of zero values which were not loaded from disk.
			if _, ok := loaded[ft.Offset]; !ok {
				continue
			}
		}

		fmt.Fprintf(buf, "%s: ", tag)
		switch ft.Type.Kind() {
		case reflect.Bool:
			fmt.Fprintf(buf, "%t\n", f.Bool())
		case reflect.Uint32:
			fmt.Fprintf(buf, "%d\n", f.Uint())
		case reflect.Uint64:
			fmt.Fprintf(buf, "%d\n", f.Uint())
		case reflect.String:
			fmt.Fprintf(buf, "%s\n", f.String())
		default:
			panic("not reached")
		}
	}
}

func (p *Properties) String() string {
	var buf bytes.Buffer
	v := reflect.ValueOf(*p)
	writeProperties(p.Loaded, v, &buf)

	// Write the UserProperties.
	keys := make([]string, 0, len(p.UserProperties))
	for key := range p.UserProperties {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	for _, key := range keys {
		fmt.Fprintf(&buf, "%s: %s\n", key, p.UserProperties[key])
	}
	return buf.String()
}

func (p *Properties) load(
	b block, blockOffset uint64, deniedUserProperties map[string]struct{},
) error {
	i, err := newRawBlockIter(bytes.Compare, b)
	if err != nil {
		return err
	}
	p.Loaded = make(map[uintptr]struct{})
	v := reflect.ValueOf(p).Elem()

	for valid := i.First(); valid; valid = i.Next() {
		if f, ok := propTagMap[string(i.Key().UserKey)]; ok {
			p.Loaded[f.Offset] = struct{}{}
			field := v.FieldByIndex(f.Index)
			switch f.Type.Kind() {
			case reflect.Bool:
				field.SetBool(bytes.Equal(i.Value(), propBoolTrue))
			case reflect.Uint32:
				field.SetUint(uint64(binary.LittleEndian.Uint32(i.Value())))
			case reflect.Uint64:
				var n uint64
				if string(i.Key().UserKey) == propGlobalSeqnumName {
					n = binary.LittleEndian.Uint64(i.Value())
				} else {
					n, _ = binary.Uvarint(i.Value())
				}
				field.SetUint(n)
			case reflect.String:
				field.SetString(intern.Bytes(i.Value()))
			default:
				panic("not reached")
			}
			continue
		}
		if p.UserProperties == nil {
			p.UserProperties = make(map[string]string)
		}

		if _, denied := deniedUserProperties[string(i.Key().UserKey)]; !denied {
			p.UserProperties[intern.Bytes(i.Key().UserKey)] = string(i.Value())
		}
	}
	return nil
}

func (p *Properties) saveBool(m map[string][]byte, offset uintptr, value bool) {
	tag := propOffsetTagMap[offset]
	if value {
		m[tag] = propBoolTrue
	} else {
		m[tag] = propBoolFalse
	}
}

func (p *Properties) saveUint32(m map[string][]byte, offset uintptr, value uint32) {
	var buf [4]byte
	binary.LittleEndian.PutUint32(buf[:], value)
	m[propOffsetTagMap[offset]] = buf[:]
}

func (p *Properties) saveUint64(m map[string][]byte, offset uintptr, value uint64) {
	var buf [8]byte
	binary.LittleEndian.PutUint64(buf[:], value)
	m[propOffsetTagMap[offset]] = buf[:]
}

func (p *Properties) saveUvarint(m map[string][]byte, offset uintptr, value uint64) {
	var buf [10]byte
	n := binary.PutUvarint(buf[:], value)
	m[propOffsetTagMap[offset]] = buf[:n]
}

func (p *Properties) saveString(m map[string][]byte, offset uintptr, value string) {
	m[propOffsetTagMap[offset]] = []byte(value)
}

func (p *Properties) save(tblFormat TableFormat, w *rawBlockWriter) {
	m := make(map[string][]byte)
	for k, v := range p.UserProperties {
		m[k] = []byte(v)
	}

	if p.ComparerName != "" {
		p.saveString(m, unsafe.Offsetof(p.ComparerName), p.ComparerName)
	}
	if p.CompressionName != "" {
		p.saveString(m, unsafe.Offsetof(p.CompressionName), p.CompressionName)
	}
	if p.CompressionOptions != "" {
		p.saveString(m, unsafe.Offsetof(p.CompressionOptions), p.CompressionOptions)
	}
	p.saveUvarint(m, unsafe.Offsetof(p.DataSize), p.DataSize)
	if p.ExternalFormatVersion != 0 {
		p.saveUint32(m, unsafe.Offsetof(p.ExternalFormatVersion), p.ExternalFormatVersion)
		p.saveUint64(m, unsafe.Offsetof(p.GlobalSeqNum), p.GlobalSeqNum)
	}
	if p.FilterPolicyName != "" {
		p.saveString(m, unsafe.Offsetof(p.FilterPolicyName), p.FilterPolicyName)
	}
	p.saveUvarint(m, unsafe.Offsetof(p.FilterSize), p.FilterSize)
	if p.IndexPartitions != 0 {
		p.saveUvarint(m, unsafe.Offsetof(p.IndexPartitions), p.IndexPartitions)
		p.saveUvarint(m, unsafe.Offsetof(p.TopLevelIndexSize), p.TopLevelIndexSize)
	}
	p.saveUvarint(m, unsafe.Offsetof(p.IndexSize), p.IndexSize)
	p.saveUint32(m, unsafe.Offsetof(p.IndexType), p.IndexType)
	if p.IsStrictObsolete {
		p.saveBool(m, unsafe.Offsetof(p.IsStrictObsolete), p.IsStrictObsolete)
	}
	if p.MergerName != "" {
		p.saveString(m, unsafe.Offsetof(p.MergerName), p.MergerName)
	}
	p.saveUvarint(m, unsafe.Offsetof(p.NumDataBlocks), p.NumDataBlocks)
	p.saveUvarint(m, unsafe.Offsetof(p.NumEntries), p.NumEntries)
	p.saveUvarint(m, unsafe.Offsetof(p.NumDeletions), p.NumDeletions)
	if p.NumSizedDeletions > 0 {
		p.saveUvarint(m, unsafe.Offsetof(p.NumSizedDeletions), p.NumSizedDeletions)
	}
	p.saveUvarint(m, unsafe.Offsetof(p.NumMergeOperands), p.NumMergeOperands)
	p.saveUvarint(m, unsafe.Offsetof(p.NumRangeDeletions), p.NumRangeDeletions)
	// NB: We only write out some properties for Pebble formats. This isn't
	// strictly necessary because unrecognized properties are interpreted as
	// user-defined properties, however writing them prevents byte-for-byte
	// equivalence with RocksDB files that some of our testing requires.
	if p.RawPointTombstoneKeySize > 0 && tblFormat >= TableFormatPebblev1 {
		p.saveUvarint(m, unsafe.Offsetof(p.RawPointTombstoneKeySize), p.RawPointTombstoneKeySize)
	}
	if p.RawPointTombstoneValueSize > 0 {
		p.saveUvarint(m, unsafe.Offsetof(p.RawPointTombstoneValueSize), p.RawPointTombstoneValueSize)
	}
	if p.NumRangeKeys() > 0 {
		p.saveUvarint(m, unsafe.Offsetof(p.NumRangeKeyDels), p.NumRangeKeyDels)
		p.saveUvarint(m, unsafe.Offsetof(p.NumRangeKeySets), p.NumRangeKeySets)
		p.saveUvarint(m, unsafe.Offsetof(p.NumRangeKeyUnsets), p.NumRangeKeyUnsets)
		p.saveUvarint(m, unsafe.Offsetof(p.RawRangeKeyKeySize), p.RawRangeKeyKeySize)
		p.saveUvarint(m, unsafe.Offsetof(p.RawRangeKeyValueSize), p.RawRangeKeyValueSize)
	}
	if p.NumValueBlocks > 0 {
		p.saveUvarint(m, unsafe.Offsetof(p.NumValueBlocks), p.NumValueBlocks)
	}
	if p.NumValuesInValueBlocks > 0 {
		p.saveUvarint(m, unsafe.Offsetof(p.NumValuesInValueBlocks), p.NumValuesInValueBlocks)
	}
	if p.PrefixExtractorName != "" {
		p.saveString(m, unsafe.Offsetof(p.PrefixExtractorName), p.PrefixExtractorName)
	}
	p.saveBool(m, unsafe.Offsetof(p.PrefixFiltering), p.PrefixFiltering)
	if p.PropertyCollectorNames != "" {
		p.saveString(m, unsafe.Offsetof(p.PropertyCollectorNames), p.PropertyCollectorNames)
	}
	if p.SnapshotPinnedKeys > 0 {
		p.saveUvarint(m, unsafe.Offsetof(p.SnapshotPinnedKeys), p.SnapshotPinnedKeys)
		p.saveUvarint(m, unsafe.Offsetof(p.SnapshotPinnedKeySize), p.SnapshotPinnedKeySize)
		p.saveUvarint(m, unsafe.Offsetof(p.SnapshotPinnedValueSize), p.SnapshotPinnedValueSize)
	}
	p.saveUvarint(m, unsafe.Offsetof(p.RawKeySize), p.RawKeySize)
	p.saveUvarint(m, unsafe.Offsetof(p.RawValueSize), p.RawValueSize)
	if p.ValueBlocksSize > 0 {
		p.saveUvarint(m, unsafe.Offsetof(p.ValueBlocksSize), p.ValueBlocksSize)
	}
	p.saveBool(m, unsafe.Offsetof(p.WholeKeyFiltering), p.WholeKeyFiltering)

	if tblFormat < TableFormatPebblev1 {
		m["rocksdb.column.family.id"] = binary.AppendUvarint([]byte(nil), math.MaxInt32)
		m["rocksdb.fixed.key.length"] = []byte{0x00}
		m["rocksdb.index.key.is.user.key"] = []byte{0x00}
		m["rocksdb.index.value.is.delta.encoded"] = []byte{0x00}
		m["rocksdb.oldest.key.time"] = []byte{0x00}
		m["rocksdb.creation.time"] = []byte{0x00}
		m["rocksdb.format.version"] = []byte{0x00}
	}

	keys := make([]string, 0, len(m))
	for key := range m {
		keys = append(keys, key)
	}
	sort.Strings(keys)
	for _, key := range keys {
		w.add(InternalKey{UserKey: []byte(key)}, m[key])
	}
}
