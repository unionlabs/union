/*
Copyright © 2020 ConsenSys

Licensed under the Apache License, Version 2.0 (the "License");
you may not use this file except in compliance with the License.
You may obtain a copy of the License at

    http://www.apache.org/licenses/LICENSE-2.0

Unless required by applicable law or agreed to in writing, software
distributed under the License is distributed on an "AS IS" BASIS,
WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
See the License for the specific language governing permissions and
limitations under the License.
*/

// Package io offers serialization interfaces for gnark objects.
package io

import (
	"io"
)

// WriterRawTo is the interface that wraps the WriteRawTo method.
//
// WriteRawTo writes data to w until there's no more data to write or
// when an error occurs. The return value n is the number of bytes
// written. Any error encountered during the write is also returned.
//
// WriteRawTo will not compress the data (as opposed to WriteTo)
type WriterRawTo interface {
	WriteRawTo(w io.Writer) (n int64, err error)
}

// UnsafeReaderFrom is the interface that wraps the UnsafeReadFrom method.
//
// UnsafeReadFrom reads data from reader but doesn't perform any checks, such as
// subgroup checks for elliptic curves points for example.
type UnsafeReaderFrom interface {
	UnsafeReadFrom(r io.Reader) (int64, error)
}

// BinaryDumper is the interface that wraps the WriteDump and ReadDump methods.
// WriteDump writes the object to w, ReadDump reads the object from r.
// The object is serialized in binary format, in a very fast, very unsafe way.
type BinaryDumper interface {
	WriteDump(w io.Writer) error
	ReadDump(r io.Reader) error
}
