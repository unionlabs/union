package indexer

import (
	"math/big"
	"time"

	"github.com/cometbft/cometbft/libs/pubsub/query/syntax"
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
	case *big.Int:
		tmp := new(big.Int)
		return tmp.Add(t, big.NewInt(1))

	case *big.Float:
		// For floats we cannot simply add one as the float to float
		// comparison is more finegrained.
		// When comparing to integers, adding one is also incorrect:
		// example: x >100.2 ; x = 101 float increased to 101.2 and condition
		// is not satisfied
		return t
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
	case *big.Int:
		tmp := new(big.Int)
		return tmp.Sub(t, big.NewInt(1))
	case *big.Float:
		return t
	case time.Time:
		return t.Unix() - 1

	default:
		panic("not implemented")
	}
}

// LookForRangesWithHeight returns a mapping of QueryRanges and the matching indexes in
// the provided query conditions.
func LookForRangesWithHeight(conditions []syntax.Condition) (queryRange QueryRanges, indexes []int, heightRange QueryRange) {
	queryRange = make(QueryRanges)
	for i, c := range conditions {
		if IsRangeOperation(c.Op) {
			heightKey := c.Tag == types.BlockHeightKey || c.Tag == types.TxHeightKey
			r, ok := queryRange[c.Tag]
			if !ok {
				r = QueryRange{Key: c.Tag}
				if c.Tag == types.BlockHeightKey || c.Tag == types.TxHeightKey {
					heightRange = QueryRange{Key: c.Tag}
				}
			}

			switch c.Op {
			case syntax.TGt:
				if heightKey {
					heightRange.LowerBound = conditionArg(c)
				}
				r.LowerBound = conditionArg(c)

			case syntax.TGeq:
				r.IncludeLowerBound = true
				r.LowerBound = conditionArg(c)
				if heightKey {
					heightRange.IncludeLowerBound = true
					heightRange.LowerBound = conditionArg(c)
				}

			case syntax.TLt:
				r.UpperBound = conditionArg(c)
				if heightKey {
					heightRange.UpperBound = conditionArg(c)
				}

			case syntax.TLeq:
				r.IncludeUpperBound = true
				r.UpperBound = conditionArg(c)
				if heightKey {
					heightRange.IncludeUpperBound = true
					heightRange.UpperBound = conditionArg(c)
				}
			}

			queryRange[c.Tag] = r
			indexes = append(indexes, i)
		}
	}

	return queryRange, indexes, heightRange
}

// Deprecated: This function is not used anymore and will be replaced with LookForRangesWithHeight
func LookForRanges(conditions []syntax.Condition) (ranges QueryRanges, indexes []int) {
	ranges = make(QueryRanges)
	for i, c := range conditions {
		if IsRangeOperation(c.Op) {
			r, ok := ranges[c.Tag]
			if !ok {
				r = QueryRange{Key: c.Tag}
			}

			switch c.Op {
			case syntax.TGt:
				r.LowerBound = conditionArg(c)

			case syntax.TGeq:
				r.IncludeLowerBound = true
				r.LowerBound = conditionArg(c)

			case syntax.TLt:
				r.UpperBound = conditionArg(c)

			case syntax.TLeq:
				r.IncludeUpperBound = true
				r.UpperBound = conditionArg(c)
			}

			ranges[c.Tag] = r
			indexes = append(indexes, i)
		}
	}

	return ranges, indexes
}

// IsRangeOperation returns a boolean signifying if a query Operator is a range
// operation or not.
func IsRangeOperation(op syntax.Token) bool {
	switch op {
	case syntax.TGt, syntax.TGeq, syntax.TLt, syntax.TLeq:
		return true

	default:
		return false
	}
}

func conditionArg(c syntax.Condition) interface{} {
	if c.Arg == nil {
		return nil
	}
	switch c.Arg.Type {
	case syntax.TNumber:
		return c.Arg.Number()
	case syntax.TTime, syntax.TDate:
		return c.Arg.Time()
	default:
		return c.Arg.Value() // string
	}
}
