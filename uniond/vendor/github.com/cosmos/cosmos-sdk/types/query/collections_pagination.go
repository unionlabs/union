package query

import (
	"context"
	"errors"
	"fmt"

	"cosmossdk.io/collections"
	collcodec "cosmossdk.io/collections/codec"
	storetypes "cosmossdk.io/store/types"
)

// WithCollectionPaginationPairPrefix applies a prefix to a collection, whose key is a collection.Pair,
// being paginated that needs prefixing.
func WithCollectionPaginationPairPrefix[K1, K2 any](prefix K1) func(o *CollectionsPaginateOptions[collections.Pair[K1, K2]]) {
	return func(o *CollectionsPaginateOptions[collections.Pair[K1, K2]]) {
		prefix := collections.PairPrefix[K1, K2](prefix)
		o.Prefix = &prefix
	}
}

// CollectionsPaginateOptions provides extra options for pagination in collections.
type CollectionsPaginateOptions[K any] struct {
	// Prefix allows to optionally set a prefix for the pagination.
	Prefix *K
}

// Collection defines the minimum required API of a collection
// to work with pagination.
type Collection[K, V any] interface {
	// IterateRaw allows to iterate over a raw set of byte keys.
	IterateRaw(ctx context.Context, start, end []byte, order collections.Order) (collections.Iterator[K, V], error)
	// KeyCodec exposes the KeyCodec of a collection, required to encode a collection key from and to bytes
	// for pagination request and response.
	KeyCodec() collcodec.KeyCodec[K]
}

// CollectionPaginate follows the same logic as Paginate but for collection types.
// transformFunc is used to transform the result to a different type.
func CollectionPaginate[K, V any, C Collection[K, V], T any](
	ctx context.Context,
	coll C,
	pageReq *PageRequest,
	transformFunc func(key K, value V) (T, error),
	opts ...func(opt *CollectionsPaginateOptions[K]),
) ([]T, *PageResponse, error) {
	return CollectionFilteredPaginate(
		ctx,
		coll,
		pageReq,
		nil,
		transformFunc,
		opts...,
	)
}

// CollectionFilteredPaginate works in the same way as CollectionPaginate but allows to filter
// results using a predicateFunc.
// A nil predicateFunc means no filtering is applied and results are collected as is.
// TransformFunc is applied only to results which are in range of the pagination and allow
// to convert the result to a different type.
// NOTE: do not collect results using the values/keys passed to predicateFunc as they are not
// guaranteed to be in the pagination range requested.
func CollectionFilteredPaginate[K, V any, C Collection[K, V], T any](
	ctx context.Context,
	coll C,
	pageReq *PageRequest,
	predicateFunc func(key K, value V) (include bool, err error),
	transformFunc func(key K, value V) (T, error),
	opts ...func(opt *CollectionsPaginateOptions[K]),
) (results []T, pageRes *PageResponse, err error) {
	pageReq = initPageRequestDefaults(pageReq)

	offset := pageReq.Offset
	key := pageReq.Key
	limit := pageReq.Limit
	countTotal := pageReq.CountTotal
	reverse := pageReq.Reverse

	if offset > 0 && key != nil {
		return nil, nil, fmt.Errorf("invalid request, either offset or key is expected, got both")
	}

	opt := new(CollectionsPaginateOptions[K])
	for _, o := range opts {
		o(opt)
	}

	var prefix []byte
	if opt.Prefix != nil {
		prefix, err = encodeCollKey[K, V](coll, *opt.Prefix)
		if err != nil {
			return nil, nil, err
		}
	}

	if len(key) != 0 {
		results, pageRes, err = collFilteredPaginateByKey(ctx, coll, prefix, key, reverse, limit, predicateFunc, transformFunc)
	} else {
		results, pageRes, err = collFilteredPaginateNoKey(ctx, coll, prefix, reverse, offset, limit, countTotal, predicateFunc, transformFunc)
	}
	// invalid iter error is ignored to retain Paginate behavior
	if errors.Is(err, collections.ErrInvalidIterator) {
		return results, new(PageResponse), nil
	}
	// strip the prefix from next key
	if len(pageRes.NextKey) != 0 && prefix != nil {
		pageRes.NextKey = pageRes.NextKey[len(prefix):]
	}
	return results, pageRes, err
}

// collFilteredPaginateNoKey applies the provided pagination on the collection when the starting key is not set.
// If predicateFunc is nil no filtering is applied.
func collFilteredPaginateNoKey[K, V any, C Collection[K, V], T any](
	ctx context.Context,
	coll C,
	prefix []byte,
	reverse bool,
	offset uint64,
	limit uint64,
	countTotal bool,
	predicateFunc func(K, V) (bool, error),
	transformFunc func(K, V) (T, error),
) ([]T, *PageResponse, error) {
	iterator, err := getCollIter[K, V](ctx, coll, prefix, nil, reverse)
	if err != nil {
		return nil, nil, err
	}
	defer iterator.Close()
	// we advance the iter equal to the provided offset
	if !advanceIter(iterator, offset) {
		return nil, nil, collections.ErrInvalidIterator
	}

	var (
		count   uint64
		nextKey []byte
		results []T
	)

	for ; iterator.Valid(); iterator.Next() {
		switch {
		// first case, we still haven't found all the results up to the limit
		case count < limit:
			kv, err := iterator.KeyValue()
			if err != nil {
				return nil, nil, err
			}
			// if no predicate function is specified then we just include the result
			if predicateFunc == nil {
				transformed, err := transformFunc(kv.Key, kv.Value)
				if err != nil {
					return nil, nil, err
				}
				results = append(results, transformed)
				count++

				// if predicate function is defined we check if the result matches the filtering criteria
			} else {
				include, err := predicateFunc(kv.Key, kv.Value)
				if err != nil {
					return nil, nil, err
				}
				if include {
					transformed, err := transformFunc(kv.Key, kv.Value)
					if err != nil {
						return nil, nil, err
					}
					results = append(results, transformed)
					count++
				}
			}
		// second case, we found all the objects specified within the limit
		case count == limit:
			key, err := iterator.Key()
			if err != nil {
				return nil, nil, err
			}
			nextKey, err = encodeCollKey[K, V](coll, key)
			if err != nil {
				return nil, nil, err
			}
			// if count total was not specified, we return the next key only
			if !countTotal {
				return results, &PageResponse{
					NextKey: nextKey,
				}, nil
			}
			// otherwise we fallthrough the third case
			fallthrough
		// this is the case in which we found all the required results
		// but we need to count how many possible results exist in total.
		// so we keep increasing the count until the iterator is fully consumed.
		case count > limit:
			if predicateFunc == nil {
				count++

				// if predicate function is defined we check if the result matches the filtering criteria
			} else {
				kv, err := iterator.KeyValue()
				if err != nil {
					return nil, nil, err
				}

				include, err := predicateFunc(kv.Key, kv.Value)
				if err != nil {
					return nil, nil, err
				}
				if include {
					count++
				}
			}
		}
	}

	resp := &PageResponse{
		NextKey: nextKey,
	}

	if countTotal {
		resp.Total = count + offset
	}
	return results, resp, nil
}

func advanceIter[I interface {
	Next()
	Valid() bool
}](iter I, offset uint64,
) bool {
	for i := uint64(0); i < offset; i++ {
		if !iter.Valid() {
			return false
		}
		iter.Next()
	}
	return true
}

// collFilteredPaginateByKey paginates a collection when a starting key
// is provided in the PageRequest. Predicate is applied only if not nil.
func collFilteredPaginateByKey[K, V any, C Collection[K, V], T any](
	ctx context.Context,
	coll C,
	prefix []byte,
	key []byte,
	reverse bool,
	limit uint64,
	predicateFunc func(key K, value V) (bool, error),
	transformFunc func(key K, value V) (transformed T, err error),
) (results []T, pageRes *PageResponse, err error) {
	iterator, err := getCollIter[K, V](ctx, coll, prefix, key, reverse)
	if err != nil {
		return nil, nil, err
	}
	defer iterator.Close()

	var (
		count   uint64
		nextKey []byte
	)

	for ; iterator.Valid(); iterator.Next() {
		// if we reached the specified limit
		// then we get the next key, and we exit the iteration.
		if count == limit {
			concreteKey, err := iterator.Key()
			if err != nil {
				return nil, nil, err
			}

			nextKey, err = encodeCollKey[K, V](coll, concreteKey)
			if err != nil {
				return nil, nil, err
			}
			break
		}

		kv, err := iterator.KeyValue()
		if err != nil {
			return nil, nil, err
		}
		// if no predicate is specified then we just append the result
		if predicateFunc == nil {
			transformed, err := transformFunc(kv.Key, kv.Value)
			if err != nil {
				return nil, nil, err
			}
			results = append(results, transformed)
			// if predicate is applied we execute the predicate function
			// and append only if predicateFunc yields true.
		} else {
			include, err := predicateFunc(kv.Key, kv.Value)
			if err != nil {
				return nil, nil, err
			}
			if include {
				transformed, err := transformFunc(kv.Key, kv.Value)
				if err != nil {
					return nil, nil, err
				}
				results = append(results, transformed)
			}
		}
		count++
	}

	return results, &PageResponse{
		NextKey: nextKey,
	}, nil
}

// todo maybe move to collections?
func encodeCollKey[K, V any, C Collection[K, V]](coll C, key K) ([]byte, error) {
	buffer := make([]byte, coll.KeyCodec().Size(key))
	_, err := coll.KeyCodec().Encode(buffer, key)
	return buffer, err
}

func getCollIter[K, V any, C Collection[K, V]](ctx context.Context, coll C, prefix, start []byte, reverse bool) (collections.Iterator[K, V], error) {
	// TODO: maybe can be simplified
	if reverse {
		var end []byte
		if prefix != nil {
			start = storetypes.PrefixEndBytes(append(prefix, start...))
			end = prefix
		}
		return coll.IterateRaw(ctx, end, start, collections.OrderDescending)
	}
	var end []byte
	if prefix != nil {
		start = append(prefix, start...)
		end = storetypes.PrefixEndBytes(prefix)
	}
	return coll.IterateRaw(ctx, start, end, collections.OrderAscending)
}
