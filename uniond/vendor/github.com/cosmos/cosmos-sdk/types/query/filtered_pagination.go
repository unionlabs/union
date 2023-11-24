package query

import (
	"fmt"

	proto "github.com/cosmos/gogoproto/proto"

	"cosmossdk.io/store/types"

	"github.com/cosmos/cosmos-sdk/codec"
)

// FilteredPaginate does pagination of all the results in the PrefixStore based on the
// provided PageRequest. onResult should be used to do actual unmarshaling and filter the results.
// If key is provided, the pagination uses the optimized querying.
// If offset is used, the pagination uses lazy filtering i.e., searches through all the records.
// The accumulate parameter represents if the response is valid based on the offset given.
// It will be false for the results (filtered) < offset  and true for `offset > accumulate <= end`.
// When accumulate is set to true the current result should be appended to the result set returned
// to the client.
func FilteredPaginate(
	prefixStore types.KVStore,
	pageRequest *PageRequest,
	onResult func(key, value []byte, accumulate bool) (bool, error),
) (*PageResponse, error) {
	pageRequest = initPageRequestDefaults(pageRequest)

	if pageRequest.Offset > 0 && pageRequest.Key != nil {
		return nil, fmt.Errorf("invalid request, either offset or key is expected, got both")
	}

	var (
		numHits uint64
		nextKey []byte
		err     error
	)

	iterator := getIterator(prefixStore, pageRequest.Key, pageRequest.Reverse)
	defer iterator.Close()

	if len(pageRequest.Key) != 0 {
		accumulateFn := func(_ uint64) bool { return true }
		for ; iterator.Valid(); iterator.Next() {
			if numHits == pageRequest.Limit {
				nextKey = iterator.Key()
				break
			}

			numHits, err = processResult(iterator, numHits, onResult, accumulateFn)
			if err != nil {
				return nil, err
			}
		}

		return &PageResponse{
			NextKey: nextKey,
		}, nil
	}

	end := pageRequest.Offset + pageRequest.Limit
	accumulateFn := func(numHits uint64) bool { return numHits >= pageRequest.Offset && numHits < end }

	for ; iterator.Valid(); iterator.Next() {
		numHits, err = processResult(iterator, numHits, onResult, accumulateFn)
		if err != nil {
			return nil, err
		}
		if numHits == end+1 {
			if nextKey == nil {
				nextKey = iterator.Key()
			}

			if !pageRequest.CountTotal {
				break
			}
		}
	}

	res := &PageResponse{NextKey: nextKey}
	if pageRequest.CountTotal {
		res.Total = numHits
	}

	return res, nil
}

func processResult(iterator types.Iterator, numHits uint64, onResult func(key, value []byte, accumulate bool) (bool, error), accumulateFn func(numHits uint64) bool) (uint64, error) {
	if iterator.Error() != nil {
		return numHits, iterator.Error()
	}

	accumulate := accumulateFn(numHits)
	hit, err := onResult(iterator.Key(), iterator.Value(), accumulate)
	if err != nil {
		return numHits, err
	}

	if hit {
		numHits++
	}

	return numHits, nil
}

func genericProcessResult[T, F proto.Message](iterator types.Iterator, numHits uint64, onResult func(key []byte, value T) (F, error), accumulateFn func(numHits uint64) bool,
	constructor func() T, cdc codec.BinaryCodec, results []F,
) ([]F, uint64, error) {
	if iterator.Error() != nil {
		return results, numHits, iterator.Error()
	}

	protoMsg := constructor()

	err := cdc.Unmarshal(iterator.Value(), protoMsg)
	if err != nil {
		return results, numHits, err
	}

	val, err := onResult(iterator.Key(), protoMsg)
	if err != nil {
		return results, numHits, err
	}

	if proto.Size(val) != 0 {
		// Previously this was the "accumulate" flag
		if accumulateFn(numHits) {
			results = append(results, val)
		}
		numHits++
	}

	return results, numHits, nil
}

// GenericFilteredPaginate does pagination of all the results in the PrefixStore based on the
// provided PageRequest. `onResult` should be used to filter or transform the results.
// `c` is a constructor function that needs to return a new instance of the type T (this is to
// workaround some generic pitfalls in which we can't instantiate a T struct inside the function).
// If key is provided, the pagination uses the optimized querying.
// If offset is used, the pagination uses lazy filtering i.e., searches through all the records.
// The resulting slice (of type F) can be of a different type than the one being iterated through
// (type T), so it's possible to do any necessary transformation inside the onResult function.
func GenericFilteredPaginate[T, F proto.Message](
	cdc codec.BinaryCodec,
	prefixStore types.KVStore,
	pageRequest *PageRequest,
	onResult func(key []byte, value T) (F, error),
	constructor func() T,
) ([]F, *PageResponse, error) {
	pageRequest = initPageRequestDefaults(pageRequest)
	results := []F{}

	if pageRequest.Offset > 0 && pageRequest.Key != nil {
		return results, nil, fmt.Errorf("invalid request, either offset or key is expected, got both")
	}

	var (
		numHits uint64
		nextKey []byte
		err     error
	)

	iterator := getIterator(prefixStore, pageRequest.Key, pageRequest.Reverse)
	defer iterator.Close()

	if len(pageRequest.Key) != 0 {
		accumulateFn := func(_ uint64) bool { return true }
		for ; iterator.Valid(); iterator.Next() {
			if numHits == pageRequest.Limit {
				nextKey = iterator.Key()
				break
			}

			results, numHits, err = genericProcessResult(iterator, numHits, onResult, accumulateFn, constructor, cdc, results)
			if err != nil {
				return nil, nil, err
			}
		}

		return results, &PageResponse{
			NextKey: nextKey,
		}, nil
	}

	end := pageRequest.Offset + pageRequest.Limit
	accumulateFn := func(numHits uint64) bool { return numHits >= pageRequest.Offset && numHits < end }

	for ; iterator.Valid(); iterator.Next() {
		results, numHits, err = genericProcessResult(iterator, numHits, onResult, accumulateFn, constructor, cdc, results)
		if err != nil {
			return nil, nil, err
		}

		if numHits == end+1 {
			if nextKey == nil {
				nextKey = iterator.Key()
			}

			if !pageRequest.CountTotal {
				break
			}
		}
	}

	res := &PageResponse{NextKey: nextKey}
	if pageRequest.CountTotal {
		res.Total = numHits
	}

	return results, res, nil
}
