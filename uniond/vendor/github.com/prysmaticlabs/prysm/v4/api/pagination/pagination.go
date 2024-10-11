// Package pagination contains useful pagination-related helpers.
package pagination

import (
	"fmt"
	"strconv"

	"github.com/pkg/errors"
	"github.com/prysmaticlabs/prysm/v4/config/params"
)

// StartAndEndPage takes in the requested page token, wanted page size, total page size.
// It returns start, end page and the next page token.
func StartAndEndPage(pageToken string, pageSize, totalSize int) (int, int, string, error) {
	if pageToken == "" {
		pageToken = "0"
	}
	if pageSize < 0 || totalSize < 0 {
		return 0, 0, "", errors.Errorf("invalid page and total sizes provided: page size %d , total size %d", pageSize, totalSize)
	}
	if pageSize == 0 {
		pageSize = params.BeaconConfig().DefaultPageSize
	}

	token, err := strconv.Atoi(pageToken)
	if err != nil {
		return 0, 0, "", errors.Wrap(err, "could not convert page token")
	}
	if token < 0 {
		return 0, 0, "", errors.Errorf("invalid token value provided: %d", token)
	}

	// Start page can not be greater than set size.
	start := token * pageSize
	if start >= totalSize {
		return 0, 0, "", fmt.Errorf("page start %d >= list %d", start, totalSize)
	}

	// End page can not go out of bound.
	end := start + pageSize
	nextPageToken := strconv.Itoa(token + 1)

	if end >= totalSize {
		end = totalSize
		nextPageToken = "" // Return an empty next page token for the last page of a set
	}

	return start, end, nextPageToken, nil
}
