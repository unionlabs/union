package query

import (
	"fmt"
	"math"

	db "github.com/cosmos/cosmos-db"
	"google.golang.org/grpc/codes"
	"google.golang.org/grpc/status"

	"cosmossdk.io/store/types"
)

// DefaultPage is the default `page` number for queries.
// If the `page` number is not supplied, `DefaultPage` will be used.
const DefaultPage = 1

// DefaultLimit is the default `limit` for queries
// if the `limit` is not supplied, paginate will use `DefaultLimit`
const DefaultLimit = 100

// PaginationMaxLimit is the maximum limit the paginate function can handle
// which equals the maximum value that can be stored in uint64
var PaginationMaxLimit uint64 = math.MaxUint64

// ParsePagination validate PageRequest and returns page number & limit.
func ParsePagination(pageReq *PageRequest) (page, limit int, err error) {
	offset := 0
	limit = DefaultLimit

	if pageReq != nil {
		offset = int(pageReq.Offset)
		limit = int(pageReq.Limit)
	}
	if offset < 0 {
		return 1, 0, status.Error(codes.InvalidArgument, "offset must greater than 0")
	}

	if limit < 0 {
		return 1, 0, status.Error(codes.InvalidArgument, "limit must greater than 0")
	} else if limit == 0 {
		limit = DefaultLimit
	}

	page = offset/limit + 1

	return page, limit, nil
}

// Paginate does pagination of all the results in the PrefixStore based on the
// provided PageRequest. onResult should be used to do actual unmarshaling.
func Paginate(
	prefixStore types.KVStore,
	pageRequest *PageRequest,
	onResult func(key, value []byte) error,
) (*PageResponse, error) {
	pageRequest = initPageRequestDefaults(pageRequest)

	if pageRequest.Offset > 0 && pageRequest.Key != nil {
		return nil, fmt.Errorf("invalid request, either offset or key is expected, got both")
	}

	iterator := getIterator(prefixStore, pageRequest.Key, pageRequest.Reverse)
	defer iterator.Close()

	var count uint64
	var nextKey []byte

	if len(pageRequest.Key) != 0 {
		for ; iterator.Valid(); iterator.Next() {
			if count == pageRequest.Limit {
				nextKey = iterator.Key()
				break
			}
			if iterator.Error() != nil {
				return nil, iterator.Error()
			}
			err := onResult(iterator.Key(), iterator.Value())
			if err != nil {
				return nil, err
			}

			count++
		}

		return &PageResponse{
			NextKey: nextKey,
		}, nil
	}

	end := pageRequest.Offset + pageRequest.Limit

	for ; iterator.Valid(); iterator.Next() {
		count++

		if count <= pageRequest.Offset {
			continue
		}
		if count <= end {
			err := onResult(iterator.Key(), iterator.Value())
			if err != nil {
				return nil, err
			}
		} else if count == end+1 {
			nextKey = iterator.Key()

			if !pageRequest.CountTotal {
				break
			}
		}
		if iterator.Error() != nil {
			return nil, iterator.Error()
		}
	}

	res := &PageResponse{NextKey: nextKey}
	if pageRequest.CountTotal {
		res.Total = count
	}

	return res, nil
}

func getIterator(prefixStore types.KVStore, start []byte, reverse bool) db.Iterator {
	if reverse {
		var end []byte
		if start != nil {
			itr := prefixStore.Iterator(start, nil)
			defer itr.Close()
			if itr.Valid() {
				itr.Next()
				end = itr.Key()
			}
		}
		return prefixStore.ReverseIterator(nil, end)
	}
	return prefixStore.Iterator(start, nil)
}

// initPageRequestDefaults initializes a PageRequest's defaults when those are not set.
func initPageRequestDefaults(pageRequest *PageRequest) *PageRequest {
	// if the PageRequest is nil, use default PageRequest
	if pageRequest == nil {
		pageRequest = &PageRequest{}
	}

	pageRequestCopy := *pageRequest
	if len(pageRequestCopy.Key) == 0 {
		pageRequestCopy.Key = nil
	}

	if pageRequestCopy.Limit == 0 {
		pageRequestCopy.Limit = DefaultLimit

		// count total results when the limit is zero/not supplied
		pageRequestCopy.CountTotal = true
	}

	return &pageRequestCopy
}
