package client_test

import (
	"testing"

	"github.com/stretchr/testify/require"

	"github.com/cosmos/cosmos-sdk/client"
)

func TestPaginate(t *testing.T) {
	testCases := []struct {
		name                           string
		numObjs, page, limit, defLimit int
		expectedStart, expectedEnd     int
	}{
		{
			"all objects in a single page",
			100, 1, 100, 100,
			0, 100,
		},
		{
			"page one of three",
			75, 1, 25, 100,
			0, 25,
		},
		{
			"page two of three",
			75, 2, 25, 100,
			25, 50,
		},
		{
			"page three of three",
			75, 3, 25, 100,
			50, 75,
		},
		{
			"end is greater than total number of objects",
			75, 2, 50, 100,
			50, 75,
		},
		{
			"fallback to default limit",
			75, 5, 0, 10,
			40, 50,
		},
		{
			"invalid start page",
			75, 4, 25, 100,
			-1, -1,
		},
		{
			"invalid zero start page",
			75, 0, 25, 100,
			-1, -1,
		},
		{
			"invalid negative start page",
			75, -1, 25, 100,
			-1, -1,
		},
		{
			"invalid default limit",
			75, 2, 0, -10,
			-1, -1,
		},
	}

	for i, tc := range testCases {
		i, tc := i, tc
		t.Run(tc.name, func(t *testing.T) {
			start, end := client.Paginate(tc.numObjs, tc.page, tc.limit, tc.defLimit)
			require.Equal(t, tc.expectedStart, start, "invalid result; test case #%d", i)
			require.Equal(t, tc.expectedEnd, end, "invalid result; test case #%d", i)
		})
	}
}
