// Copyright 2023 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package remoteobjcat

import (
	"bytes"
	"strings"
	"testing"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/objstorage"
	"github.com/kr/pretty"
)

func TestVersionEditRoundTrip(t *testing.T) {
	for _, ve := range []VersionEdit{
		{},
		{
			CreatorID: 12345,
		},
		{
			NewObjects: []RemoteObjectMetadata{
				{
					FileNum:          base.FileNum(1).DiskFileNum(),
					FileType:         base.FileTypeTable,
					CreatorID:        12,
					CreatorFileNum:   base.FileNum(123).DiskFileNum(),
					CleanupMethod:    objstorage.SharedNoCleanup,
					Locator:          "",
					CustomObjectName: "foo",
				},
			},
		},
		{
			DeletedObjects: []base.DiskFileNum{base.FileNum(1).DiskFileNum()},
		},
		{
			CreatorID: 12345,
			NewObjects: []RemoteObjectMetadata{
				{
					FileNum:          base.FileNum(1).DiskFileNum(),
					FileType:         base.FileTypeTable,
					CreatorID:        12,
					CreatorFileNum:   base.FileNum(123).DiskFileNum(),
					CleanupMethod:    objstorage.SharedRefTracking,
					Locator:          "foo",
					CustomObjectName: "",
				},
				{
					FileNum:          base.FileNum(2).DiskFileNum(),
					FileType:         base.FileTypeTable,
					CreatorID:        22,
					CreatorFileNum:   base.FileNum(223).DiskFileNum(),
					Locator:          "bar",
					CustomObjectName: "obj1",
				},
				{
					FileNum:          base.FileNum(3).DiskFileNum(),
					FileType:         base.FileTypeTable,
					CreatorID:        32,
					CreatorFileNum:   base.FileNum(323).DiskFileNum(),
					CleanupMethod:    objstorage.SharedRefTracking,
					Locator:          "baz",
					CustomObjectName: "obj2",
				},
			},
			DeletedObjects: []base.DiskFileNum{base.FileNum(4).DiskFileNum(), base.FileNum(5).DiskFileNum()},
		},
	} {
		if err := checkRoundTrip(ve); err != nil {
			t.Fatalf("%+v did not roundtrip: %v", ve, err)
		}
	}
}

func checkRoundTrip(e0 VersionEdit) error {
	var e1 VersionEdit
	buf := new(bytes.Buffer)
	if err := e0.Encode(buf); err != nil {
		return errors.Wrap(err, "encode")
	}
	if err := e1.Decode(buf); err != nil {
		return errors.Wrap(err, "decode")
	}
	if diff := pretty.Diff(e0, e1); diff != nil {
		return errors.Errorf("%s", strings.Join(diff, "\n"))
	}
	return nil
}
