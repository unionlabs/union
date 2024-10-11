package grocksdb

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestWriteOptions(t *testing.T) {
	t.Parallel()

	wo := NewDefaultWriteOptions()
	defer wo.Destroy()

	require.EqualValues(t, false, wo.IsSync())
	wo.SetSync(true)
	require.EqualValues(t, true, wo.IsSync())

	require.EqualValues(t, false, wo.IsDisableWAL())
	wo.DisableWAL(true)
	require.EqualValues(t, true, wo.IsDisableWAL())

	require.EqualValues(t, false, wo.IgnoreMissingColumnFamilies())
	wo.SetIgnoreMissingColumnFamilies(true)
	require.EqualValues(t, true, wo.IgnoreMissingColumnFamilies())

	require.EqualValues(t, false, wo.IsNoSlowdown())
	wo.SetNoSlowdown(true)
	require.EqualValues(t, true, wo.IsNoSlowdown())

	require.EqualValues(t, false, wo.IsLowPri())
	wo.SetLowPri(true)
	require.EqualValues(t, true, wo.IsLowPri())

	require.EqualValues(t, false, wo.MemtableInsertHintPerBatch())
	wo.SetMemtableInsertHintPerBatch(true)
	require.EqualValues(t, true, wo.MemtableInsertHintPerBatch())
}
