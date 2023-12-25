// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package objstorageprovider

import (
	"fmt"

	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/objstorage"
)

// remoteObjectName returns the name of an object on remote storage.
//
// For sstables, the format is: <hash>-<creator-id>-<file-num>.sst
// For example: 1a3f-2-000001.sst
func remoteObjectName(meta objstorage.ObjectMetadata) string {
	if meta.Remote.CustomObjectName != "" {
		return meta.Remote.CustomObjectName
	}
	switch meta.FileType {
	case base.FileTypeTable:
		return fmt.Sprintf(
			"%04x-%d-%06d.sst",
			objHash(meta), meta.Remote.CreatorID, meta.Remote.CreatorFileNum.FileNum(),
		)
	}
	panic("unknown FileType")
}

// sharedObjectRefName returns the name of the object's ref marker associated
// with a given referencing provider. This name is the object's name concatenated with
// ".ref.<ref-creator-id>.<local-file-num>".
//
// For example: 1a3f-2-000001.sst.ref.5.000008
func sharedObjectRefName(
	meta objstorage.ObjectMetadata, refCreatorID objstorage.CreatorID, refFileNum base.DiskFileNum,
) string {
	if meta.Remote.CleanupMethod != objstorage.SharedRefTracking {
		panic("ref object used when ref tracking disabled")
	}
	if meta.Remote.CustomObjectName != "" {
		return fmt.Sprintf(
			"%s.ref.%d.%06d", meta.Remote.CustomObjectName, refCreatorID, refFileNum.FileNum(),
		)
	}
	switch meta.FileType {
	case base.FileTypeTable:
		return fmt.Sprintf(
			"%04x-%d-%06d.sst.ref.%d.%06d",
			objHash(meta), meta.Remote.CreatorID, meta.Remote.CreatorFileNum.FileNum(), refCreatorID, refFileNum.FileNum(),
		)
	}
	panic("unknown FileType")
}

func sharedObjectRefPrefix(meta objstorage.ObjectMetadata) string {
	if meta.Remote.CustomObjectName != "" {
		return meta.Remote.CustomObjectName + ".ref."
	}
	switch meta.FileType {
	case base.FileTypeTable:
		return fmt.Sprintf(
			"%04x-%d-%06d.sst.ref.",
			objHash(meta), meta.Remote.CreatorID, meta.Remote.CreatorFileNum.FileNum(),
		)
	}
	panic("unknown FileType")
}

// sharedObjectRefName returns the name of the object's ref marker associated
// with this provider. This name is the object's name concatenated with
// ".ref.<creator-id>.<local-file-num>".
//
// For example: 1a3f-2-000001.sst.ref.5.000008
func (p *provider) sharedObjectRefName(meta objstorage.ObjectMetadata) string {
	if meta.Remote.CleanupMethod != objstorage.SharedRefTracking {
		panic("ref object used when ref tracking disabled")
	}
	return sharedObjectRefName(meta, p.remote.shared.creatorID, meta.DiskFileNum)
}

// objHash returns a 16-bit hash value derived from the creator ID and creator
// file num. We prepend this value to object names to ensure balanced
// partitioning with AWS (and likely other blob storage providers).
func objHash(meta objstorage.ObjectMetadata) uint16 {
	const prime1 = 7459
	const prime2 = 17539
	return uint16(uint64(meta.Remote.CreatorID)*prime1 + uint64(meta.Remote.CreatorFileNum.FileNum())*prime2)
}
