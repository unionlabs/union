// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package objstorageprovider

import (
	"fmt"
	"math/rand"
	"testing"

	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/stretchr/testify/require"
)

func TestSharedObjectNames(t *testing.T) {
	t.Run("crosscheck", func(t *testing.T) {
		supportedFileTypes := []base.FileType{
			base.FileTypeTable,
		}
		for it := 0; it < 100; it++ {
			var meta objstorage.ObjectMetadata
			meta.DiskFileNum = base.FileNum(rand.Intn(100000)).DiskFileNum()
			meta.FileType = supportedFileTypes[rand.Int()%len(supportedFileTypes)]
			meta.Remote.CreatorID = objstorage.CreatorID(rand.Int63())
			meta.Remote.CreatorFileNum = base.FileNum(rand.Intn(100000)).DiskFileNum()
			if rand.Intn(4) == 0 {
				meta.Remote.CustomObjectName = fmt.Sprintf("foo-%d.sst", rand.Intn(10000))
			}

			obj := remoteObjectName(meta)
			// Cross-check against cleaner implementations.
			expObj := meta.Remote.CustomObjectName
			if expObj == "" {
				expObj = fmt.Sprintf("%04x-%s-%s", objHash(meta), meta.Remote.CreatorID, base.MakeFilename(meta.FileType, meta.Remote.CreatorFileNum))
			}
			require.Equal(t, expObj, obj)

			require.Equal(t, expObj+".ref.", sharedObjectRefPrefix(meta))

			refCreatorID := objstorage.CreatorID(rand.Int63())
			refObj := sharedObjectRefName(meta, refCreatorID, meta.DiskFileNum)
			expRefObj := fmt.Sprintf("%s.ref.%s.%s", expObj, refCreatorID, meta.DiskFileNum)
			require.Equal(t, refObj, expRefObj)
		}
	})

	t.Run("example", func(t *testing.T) {
		var meta objstorage.ObjectMetadata
		meta.DiskFileNum = base.FileNum(123).DiskFileNum()
		meta.FileType = base.FileTypeTable
		meta.Remote.CreatorID = objstorage.CreatorID(456)
		meta.Remote.CreatorFileNum = base.FileNum(789).DiskFileNum()
		require.Equal(t, remoteObjectName(meta), "0e17-456-000789.sst")
		require.Equal(t, sharedObjectRefPrefix(meta), "0e17-456-000789.sst.ref.")

		refCreatorID := objstorage.CreatorID(101112)
		require.Equal(
			t, sharedObjectRefName(meta, refCreatorID, meta.DiskFileNum),
			"0e17-456-000789.sst.ref.101112.000123",
		)
	})
}
