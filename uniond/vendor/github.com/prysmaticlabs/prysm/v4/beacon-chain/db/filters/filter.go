// Package filters specifies utilities for building a set of data attribute
// filters to be used when filtering data through database queries in practice.
// For example, one can specify a filter query for data by start epoch + end epoch + shard
// for attestations, build a filter as follows, and respond to it accordingly:
//
//	f := filters.NewFilter().SetStartEpoch(3).SetEndEpoch(5)
//	for k, v := range f.Filters() {
//	    switch k {
//	    case filters.StartEpoch:
//	       // Verify data matches filter criteria...
//	    case filters.EndEpoch:
//	       // Verify data matches filter criteria...
//	    }
//	}
package filters

import primitives "github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"

// FilterType defines an enum which is used as the keys in a map that tracks
// set attribute filters for data as part of the `FilterQuery` struct type.
type FilterType uint8

const (
	// ParentRoot defines a filter for parent roots of blocks using Simple Serialize (SSZ).
	ParentRoot FilterType = iota
	// StartSlot is used for range filters of objects by their slot (inclusive).
	StartSlot
	// EndSlot is used for range filters of objects by their slot (inclusive).
	EndSlot
	// StartEpoch is used for range filters of objects by their epoch (inclusive).
	StartEpoch
	// EndEpoch is used for range filters of objects by their epoch (inclusive).
	EndEpoch
	// HeadBlockRoot defines a filter for the head block root attribute of objects.
	HeadBlockRoot
	// SourceEpoch defines a filter for the source epoch attribute of objects.
	SourceEpoch
	// SourceRoot defines a filter for the source root attribute of objects.
	SourceRoot
	// TargetEpoch defines a filter for the target epoch attribute of objects.
	TargetEpoch
	// TargetRoot defines a filter for the target root attribute of objects.
	TargetRoot
	// SlotStep is used for range filters of objects by their slot in step increments.
	SlotStep
)

// QueryFilter defines a generic interface for type-asserting
// specific filters to use in querying DB objects.
type QueryFilter struct {
	queries map[FilterType]interface{}
}

// NewFilter instantiates a new QueryFilter type used to build filters for
// certain Ethereum data types by attribute.
func NewFilter() *QueryFilter {
	return &QueryFilter{
		queries: make(map[FilterType]interface{}),
	}
}

// Filters returns and underlying map of FilterType to interface{}, giving us
// a copy of the currently set filters which can then be iterated over and type
// asserted for use anywhere.
func (q *QueryFilter) Filters() map[FilterType]interface{} {
	return q.queries
}

// SetParentRoot allows for filtering by the parent root data attribute of an object.
func (q *QueryFilter) SetParentRoot(val []byte) *QueryFilter {
	q.queries[ParentRoot] = val
	return q
}

// SetHeadBlockRoot allows for filtering by the beacon block root data attribute of an object.
func (q *QueryFilter) SetHeadBlockRoot(val []byte) *QueryFilter {
	q.queries[HeadBlockRoot] = val
	return q
}

// SetSourceRoot allows for filtering by the source root data attribute of an object.
func (q *QueryFilter) SetSourceRoot(val []byte) *QueryFilter {
	q.queries[SourceRoot] = val
	return q
}

// SetTargetRoot allows for filtering by the target root data attribute of an object.
func (q *QueryFilter) SetTargetRoot(val []byte) *QueryFilter {
	q.queries[TargetRoot] = val
	return q
}

// SetSourceEpoch enables filtering by the source epoch data attribute of an object.
func (q *QueryFilter) SetSourceEpoch(val primitives.Epoch) *QueryFilter {
	q.queries[SourceEpoch] = val
	return q
}

// SetTargetEpoch enables filtering by the target epoch data attribute of an object.
func (q *QueryFilter) SetTargetEpoch(val primitives.Epoch) *QueryFilter {
	q.queries[TargetEpoch] = val
	return q
}

// SetStartSlot enables filtering by all the items that begin at a slot (inclusive).
func (q *QueryFilter) SetStartSlot(val primitives.Slot) *QueryFilter {
	q.queries[StartSlot] = val
	return q
}

// SetEndSlot enables filtering by all the items that end at a slot (inclusive).
func (q *QueryFilter) SetEndSlot(val primitives.Slot) *QueryFilter {
	q.queries[EndSlot] = val
	return q
}

// SetStartEpoch enables filtering by the StartEpoch attribute of an object (inclusive).
func (q *QueryFilter) SetStartEpoch(val primitives.Epoch) *QueryFilter {
	q.queries[StartEpoch] = val
	return q
}

// SetEndEpoch enables filtering by the EndEpoch attribute of an object (inclusive).
func (q *QueryFilter) SetEndEpoch(val primitives.Epoch) *QueryFilter {
	q.queries[EndEpoch] = val
	return q
}

// SetSlotStep enables filtering by slot for every step interval. For example, a slot range query
// for blocks from 0 to 9 with a step of 2 would return objects at slot 0, 2, 4, 6, 8.
func (q *QueryFilter) SetSlotStep(val uint64) *QueryFilter {
	q.queries[SlotStep] = val
	return q
}
