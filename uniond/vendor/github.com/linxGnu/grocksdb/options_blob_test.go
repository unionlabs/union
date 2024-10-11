package grocksdb

import (
	"testing"

	"github.com/stretchr/testify/require"
)

func TestOptionBlobFile(t *testing.T) {
	t.Parallel()

	opt := NewDefaultOptions()
	defer opt.Destroy()

	opt.EnableBlobFiles(true)
	require.True(t, opt.IsBlobFilesEnabled())

	opt.SetMinBlobSize(1024)
	require.EqualValues(t, 1024, opt.GetMinBlobSize())

	require.EqualValues(t, 256<<20, opt.GetBlobFileSize())
	opt.SetBlobFileSize(128 << 20)
	require.EqualValues(t, 128<<20, opt.GetBlobFileSize())

	require.Equal(t, NoCompression, opt.GetBlobCompressionType())
	opt.SetBlobCompressionType(SnappyCompression)
	require.Equal(t, SnappyCompression, opt.GetBlobCompressionType())

	require.False(t, opt.IsBlobGCEnabled())
	opt.EnableBlobGC(true)
	require.True(t, opt.IsBlobGCEnabled())

	require.EqualValues(t, 0.25, opt.GetBlobGCAgeCutoff())
	opt.SetBlobGCAgeCutoff(0.3)
	require.EqualValues(t, 0.3, opt.GetBlobGCAgeCutoff())

	require.EqualValues(t, 1.0, opt.GetBlobGCForceThreshold())
	opt.SetBlobGCForceThreshold(1.3)
	require.EqualValues(t, 1.3, opt.GetBlobGCForceThreshold())
}
