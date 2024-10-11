// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package manifest

import (
	"bytes"
	"fmt"
	"strings"
	"testing"

	"github.com/cockroachdb/datadriven"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/stretchr/testify/require"
)

func TestLevelIterator(t *testing.T) {
	var level LevelSlice
	datadriven.RunTest(t, "testdata/level_iterator",
		func(t *testing.T, d *datadriven.TestData) string {
			switch d.Cmd {
			case "define":
				var files []*FileMetadata
				var startReslice int
				var endReslice int
				for _, metaStr := range strings.Split(d.Input, " ") {
					switch metaStr {
					case "[":
						startReslice = len(files)
						continue
					case "]":
						endReslice = len(files)
						continue
					case " ", "":
						continue
					default:
						parts := strings.Split(metaStr, "-")
						if len(parts) != 2 {
							t.Fatalf("malformed table spec: %q", metaStr)
						}
						m := &FileMetadata{FileNum: base.FileNum(len(files) + 1)}
						m.ExtendPointKeyBounds(
							base.DefaultComparer.Compare,
							base.ParseInternalKey(strings.TrimSpace(parts[0])),
							base.ParseInternalKey(strings.TrimSpace(parts[1])),
						)
						m.SmallestSeqNum = m.Smallest.SeqNum()
						m.LargestSeqNum = m.Largest.SeqNum()
						m.InitPhysicalBacking()
						files = append(files, m)
					}
				}
				level = NewLevelSliceKeySorted(base.DefaultComparer.Compare, files)
				level = level.Reslice(func(start, end *LevelIterator) {
					for i := 0; i < startReslice; i++ {
						start.Next()
					}
					for i := len(files); i > endReslice; i-- {
						end.Prev()
					}
				})
				return ""

			case "iter":
				return runIterCmd(t, d, level.Iter(), false /* verbose */)

			default:
				return fmt.Sprintf("unknown command %q", d.Cmd)
			}
		})
}

func TestLevelIteratorFiltered(t *testing.T) {
	var level LevelSlice
	datadriven.RunTest(t, "testdata/level_iterator_filtered",
		func(t *testing.T, d *datadriven.TestData) string {
			switch d.Cmd {
			case "define":
				var files []*FileMetadata
				for _, metaStr := range strings.Split(d.Input, "\n") {
					m, err := ParseFileMetadataDebug(metaStr)
					require.NoError(t, err)
					files = append(files, m)
				}
				level = NewLevelSliceKeySorted(base.DefaultComparer.Compare, files)
				return ""

			case "iter":
				var keyType string
				d.ScanArgs(t, "key-type", &keyType)
				iter := level.Iter()
				switch keyType {
				case "both":
					// noop
				case "points":
					iter = iter.Filter(KeyTypePoint)
				case "ranges":
					iter = iter.Filter(KeyTypeRange)
				}
				return runIterCmd(t, d, iter, true /* verbose */)

			default:
				return fmt.Sprintf("unknown command %q", d.Cmd)
			}
		})
}

func runIterCmd(t *testing.T, d *datadriven.TestData, iter LevelIterator, verbose bool) string {
	var buf bytes.Buffer
	for _, line := range strings.Split(d.Input, "\n") {
		parts := strings.Fields(line)
		if len(parts) == 0 {
			continue
		}
		var m *FileMetadata
		switch parts[0] {
		case "first":
			m = iter.First()
		case "last":
			m = iter.Last()
		case "next":
			m = iter.Next()
		case "prev":
			m = iter.Prev()
		case "seek-ge":
			m = iter.SeekGE(base.DefaultComparer.Compare, []byte(parts[1]))
		case "seek-lt":
			m = iter.SeekLT(base.DefaultComparer.Compare, []byte(parts[1]))
		default:
			return fmt.Sprintf("unknown command %q", parts[0])
		}
		if m == nil {
			fmt.Fprintln(&buf, ".")
		} else {
			if verbose {
				fmt.Fprintln(&buf, m.DebugString(base.DefaultComparer.FormatKey, verbose))
			} else {
				fmt.Fprintln(&buf, m)
			}
		}
	}
	return buf.String()
}
