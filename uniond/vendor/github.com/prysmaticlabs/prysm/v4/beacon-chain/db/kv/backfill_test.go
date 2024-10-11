package kv

import (
	"context"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/proto/dbval"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"google.golang.org/protobuf/proto"
)

func TestBackfillRoundtrip(t *testing.T) {
	db := setupDB(t)
	b := &dbval.BackfillStatus{}
	b.LowSlot = 23
	b.LowRoot = bytesutil.PadTo([]byte("low"), 32)
	b.LowParentRoot = bytesutil.PadTo([]byte("parent"), 32)
	m, err := proto.Marshal(b)
	require.NoError(t, err)
	ub := &dbval.BackfillStatus{}
	require.NoError(t, proto.Unmarshal(m, ub))
	require.Equal(t, b.LowSlot, ub.LowSlot)
	require.DeepEqual(t, b.LowRoot, ub.LowRoot)
	require.DeepEqual(t, b.LowParentRoot, ub.LowParentRoot)

	ctx := context.Background()
	require.NoError(t, db.SaveBackfillStatus(ctx, b))
	dbub, err := db.BackfillStatus(ctx)
	require.NoError(t, err)

	require.Equal(t, b.LowSlot, dbub.LowSlot)
	require.DeepEqual(t, b.LowRoot, dbub.LowRoot)
	require.DeepEqual(t, b.LowParentRoot, dbub.LowParentRoot)
}
