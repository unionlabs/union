package filesystem

import (
	"bytes"
	"fmt"
	"math"
	"os"
	"path"
	"sort"
	"testing"

	"github.com/prysmaticlabs/prysm/v4/beacon-chain/verification"
	fieldparams "github.com/prysmaticlabs/prysm/v4/config/fieldparams"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/testing/require"
	"github.com/prysmaticlabs/prysm/v4/testing/util"
	"github.com/spf13/afero"
)

func TestTryPruneDir_CachedNotExpired(t *testing.T) {
	fs := afero.NewMemMapFs()
	pr, err := newBlobPruner(fs, 0)
	require.NoError(t, err)
	slot := pr.windowSize
	_, sidecars := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, slot, fieldparams.MaxBlobsPerBlock)
	sc, err := verification.BlobSidecarNoop(sidecars[0])
	require.NoError(t, err)
	root := fmt.Sprintf("%#x", sc.BlockRoot())
	// This slot is right on the edge of what would need to be pruned, so by adding it to the cache and
	// skipping any other test setup, we can be certain the hot cache path never touches the filesystem.
	pr.slotMap.ensure(root, sc.Slot())
	pruned, err := pr.tryPruneDir(root, pr.windowSize)
	require.NoError(t, err)
	require.Equal(t, 0, pruned)
}

func TestTryPruneDir_CachedExpired(t *testing.T) {
	t.Run("empty directory", func(t *testing.T) {
		fs := afero.NewMemMapFs()
		pr, err := newBlobPruner(fs, 0)
		require.NoError(t, err)
		var slot primitives.Slot = 0
		_, sidecars := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, slot, 1)
		sc, err := verification.BlobSidecarNoop(sidecars[0])
		require.NoError(t, err)
		root := fmt.Sprintf("%#x", sc.BlockRoot())
		require.NoError(t, fs.Mkdir(root, directoryPermissions)) // make empty directory
		pr.slotMap.ensure(root, sc.Slot())
		pruned, err := pr.tryPruneDir(root, slot+1)
		require.NoError(t, err)
		require.Equal(t, 0, pruned)
	})
	t.Run("blobs to delete", func(t *testing.T) {
		fs, bs, err := NewEphemeralBlobStorageWithFs(t)
		require.NoError(t, err)
		var slot primitives.Slot = 0
		_, sidecars := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, slot, 2)
		scs, err := verification.BlobSidecarSliceNoop(sidecars)
		require.NoError(t, err)

		require.NoError(t, bs.Save(scs[0]))
		require.NoError(t, bs.Save(scs[1]))

		// check that the root->slot is cached
		root := fmt.Sprintf("%#x", scs[0].BlockRoot())
		cs, cok := bs.pruner.slotMap.slot(root)
		require.Equal(t, true, cok)
		require.Equal(t, slot, cs)

		// ensure that we see the saved files in the filesystem
		files, err := listDir(fs, root)
		require.NoError(t, err)
		require.Equal(t, 2, len(files))

		pruned, err := bs.pruner.tryPruneDir(root, slot+1)
		require.NoError(t, err)
		require.Equal(t, 2, pruned)
		files, err = listDir(fs, root)
		require.ErrorIs(t, err, os.ErrNotExist)
		require.Equal(t, 0, len(files))
	})
}

func TestTryPruneDir_SlotFromFile(t *testing.T) {
	t.Run("expired blobs deleted", func(t *testing.T) {
		fs, bs, err := NewEphemeralBlobStorageWithFs(t)
		require.NoError(t, err)
		var slot primitives.Slot = 0
		_, sidecars := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, slot, 2)
		scs, err := verification.BlobSidecarSliceNoop(sidecars)
		require.NoError(t, err)

		require.NoError(t, bs.Save(scs[0]))
		require.NoError(t, bs.Save(scs[1]))

		// check that the root->slot is cached
		root := fmt.Sprintf("%#x", scs[0].BlockRoot())
		cs, ok := bs.pruner.slotMap.slot(root)
		require.Equal(t, true, ok)
		require.Equal(t, slot, cs)
		// evict it from the cache so that we trigger the file read path
		bs.pruner.slotMap.evict(root)
		_, ok = bs.pruner.slotMap.slot(root)
		require.Equal(t, false, ok)

		// ensure that we see the saved files in the filesystem
		files, err := listDir(fs, root)
		require.NoError(t, err)
		require.Equal(t, 2, len(files))

		pruned, err := bs.pruner.tryPruneDir(root, slot+1)
		require.NoError(t, err)
		require.Equal(t, 2, pruned)
		files, err = listDir(fs, root)
		require.ErrorIs(t, err, os.ErrNotExist)
		require.Equal(t, 0, len(files))
	})
	t.Run("not expired, intact", func(t *testing.T) {
		fs, bs, err := NewEphemeralBlobStorageWithFs(t)
		require.NoError(t, err)
		// Set slot equal to the window size, so it should be retained.
		var slot primitives.Slot = bs.pruner.windowSize
		_, sidecars := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, slot, 2)
		scs, err := verification.BlobSidecarSliceNoop(sidecars)
		require.NoError(t, err)

		require.NoError(t, bs.Save(scs[0]))
		require.NoError(t, bs.Save(scs[1]))

		// Evict slot mapping from the cache so that we trigger the file read path.
		root := fmt.Sprintf("%#x", scs[0].BlockRoot())
		bs.pruner.slotMap.evict(root)
		_, ok := bs.pruner.slotMap.slot(root)
		require.Equal(t, false, ok)

		// Ensure that we see the saved files in the filesystem.
		files, err := listDir(fs, root)
		require.NoError(t, err)
		require.Equal(t, 2, len(files))

		// This should use the slotFromFile code (simulating restart).
		// Setting pruneBefore == slot, so that the slot will be outside the window (at the boundary).
		pruned, err := bs.pruner.tryPruneDir(root, slot)
		require.NoError(t, err)
		require.Equal(t, 0, pruned)

		// Ensure files are still present.
		files, err = listDir(fs, root)
		require.NoError(t, err)
		require.Equal(t, 2, len(files))
	})
}

func TestSlotFromBlob(t *testing.T) {
	cases := []struct {
		slot primitives.Slot
	}{
		{slot: 0},
		{slot: 2},
		{slot: 1123581321},
		{slot: math.MaxUint64},
	}
	for _, c := range cases {
		t.Run(fmt.Sprintf("slot %d", c.slot), func(t *testing.T) {
			_, sidecars := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, c.slot, 1)
			sc := sidecars[0]
			enc, err := sc.MarshalSSZ()
			require.NoError(t, err)
			slot, err := slotFromBlob(bytes.NewReader(enc))
			require.NoError(t, err)
			require.Equal(t, c.slot, slot)
		})
	}
}

func TestSlotFromFile(t *testing.T) {
	cases := []struct {
		slot primitives.Slot
	}{
		{slot: 0},
		{slot: 2},
		{slot: 1123581321},
		{slot: math.MaxUint64},
	}
	for _, c := range cases {
		t.Run(fmt.Sprintf("slot %d", c.slot), func(t *testing.T) {
			fs, bs, err := NewEphemeralBlobStorageWithFs(t)
			require.NoError(t, err)
			_, sidecars := util.GenerateTestDenebBlockWithSidecar(t, [32]byte{}, c.slot, 1)
			sc, err := verification.BlobSidecarNoop(sidecars[0])
			require.NoError(t, err)
			require.NoError(t, bs.Save(sc))
			fname := namerForSidecar(sc)
			sszPath := fname.path()
			slot, err := slotFromFile(sszPath, fs)
			require.NoError(t, err)
			require.Equal(t, c.slot, slot)
		})
	}
}

type dirFiles struct {
	name     string
	isDir    bool
	children []dirFiles
}

func (df dirFiles) reify(t *testing.T, fs afero.Fs, base string) {
	fullPath := path.Join(base, df.name)
	if df.isDir {
		if df.name != "" {
			require.NoError(t, fs.Mkdir(fullPath, directoryPermissions))
		}
		for _, c := range df.children {
			c.reify(t, fs, fullPath)
		}
	} else {
		fp, err := fs.Create(fullPath)
		require.NoError(t, err)
		_, err = fp.WriteString("derp")
		require.NoError(t, err)
	}
}

func (df dirFiles) childNames() []string {
	cn := make([]string, len(df.children))
	for i := range df.children {
		cn[i] = df.children[i].name
	}
	return cn
}

func TestListDir(t *testing.T) {
	fs := afero.NewMemMapFs()

	// parent directory
	fsLayout := dirFiles{isDir: true}
	// break out each subdir for easier assertions
	notABlob := dirFiles{name: "notABlob", isDir: true}
	childlessBlob := dirFiles{name: "0x0987654321", isDir: true}
	blobWithSsz := dirFiles{name: "0x1123581321", isDir: true,
		children: []dirFiles{{name: "1.ssz"}, {name: "2.ssz"}},
	}
	blobWithSszAndTmp := dirFiles{name: "0x1234567890", isDir: true,
		children: []dirFiles{{name: "5.ssz"}, {name: "0.part"}}}
	fsLayout.children = append(fsLayout.children, notABlob)
	fsLayout.children = append(fsLayout.children, childlessBlob)
	fsLayout.children = append(fsLayout.children, blobWithSsz)
	fsLayout.children = append(fsLayout.children, blobWithSszAndTmp)

	topChildren := make([]string, len(fsLayout.children))
	for i := range fsLayout.children {
		topChildren[i] = fsLayout.children[i].name
	}

	fsLayout.reify(t, fs, "")
	cases := []struct {
		name     string
		dirPath  string
		expected []string
		filter   func(string) bool
		err      error
	}{
		{
			name:     "non-existent",
			dirPath:  "derp",
			expected: []string{},
			err:      os.ErrNotExist,
		},
		{
			name:     "empty",
			dirPath:  childlessBlob.name,
			expected: []string{},
		},
		{
			name:     "top",
			dirPath:  ".",
			expected: topChildren,
		},
		{
			name:     "custom filter: only notABlob",
			dirPath:  ".",
			expected: []string{notABlob.name},
			filter: func(s string) bool {
				if s == notABlob.name {
					return true
				}
				return false
			},
		},
		{
			name:     "root filter",
			dirPath:  ".",
			expected: []string{childlessBlob.name, blobWithSsz.name, blobWithSszAndTmp.name},
			filter:   filterRoot,
		},
		{
			name:     "ssz filter",
			dirPath:  blobWithSsz.name,
			expected: blobWithSsz.childNames(),
			filter:   filterSsz,
		},
		{
			name:     "ssz mixed filter",
			dirPath:  blobWithSszAndTmp.name,
			expected: []string{"5.ssz"},
			filter:   filterSsz,
		},
	}
	for _, c := range cases {
		t.Run(c.name, func(t *testing.T) {
			result, err := listDir(fs, c.dirPath)
			if c.filter != nil {
				result = filter(result, c.filter)
			}
			if c.err != nil {
				require.ErrorIs(t, err, c.err)
				require.Equal(t, 0, len(result))
			} else {
				require.NoError(t, err)
				sort.Strings(c.expected)
				sort.Strings(result)
				require.DeepEqual(t, c.expected, result)
			}
		})
	}
}
