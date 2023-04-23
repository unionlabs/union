package indexer

import (
	"time"

	"github.com/cometbft/cometbft/libs/pubsub/query"
	"github.com/cometbft/cometbft/types"
)

// QueryRanges defines a mapping between a composite event key and a QueryRange.
//
// e.g.account.number => queryRange{lowerBound: 1, upperBound: 5}
type QueryRanges map[string]QueryRange

// QueryRange defines a range within a query condition.
type QueryRange struct {
	LowerBound        interface{} // int || time.Time
	UpperBound        interface{} // int || time.Time
	Key               string
	IncludeLowerBound bool
	IncludeUpperBound bool
}

// AnyBound returns either the lower bound if non-nil, otherwise the upper bound.
func (qr QueryRange) AnyBound() interface{} {
	if qr.LowerBound != nil {
		return qr.LowerBound
	}

	return qr.UpperBound
}

// LowerBoundValue returns the value for the lower bound. If the lower bound is
// nil, nil will be returned.
func (qr QueryRange) LowerBoundValue() interface{} {
	if qr.LowerBound == nil {
		return nil
	}

	if qr.IncludeLowerBound {
		return qr.LowerBound
	}

	switch t := qr.LowerBound.(type) {
	case int64:
		return t + 1

	case time.Time:
		return t.Unix() + 1

	default:
		panic("not implemented")
	}
}

// UpperBoundValue returns the value for the upper bound. If the upper bound is
// nil, nil will be returned.
func (qr QueryRange) UpperBoundValue() interface{} {
	if qr.UpperBound == nil {
		return nil
	}

	if qr.IncludeUpperBound {
		return qr.UpperBound
	}

	switch t := qr.UpperBound.(type) {
	case int64:
		return t - 1

	case time.Time:
		return t.Unix() - 1

	default:
		panic("not implemented")
	}
}

// LookForRangesWithHeight returns a mapping of QueryRanges and the matching indexes in
// the provided query conditions.
func LookForRangesWithHeight(conditions []query.Condition) (queryRange QueryRanges, indexes []int, heightRange QueryRange) {
	queryRange = make(QueryRanges)
	for i, c := range conditions {
		heightKey := false
		if IsRangeOperation(c.Op) {
			r, ok := queryRange[c.CompositeKey]
			if !ok {
				r = QueryRange{Key: c.CompositeKey}
				if c.CompositeKey == types.BlockHeightKey || c.CompositeKey == types.TxHeightKey {
					heightRange = QueryRange{Key: c.CompositeKey}
					heightKey = true
				}
			}

			switch c.Op {
			case query.OpGreater:
				if heightKey {
					heightRange.LowerBound = c.Operand
				}
				r.LowerBound = c.Operand

			case query.OpGreaterEqual:
				r.IncludeLowerBound = true
				r.LowerBound = c.Operand
				if heightKey {
					heightRange.IncludeLowerBound = true
					heightRange.LowerBound = c.Operand
				}

			case query.OpLess:
				r.UpperBound = c.Operand
				if heightKey {
					heightRange.UpperBound = c.Operand
				}

			case query.OpLessEqual:
				r.IncludeUpperBound = true
				r.UpperBound = c.Operand
				if heightKey {
					heightRange.IncludeUpperBound = true
					heightRange.UpperBound = c.Operand
				}
			}

			queryRange[c.CompositeKey] = r
			indexes = append(indexes, i)
		}
	}

	return queryRange, indexes, heightRange
}

// Deprecated: This function is not used anymore and will be replaced with LookForRangesWithHeight
func LookForRanges(conditions []query.Condition) (ranges QueryRanges, indexes []int) {
	ranges = make(QueryRanges)
	for i, c := range conditions {
		if IsRangeOperation(c.Op) {
			r, ok := ranges[c.CompositeKey]
			if !ok {
				r = QueryRange{Key: c.CompositeKey}
			}

			switch c.Op {
			case query.OpGreater:
				r.LowerBound = c.Operand

			case query.OpGreaterEqual:
				r.IncludeLowerBound = true
				r.LowerBound = c.Operand

			case query.OpLess:
				r.UpperBound = c.Operand

			case query.OpLessEqual:
				r.IncludeUpperBound = true
				r.UpperBound = c.Operand
			}

			ranges[c.CompositeKey] = r
			indexes = append(indexes, i)
		}
	}

	return ranges, indexes
}

// IsRangeOperation returns a boolean signifying if a query Operator is a range
// operation or not.
func IsRangeOperation(op query.Operator) bool {
	switch op {
	case query.OpGreater, query.OpGreaterEqual, query.OpLess, query.OpLessEqual:
		return true

	default:
		return false
	}
}
