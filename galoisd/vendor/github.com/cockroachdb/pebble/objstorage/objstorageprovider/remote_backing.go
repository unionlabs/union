// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package objstorageprovider

import (
	"bytes"
	"encoding/binary"
	"io"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/cockroachdb/pebble/objstorage/objstorageprovider/remoteobjcat"
	"github.com/cockroachdb/pebble/objstorage/remote"
)

const (
	tagCreatorID      = 1
	tagCreatorFileNum = 2
	tagCleanupMethod  = 3
	// tagRefCheckID encodes the information for a ref marker that needs to be
	// checked when attaching this object to another provider. This is set to the
	// creator ID and FileNum for the provider that encodes the backing, and
	// allows the "target" provider to check that the "source" provider kept its
	// reference on the object alive.
	tagRefCheckID = 4
	// tagLocator encodes the remote.Locator; if absent the locator is "". It is
	// followed by the locator string length and the locator string.
	tagLocator = 5
	// tagLocator encodes a custom object name (if present). It is followed by the
	// custom name string length and the string.
	tagCustomObjectName = 6

	// Any new tags that don't have the tagNotSafeToIgnoreMask bit set must be
	// followed by the length of the data (so they can be skipped).

	// Any new tags that have the tagNotSafeToIgnoreMask bit set cause errors if
	// they are encountered by earlier code that doesn't know the tag.
	tagNotSafeToIgnoreMask = 64
)

func (p *provider) encodeRemoteObjectBacking(
	meta *objstorage.ObjectMetadata,
) (objstorage.RemoteObjectBacking, error) {
	if !meta.IsRemote() {
		return nil, errors.AssertionFailedf("object %s not on remote storage", meta.DiskFileNum)
	}

	buf := make([]byte, 0, binary.MaxVarintLen64*4)
	buf = binary.AppendUvarint(buf, tagCreatorID)
	buf = binary.AppendUvarint(buf, uint64(meta.Remote.CreatorID))
	// TODO(radu): encode file type as well?
	buf = binary.AppendUvarint(buf, tagCreatorFileNum)
	buf = binary.AppendUvarint(buf, uint64(meta.Remote.CreatorFileNum.FileNum()))
	buf = binary.AppendUvarint(buf, tagCleanupMethod)
	buf = binary.AppendUvarint(buf, uint64(meta.Remote.CleanupMethod))
	if meta.Remote.CleanupMethod == objstorage.SharedRefTracking {
		buf = binary.AppendUvarint(buf, tagRefCheckID)
		buf = binary.AppendUvarint(buf, uint64(p.remote.shared.creatorID))
		buf = binary.AppendUvarint(buf, uint64(meta.DiskFileNum.FileNum()))
	}
	if meta.Remote.Locator != "" {
		buf = binary.AppendUvarint(buf, tagLocator)
		buf = encodeString(buf, string(meta.Remote.Locator))
	}
	if meta.Remote.CustomObjectName != "" {
		buf = binary.AppendUvarint(buf, tagCustomObjectName)
		buf = encodeString(buf, meta.Remote.CustomObjectName)
	}
	return buf, nil
}

type remoteObjectBackingHandle struct {
	backing objstorage.RemoteObjectBacking
	fileNum base.DiskFileNum
	p       *provider
}

func (s *remoteObjectBackingHandle) Get() (objstorage.RemoteObjectBacking, error) {
	if s.backing == nil {
		return nil, errors.Errorf("RemoteObjectBackingHandle.Get() called after Close()")
	}
	return s.backing, nil
}

func (s *remoteObjectBackingHandle) Close() {
	if s.backing != nil {
		s.backing = nil
		s.p.unprotectObject(s.fileNum)
	}
}

var _ objstorage.RemoteObjectBackingHandle = (*remoteObjectBackingHandle)(nil)

// RemoteObjectBacking is part of the objstorage.Provider interface.
func (p *provider) RemoteObjectBacking(
	meta *objstorage.ObjectMetadata,
) (objstorage.RemoteObjectBackingHandle, error) {
	backing, err := p.encodeRemoteObjectBacking(meta)
	if err != nil {
		return nil, err
	}
	p.protectObject(meta.DiskFileNum)
	return &remoteObjectBackingHandle{
		backing: backing,
		fileNum: meta.DiskFileNum,
		p:       p,
	}, nil
}

// CreateExternalObjectBacking is part of the objstorage.Provider interface.
func (p *provider) CreateExternalObjectBacking(
	locator remote.Locator, objName string,
) (objstorage.RemoteObjectBacking, error) {
	var meta objstorage.ObjectMetadata
	meta.Remote.Locator = locator
	meta.Remote.CustomObjectName = objName
	meta.Remote.CleanupMethod = objstorage.SharedNoCleanup
	return p.encodeRemoteObjectBacking(&meta)
}

type decodedBacking struct {
	meta objstorage.ObjectMetadata
	// refToCheck is set only when meta.Remote.CleanupMethod is RefTracking
	refToCheck struct {
		creatorID objstorage.CreatorID
		fileNum   base.DiskFileNum
	}
}

// decodeRemoteObjectBacking decodes the remote object metadata.
//
// Note that the meta.Remote.Storage field is not set.
func decodeRemoteObjectBacking(
	fileType base.FileType, fileNum base.DiskFileNum, buf objstorage.RemoteObjectBacking,
) (decodedBacking, error) {
	var creatorID, creatorFileNum, cleanupMethod, refCheckCreatorID, refCheckFileNum uint64
	var locator, customObjName string
	br := bytes.NewReader(buf)
	for {
		tag, err := binary.ReadUvarint(br)
		if err == io.EOF {
			break
		}
		if err != nil {
			return decodedBacking{}, err
		}
		switch tag {
		case tagCreatorID:
			creatorID, err = binary.ReadUvarint(br)

		case tagCreatorFileNum:
			creatorFileNum, err = binary.ReadUvarint(br)

		case tagCleanupMethod:
			cleanupMethod, err = binary.ReadUvarint(br)

		case tagRefCheckID:
			refCheckCreatorID, err = binary.ReadUvarint(br)
			if err == nil {
				refCheckFileNum, err = binary.ReadUvarint(br)
			}

		case tagLocator:
			locator, err = decodeString(br)

		case tagCustomObjectName:
			customObjName, err = decodeString(br)

		default:
			// Ignore unknown tags, unless they're not safe to ignore.
			if tag&tagNotSafeToIgnoreMask != 0 {
				return decodedBacking{}, errors.Newf("unknown tag %d", tag)
			}
			var dataLen uint64
			dataLen, err = binary.ReadUvarint(br)
			if err == nil {
				_, err = br.Seek(int64(dataLen), io.SeekCurrent)
			}
		}
		if err != nil {
			return decodedBacking{}, err
		}
	}
	if customObjName == "" {
		if creatorID == 0 {
			return decodedBacking{}, errors.Newf("remote object backing missing creator ID")
		}
		if creatorFileNum == 0 {
			return decodedBacking{}, errors.Newf("remote object backing missing creator file num")
		}
	}
	var res decodedBacking
	res.meta.DiskFileNum = fileNum
	res.meta.FileType = fileType
	res.meta.Remote.CreatorID = objstorage.CreatorID(creatorID)
	res.meta.Remote.CreatorFileNum = base.FileNum(creatorFileNum).DiskFileNum()
	res.meta.Remote.CleanupMethod = objstorage.SharedCleanupMethod(cleanupMethod)
	if res.meta.Remote.CleanupMethod == objstorage.SharedRefTracking {
		if refCheckCreatorID == 0 || refCheckFileNum == 0 {
			return decodedBacking{}, errors.Newf("remote object backing missing ref to check")
		}
		res.refToCheck.creatorID = objstorage.CreatorID(refCheckCreatorID)
		res.refToCheck.fileNum = base.FileNum(refCheckFileNum).DiskFileNum()
	}
	res.meta.Remote.Locator = remote.Locator(locator)
	res.meta.Remote.CustomObjectName = customObjName
	return res, nil
}

func encodeString(buf []byte, s string) []byte {
	buf = binary.AppendUvarint(buf, uint64(len(s)))
	buf = append(buf, []byte(s)...)
	return buf
}

func decodeString(br io.ByteReader) (string, error) {
	length, err := binary.ReadUvarint(br)
	if err != nil || length == 0 {
		return "", err
	}
	buf := make([]byte, length)
	for i := range buf {
		buf[i], err = br.ReadByte()
		if err != nil {
			return "", err
		}
	}
	return string(buf), nil
}

// AttachRemoteObjects is part of the objstorage.Provider interface.
func (p *provider) AttachRemoteObjects(
	objs []objstorage.RemoteObjectToAttach,
) ([]objstorage.ObjectMetadata, error) {
	decoded := make([]decodedBacking, len(objs))
	for i, o := range objs {
		var err error
		decoded[i], err = decodeRemoteObjectBacking(o.FileType, o.FileNum, o.Backing)
		if err != nil {
			return nil, err
		}
		decoded[i].meta.Remote.Storage, err = p.ensureStorage(decoded[i].meta.Remote.Locator)
		if err != nil {
			return nil, err
		}
	}

	// Create the reference marker objects.
	// TODO(radu): parallelize this.
	for _, d := range decoded {
		if d.meta.Remote.CleanupMethod != objstorage.SharedRefTracking {
			continue
		}
		if err := p.sharedCreateRef(d.meta); err != nil {
			// TODO(radu): clean up references previously created in this loop.
			return nil, err
		}
		// Check the "origin"'s reference.
		refName := sharedObjectRefName(d.meta, d.refToCheck.creatorID, d.refToCheck.fileNum)
		if _, err := d.meta.Remote.Storage.Size(refName); err != nil {
			_ = p.sharedUnref(d.meta)
			// TODO(radu): clean up references previously created in this loop.
			if d.meta.Remote.Storage.IsNotExistError(err) {
				return nil, errors.Errorf("origin marker object %q does not exist;"+
					" object probably removed from the provider which created the backing", refName)
			}
			return nil, errors.Wrapf(err, "checking origin's marker object %s", refName)
		}
	}

	func() {
		p.mu.Lock()
		defer p.mu.Unlock()
		for _, d := range decoded {
			p.mu.remote.catalogBatch.AddObject(remoteobjcat.RemoteObjectMetadata{
				FileNum:        d.meta.DiskFileNum,
				FileType:       d.meta.FileType,
				CreatorID:      d.meta.Remote.CreatorID,
				CreatorFileNum: d.meta.Remote.CreatorFileNum,
				CleanupMethod:  d.meta.Remote.CleanupMethod,
				Locator:        d.meta.Remote.Locator,
			})
		}
	}()
	if err := p.sharedSync(); err != nil {
		return nil, err
	}

	metas := make([]objstorage.ObjectMetadata, len(decoded))
	for i, d := range decoded {
		metas[i] = d.meta
	}

	p.mu.Lock()
	defer p.mu.Unlock()
	for _, meta := range metas {
		p.mu.knownObjects[meta.DiskFileNum] = meta
	}
	return metas, nil
}
