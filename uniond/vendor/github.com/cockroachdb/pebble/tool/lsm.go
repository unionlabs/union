// Copyright 2020 The LevelDB-Go and Pebble Authors. All rights reserved. Use
// of this source code is governed by a BSD-style license that can be found in
// the LICENSE file.

package tool

import (
	"encoding/json"
	"fmt"
	"io"
	"log"
	"math"
	"sort"

	"github.com/cockroachdb/errors"
	"github.com/cockroachdb/pebble"
	"github.com/cockroachdb/pebble/internal/base"
	"github.com/cockroachdb/pebble/internal/manifest"
	"github.com/cockroachdb/pebble/record"
	"github.com/cockroachdb/pebble/sstable"
	"github.com/spf13/cobra"
)

//go:generate ./make_lsm_data.sh

type lsmFileMetadata struct {
	Size           uint64
	Smallest       int // ID of smallest key
	Largest        int // ID of largest key
	SmallestSeqNum uint64
	LargestSeqNum  uint64
}

type lsmVersionEdit struct {
	// Reason for the edit: flushed, ingested, compacted, added.
	Reason string
	// Map from level to files added to the level.
	Added map[int][]base.FileNum `json:",omitempty"`
	// Map from level to files deleted from the level.
	Deleted map[int][]base.FileNum `json:",omitempty"`
	// L0 sublevels for any files with changed sublevels so far.
	Sublevels map[base.FileNum]int `json:",omitempty"`
}

type lsmKey struct {
	Pretty string
	SeqNum uint64
	Kind   int
}

type lsmState struct {
	Manifest  string
	Edits     []lsmVersionEdit                 `json:",omitempty"`
	Files     map[base.FileNum]lsmFileMetadata `json:",omitempty"`
	Keys      []lsmKey                         `json:",omitempty"`
	StartEdit int64
}

type lsmT struct {
	Root *cobra.Command

	// Configuration.
	opts      *pebble.Options
	comparers sstable.Comparers

	fmtKey    keyFormatter
	embed     bool
	pretty    bool
	startEdit int64
	endEdit   int64
	editCount int64

	cmp    *base.Comparer
	state  lsmState
	keyMap map[lsmKey]int
}

func newLSM(opts *pebble.Options, comparers sstable.Comparers) *lsmT {
	l := &lsmT{
		opts:      opts,
		comparers: comparers,
	}
	l.fmtKey.mustSet("quoted")

	l.Root = &cobra.Command{
		Use:   "lsm <manifest>",
		Short: "LSM visualization tool",
		Long: `
Visualize the evolution of an LSM from the version edits in a MANIFEST.

Given an input MANIFEST, output an HTML file containing a visualization showing
the evolution of the LSM. Each version edit in the MANIFEST becomes a single
step in the visualization. The 7 levels of the LSM are depicted with each
sstable represented as a 1-pixel wide rectangle. The height of the rectangle is
proportional to the size (in bytes) of the sstable. The sstables are displayed
in the same order as they occur in the LSM. Note that the sstables from
different levels are NOT aligned according to their start and end keys (doing so
is also interesting, but it works against using the area of the rectangle to
indicate size).
`,
		Args: cobra.ExactArgs(1),
		RunE: l.runLSM,
	}

	l.Root.Flags().Var(&l.fmtKey, "key", "key formatter")
	l.Root.Flags().BoolVar(&l.embed, "embed", true, "embed javascript in HTML (disable for development)")
	l.Root.Flags().BoolVar(&l.pretty, "pretty", false, "pretty JSON output")
	l.Root.Flags().Int64Var(&l.startEdit, "start-edit", 0, "starting edit # to include in visualization")
	l.Root.Flags().Int64Var(&l.endEdit, "end-edit", math.MaxInt64, "ending edit # to include in visualization")
	l.Root.Flags().Int64Var(&l.editCount, "edit-count", math.MaxInt64, "count of edits to include in visualization")
	return l
}

func (l *lsmT) isFlagSet(name string) bool {
	return l.Root.Flags().Changed(name)
}

func (l *lsmT) validateFlags() error {
	if l.isFlagSet("edit-count") {
		if l.isFlagSet("start-edit") && l.isFlagSet("end-edit") {
			return errors.Errorf("edit-count cannot be provided with both start-edit and end-edit")
		} else if l.isFlagSet("end-edit") {
			return errors.Errorf("cannot use edit-count with end-edit, use start-edit and end-edit instead")
		}
	}

	if l.startEdit > l.endEdit {
		return errors.Errorf("start-edit cannot be after end-edit")
	}

	return nil
}

func (l *lsmT) runLSM(cmd *cobra.Command, args []string) error {
	err := l.validateFlags()
	if err != nil {
		return err
	}

	edits := l.readManifest(args[0])
	if edits == nil {
		return nil
	}

	if l.startEdit > 0 {
		edits, err = l.coalesceEdits(edits)
		if err != nil {
			return err
		}
	}
	if l.endEdit < int64(len(edits)) {
		edits = edits[:l.endEdit-l.startEdit+1]
	}
	if l.editCount < int64(len(edits)) {
		edits = edits[:l.editCount]
	}

	l.buildKeys(edits)
	err = l.buildEdits(edits)
	if err != nil {
		return err
	}

	w := l.Root.OutOrStdout()

	fmt.Fprintf(w, `<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1, shrink-to-fit=no">
`)
	if l.embed {
		fmt.Fprintf(w, "<style>%s</style>\n", lsmDataCSS)
	} else {
		fmt.Fprintf(w, "<link rel=\"stylesheet\" href=\"data/lsm.css\">\n")
	}
	fmt.Fprintf(w, "</head>\n<body>\n")
	if l.embed {
		fmt.Fprintf(w, "<script src=\"https://d3js.org/d3.v5.min.js\"></script>\n")
	} else {
		fmt.Fprintf(w, "<script src=\"data/d3.v5.min.js\"></script>\n")
	}
	fmt.Fprintf(w, "<script type=\"text/javascript\">\n")
	fmt.Fprintf(w, "data = %s\n", l.formatJSON(l.state))
	fmt.Fprintf(w, "</script>\n")
	if l.embed {
		fmt.Fprintf(w, "<script type=\"text/javascript\">%s</script>\n", lsmDataJS)
	} else {
		fmt.Fprintf(w, "<script src=\"data/lsm.js\"></script>\n")
	}
	fmt.Fprintf(w, "</body>\n</html>\n")

	return nil
}

func (l *lsmT) readManifest(path string) []*manifest.VersionEdit {
	f, err := l.opts.FS.Open(path)
	if err != nil {
		fmt.Fprintf(l.Root.OutOrStderr(), "%s\n", err)
		return nil
	}
	defer f.Close()

	l.state.Manifest = path

	var edits []*manifest.VersionEdit
	w := l.Root.OutOrStdout()
	rr := record.NewReader(f, 0 /* logNum */)
	for i := 0; ; i++ {
		r, err := rr.Next()
		if err != nil {
			if err != io.EOF {
				fmt.Fprintf(w, "%s\n", err)
			}
			break
		}

		ve := &manifest.VersionEdit{}
		err = ve.Decode(r)
		if err != nil {
			fmt.Fprintf(w, "%s\n", err)
			break
		}
		edits = append(edits, ve)

		if ve.ComparerName != "" {
			l.cmp = l.comparers[ve.ComparerName]
			if l.cmp == nil {
				fmt.Fprintf(w, "%d: unknown comparer %q\n", i, ve.ComparerName)
				return nil
			}
			l.fmtKey.setForComparer(ve.ComparerName, l.comparers)
		} else if l.cmp == nil {
			l.cmp = base.DefaultComparer
		}
	}
	return edits
}

func (l *lsmT) buildKeys(edits []*manifest.VersionEdit) {
	var keys []base.InternalKey
	for _, ve := range edits {
		for i := range ve.NewFiles {
			nf := &ve.NewFiles[i]
			keys = append(keys, nf.Meta.Smallest)
			keys = append(keys, nf.Meta.Largest)
		}
	}

	l.keyMap = make(map[lsmKey]int)

	sort.Slice(keys, func(i, j int) bool {
		return base.InternalCompare(l.cmp.Compare, keys[i], keys[j]) < 0
	})

	for i := range keys {
		k := &keys[i]
		if i > 0 && base.InternalCompare(l.cmp.Compare, keys[i-1], keys[i]) == 0 {
			continue
		}
		j := len(l.state.Keys)
		l.state.Keys = append(l.state.Keys, lsmKey{
			Pretty: fmt.Sprint(l.fmtKey.fn(k.UserKey)),
			SeqNum: k.SeqNum(),
			Kind:   int(k.Kind()),
		})
		l.keyMap[lsmKey{string(k.UserKey), k.SeqNum(), int(k.Kind())}] = j
	}
}

func (l *lsmT) buildEdits(edits []*manifest.VersionEdit) error {
	l.state.Edits = nil
	l.state.StartEdit = l.startEdit
	l.state.Files = make(map[base.FileNum]lsmFileMetadata)
	var currentFiles [manifest.NumLevels][]*manifest.FileMetadata

	for _, ve := range edits {
		if len(ve.DeletedFiles) == 0 && len(ve.NewFiles) == 0 {
			continue
		}

		edit := lsmVersionEdit{
			Reason:  l.reason(ve),
			Added:   make(map[int][]base.FileNum),
			Deleted: make(map[int][]base.FileNum),
		}

		for j := range ve.NewFiles {
			nf := &ve.NewFiles[j]
			if _, ok := l.state.Files[nf.Meta.FileNum]; !ok {
				l.state.Files[nf.Meta.FileNum] = lsmFileMetadata{
					Size:           nf.Meta.Size,
					Smallest:       l.findKey(nf.Meta.Smallest),
					Largest:        l.findKey(nf.Meta.Largest),
					SmallestSeqNum: nf.Meta.SmallestSeqNum,
					LargestSeqNum:  nf.Meta.LargestSeqNum,
				}
			}
			edit.Added[nf.Level] = append(edit.Added[nf.Level], nf.Meta.FileNum)
			currentFiles[nf.Level] = append(currentFiles[nf.Level], nf.Meta)
		}

		for df := range ve.DeletedFiles {
			edit.Deleted[df.Level] = append(edit.Deleted[df.Level], df.FileNum)
			for j, f := range currentFiles[df.Level] {
				if f.FileNum == df.FileNum {
					copy(currentFiles[df.Level][j:], currentFiles[df.Level][j+1:])
					currentFiles[df.Level] = currentFiles[df.Level][:len(currentFiles[df.Level])-1]
				}
			}
		}

		v := manifest.NewVersion(l.cmp.Compare, l.fmtKey.fn, 0, currentFiles)
		edit.Sublevels = make(map[base.FileNum]int)
		for sublevel, files := range v.L0SublevelFiles {
			iter := files.Iter()
			for f := iter.First(); f != nil; f = iter.Next() {
				if len(l.state.Edits) > 0 {
					lastEdit := l.state.Edits[len(l.state.Edits)-1]
					if sublevel2, ok := lastEdit.Sublevels[f.FileNum]; ok && sublevel == sublevel2 {
						continue
					}
				}
				edit.Sublevels[f.FileNum] = sublevel
			}
		}
		l.state.Edits = append(l.state.Edits, edit)
	}

	if l.state.Edits == nil {
		return errors.Errorf("there are no edits in [start-edit, end-edit], which add or delete files")
	}
	return nil
}

func (l *lsmT) coalesceEdits(edits []*manifest.VersionEdit) ([]*manifest.VersionEdit, error) {
	if l.startEdit >= int64(len(edits)) {
		return nil, errors.Errorf("start-edit is more than the number of edits, %d", len(edits))
	}

	be := manifest.BulkVersionEdit{}
	be.AddedByFileNum = make(map[base.FileNum]*manifest.FileMetadata)

	// Coalesce all edits from [0, l.startEdit) into a BulkVersionEdit.
	for _, ve := range edits[:l.startEdit] {
		err := be.Accumulate(ve)
		if err != nil {
			return nil, err
		}
	}

	startingEdit := edits[l.startEdit]
	var beNewFiles []manifest.NewFileEntry
	beDeletedFiles := make(map[manifest.DeletedFileEntry]*manifest.FileMetadata)

	for level, deletedFiles := range be.Deleted {
		for _, file := range deletedFiles {
			dfe := manifest.DeletedFileEntry{
				Level:   level,
				FileNum: file.FileNum,
			}
			beDeletedFiles[dfe] = file
		}
	}

	// Filter out added files that were also deleted in the BulkVersionEdit.
	for level, newFiles := range be.Added {
		for _, file := range newFiles {
			dfe := manifest.DeletedFileEntry{
				Level:   level,
				FileNum: file.FileNum,
			}

			if _, ok := beDeletedFiles[dfe]; !ok {
				beNewFiles = append(beNewFiles, manifest.NewFileEntry{
					Level: level,
					Meta:  file,
				})
			}
		}
	}
	startingEdit.NewFiles = append(beNewFiles, startingEdit.NewFiles...)

	edits = edits[l.startEdit:]
	return edits, nil
}

func (l *lsmT) findKey(key base.InternalKey) int {
	return l.keyMap[lsmKey{string(key.UserKey), key.SeqNum(), int(key.Kind())}]
}

func (l *lsmT) reason(ve *manifest.VersionEdit) string {
	if len(ve.DeletedFiles) > 0 {
		return "compacted"
	}
	if ve.MinUnflushedLogNum != 0 {
		return "flushed"
	}
	for i := range ve.NewFiles {
		nf := &ve.NewFiles[i]
		if nf.Meta.SmallestSeqNum == nf.Meta.LargestSeqNum {
			return "ingested"
		}
	}
	return "added"
}

func (l *lsmT) formatJSON(v interface{}) string {
	if l.pretty {
		return l.prettyJSON(v)
	}
	return l.uglyJSON(v)
}

func (l *lsmT) uglyJSON(v interface{}) string {
	data, err := json.Marshal(v)
	if err != nil {
		log.Fatal(err)
	}
	return string(data)
}

func (l *lsmT) prettyJSON(v interface{}) string {
	data, err := json.MarshalIndent(v, "", "\t")
	if err != nil {
		log.Fatal(err)
	}
	return string(data)
}
