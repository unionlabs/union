// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package remoteobjcat

import (
	"bufio"
	"encoding/binary"
	"io"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/invariants"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/cockroachdb/pebble/objstorage/remote"
)

// VersionEdit is a modification to the remote object state which can be encoded
// into a record.
//
// TODO(radu): consider adding creation and deletion time for debugging purposes.
type VersionEdit struct {
	NewObjects     []RemoteObjectMetadata
	DeletedObjects []base.DiskFileNum
	CreatorID      objstorage.CreatorID
}

const (
	// tagNewObject is followed by the FileNum, creator ID, creator FileNum,
	// cleanup method, optional new object tags, and ending with a 0 byte.
	tagNewObject = 1
	// tagDeletedObject is followed by the FileNum.
	tagDeletedObject = 2
	// tagCreatorID is followed by the Creator ID for this store. This ID can
	// never change.
	tagCreatorID = 3
	// tagNewObjectLocator is an optional tag inside the tagNewObject payload. It
	// is followed by the encoded length of the locator string and the string.
	tagNewObjectLocator = 4
	// tagNewObjectCustomName is an optional tag inside the tagNewObject payload.
	// It is followed by the encoded length of the custom object name string
	// followed by the string.
	tagNewObjectCustomName = 5
)

// Object type values. We don't want to encode FileType directly because it is
// more general (and we want freedom to change it in the future).
const (
	objTypeTable = 1
)

func objTypeToFileType(objType uint64) (base.FileType, error) {
	switch objType {
	case objTypeTable:
		return base.FileTypeTable, nil
	default:
		return 0, errors.Newf("unknown object type %d", objType)
	}
}

func fileTypeToObjType(fileType base.FileType) (uint64, error) {
	switch fileType {
	case base.FileTypeTable:
		return objTypeTable, nil

	default:
		return 0, errors.Newf("unknown object type for file type %d", fileType)
	}
}

// Encode encodes an edit to the specified writer.
func (v *VersionEdit) Encode(w io.Writer) error {
	buf := make([]byte, 0, binary.MaxVarintLen64*(len(v.NewObjects)*10+len(v.DeletedObjects)*2+2))
	for _, meta := range v.NewObjects {
		objType, err := fileTypeToObjType(meta.FileType)
		if err != nil {
			return err
		}
		buf = binary.AppendUvarint(buf, uint64(tagNewObject))
		buf = binary.AppendUvarint(buf, uint64(meta.FileNum.FileNum()))
		buf = binary.AppendUvarint(buf, objType)
		buf = binary.AppendUvarint(buf, uint64(meta.CreatorID))
		buf = binary.AppendUvarint(buf, uint64(meta.CreatorFileNum.FileNum()))
		buf = binary.AppendUvarint(buf, uint64(meta.CleanupMethod))
		if meta.Locator != "" {
			buf = binary.AppendUvarint(buf, uint64(tagNewObjectLocator))
			buf = encodeString(buf, string(meta.Locator))
		}
		if meta.CustomObjectName != "" {
			buf = binary.AppendUvarint(buf, uint64(tagNewObjectCustomName))
			buf = encodeString(buf, meta.CustomObjectName)
		}
		// Append 0 as the terminator for optional new object tags.
		buf = binary.AppendUvarint(buf, 0)
	}

	for _, dfn := range v.DeletedObjects {
		buf = binary.AppendUvarint(buf, uint64(tagDeletedObject))
		buf = binary.AppendUvarint(buf, uint64(dfn.FileNum()))
	}
	if v.CreatorID.IsSet() {
		buf = binary.AppendUvarint(buf, uint64(tagCreatorID))
		buf = binary.AppendUvarint(buf, uint64(v.CreatorID))
	}
	_, err := w.Write(buf)
	return err
}

// Decode decodes an edit from the specified reader.
func (v *VersionEdit) Decode(r io.Reader) error {
	br, ok := r.(io.ByteReader)
	if !ok {
		br = bufio.NewReader(r)
	}
	for {
		tag, err := binary.ReadUvarint(br)
		if err == io.EOF {
			break
		}
		if err != nil {
			return err
		}

		err = nil
		switch tag {
		case tagNewObject:
			var fileNum, creatorID, creatorFileNum, cleanupMethod uint64
			var locator, customName string
			var fileType base.FileType
			fileNum, err = binary.ReadUvarint(br)
			if err == nil {
				var objType uint64
				objType, err = binary.ReadUvarint(br)
				if err == nil {
					fileType, err = objTypeToFileType(objType)
				}
			}
			if err == nil {
				creatorID, err = binary.ReadUvarint(br)
			}
			if err == nil {
				creatorFileNum, err = binary.ReadUvarint(br)
			}
			if err == nil {
				cleanupMethod, err = binary.ReadUvarint(br)
			}
			for err == nil {
				var optionalTag uint64
				optionalTag, err = binary.ReadUvarint(br)
				if err != nil || optionalTag == 0 {
					break
				}

				switch optionalTag {
				case tagNewObjectLocator:
					locator, err = decodeString(br)

				case tagNewObjectCustomName:
					customName, err = decodeString(br)

				default:
					err = errors.Newf("unknown newObject tag %d", optionalTag)
				}
			}

			if err == nil {
				v.NewObjects = append(v.NewObjects, RemoteObjectMetadata{
					FileNum:          base.FileNum(fileNum).DiskFileNum(),
					FileType:         fileType,
					CreatorID:        objstorage.CreatorID(creatorID),
					CreatorFileNum:   base.FileNum(creatorFileNum).DiskFileNum(),
					CleanupMethod:    objstorage.SharedCleanupMethod(cleanupMethod),
					Locator:          remote.Locator(locator),
					CustomObjectName: customName,
				})
			}

		case tagDeletedObject:
			var fileNum uint64
			fileNum, err = binary.ReadUvarint(br)
			if err == nil {
				v.DeletedObjects = append(v.DeletedObjects, base.FileNum(fileNum).DiskFileNum())
			}

		case tagCreatorID:
			var id uint64
			id, err = binary.ReadUvarint(br)
			if err == nil {
				v.CreatorID = objstorage.CreatorID(id)
			}

		default:
			err = errors.Newf("unknown tag %d", tag)
		}

		if err != nil {
			if err == io.EOF {
				return errCorruptCatalog
			}
			return err
		}
	}
	return nil
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

var errCorruptCatalog = base.CorruptionErrorf("pebble: corrupt remote object catalog")

// Apply the version edit to a creator ID and a map of objects.
func (v *VersionEdit) Apply(
	creatorID *objstorage.CreatorID, objects map[base.DiskFileNum]RemoteObjectMetadata,
) error {
	if v.CreatorID.IsSet() {
		*creatorID = v.CreatorID
	}
	for _, meta := range v.NewObjects {
		if invariants.Enabled {
			if _, exists := objects[meta.FileNum]; exists {
				return errors.AssertionFailedf("version edit adds existing object %s", meta.FileNum)
			}
		}
		objects[meta.FileNum] = meta
	}
	for _, fileNum := range v.DeletedObjects {
		if invariants.Enabled {
			if _, exists := objects[fileNum]; !exists {
				return errors.AssertionFailedf("version edit deletes non-existent object %s", fileNum)
			}
		}
		delete(objects, fileNum)
	}
	return nil
}
