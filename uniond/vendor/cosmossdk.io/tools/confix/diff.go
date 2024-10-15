package confix

import (
	"fmt"
	"io"
	"sort"

	"github.com/creachadair/tomledit"
	"github.com/creachadair/tomledit/parser"
	"github.com/creachadair/tomledit/transform"
)

type DiffType string

const (
	Section DiffType = "S"
	Mapping DiffType = "M"
)

type KV struct {
	Key   string
	Value string
	Block []string // comment block
}

type Diff struct {
	Type    DiffType
	Deleted bool

	KV KV
}

// DiffKeys diffs the keyspaces of the TOML documents in files lhs and rhs.
// Comments, order, and values are ignored for comparison purposes.
func DiffKeys(lhs, rhs *tomledit.Document) []Diff {
	// diff sections
	diff := diffDocs(allKVs(lhs.Global), allKVs(rhs.Global), false)

	lsec, rsec := lhs.Sections, rhs.Sections
	transform.SortSectionsByName(lsec)
	transform.SortSectionsByName(rsec)

	i, j := 0, 0
	for i < len(lsec) && j < len(rsec) {
		switch {
		case lsec[i].Name.Before(rsec[j].Name):
			diff = append(diff, Diff{Type: Section, Deleted: true, KV: KV{Key: lsec[i].Name.String()}})
			for _, kv := range allKVs(lsec[i]) {
				diff = append(diff, Diff{Type: Mapping, Deleted: true, KV: kv})
			}
			i++
		case rsec[j].Name.Before(lsec[i].Name):
			diff = append(diff, Diff{Type: Section, KV: KV{Key: rsec[j].Name.String()}})
			for _, kv := range allKVs(rsec[j]) {
				diff = append(diff, Diff{Type: Mapping, KV: kv})
			}
			j++
		default:
			diff = append(diff, diffDocs(allKVs(lsec[i]), allKVs(rsec[j]), false)...)
			i++
			j++
		}
	}
	for ; i < len(lsec); i++ {
		diff = append(diff, Diff{Type: Section, Deleted: true, KV: KV{Key: lsec[i].Name.String()}})
		for _, kv := range allKVs(lsec[i]) {
			diff = append(diff, Diff{Type: Mapping, Deleted: true, KV: kv})
		}
	}
	for ; j < len(rsec); j++ {
		diff = append(diff, Diff{Type: Section, KV: KV{Key: rsec[j].Name.String()}})
		for _, kv := range allKVs(rsec[j]) {
			diff = append(diff, Diff{Type: Mapping, KV: kv})
		}
	}

	return diff
}

// DiffValues diffs the keyspaces with different values of the TOML documents in files lhs and rhs.
func DiffValues(lhs, rhs *tomledit.Document) []Diff {
	diff := diffDocs(allKVs(lhs.Global), allKVs(rhs.Global), true)

	lsec, rsec := lhs.Sections, rhs.Sections
	transform.SortSectionsByName(lsec)
	transform.SortSectionsByName(rsec)

	i, j := 0, 0
	for i < len(lsec) && j < len(rsec) {
		switch {
		case lsec[i].Name.Before(rsec[j].Name):
			// skip keys present in lhs but not in rhs
			i++
		case rsec[j].Name.Before(lsec[i].Name):
			// skip keys present in rhs but not in lhs
			j++
		default:
			for _, d := range diffDocs(allKVs(lsec[i]), allKVs(rsec[j]), true) {
				if !d.Deleted {
					diff = append(diff, d)
				}
			}
			i++
			j++
		}
	}

	return diff
}

func allKVs(s *tomledit.Section) []KV {
	keys := []KV{}
	s.Scan(func(key parser.Key, entry *tomledit.Entry) bool {
		keys = append(keys, KV{
			Key: key.String(),
			// we get the value of the current configuration (i.e the one we want to compare/migrate)
			Value: entry.Value.String(),
			Block: entry.Block,
		})

		return true
	})
	return keys
}

// diffDocs get the diff between all keys in lhs and rhs.
// when a key is in both lhs and rhs, it is ignored, unless value is true in which case the value is as well compared.
func diffDocs(lhs, rhs []KV, value bool) []Diff {
	diff := []Diff{}

	sort.Slice(lhs, func(i, j int) bool {
		return lhs[i].Key < lhs[j].Key
	})
	sort.Slice(rhs, func(i, j int) bool {
		return rhs[i].Key < rhs[j].Key
	})

	i, j := 0, 0
	for i < len(lhs) && j < len(rhs) {
		switch {
		case lhs[i].Key < rhs[j].Key:
			diff = append(diff, Diff{Type: Mapping, Deleted: true, KV: lhs[i]})
			i++
		case lhs[i].Key > rhs[j].Key:
			diff = append(diff, Diff{Type: Mapping, KV: rhs[j]})
			j++
		default:
			// key exists in both lhs and rhs
			// if value is true, compare the values
			if value && lhs[i].Value != rhs[j].Value {
				diff = append(diff, Diff{Type: Mapping, KV: lhs[i]})
			}
			i++
			j++
		}
	}
	for ; i < len(lhs); i++ {
		diff = append(diff, Diff{Type: Mapping, Deleted: true, KV: lhs[i]})
	}
	for ; j < len(rhs); j++ {
		diff = append(diff, Diff{Type: Mapping, KV: rhs[j]})
	}

	return diff
}

// PrintDiff output prints one line per key that differs:
// -S name    -- section exists in f1 but not f2
// +S name    -- section exists in f2 but not f1
// -M name    -- mapping exists in f1 but not f2
// +M name    -- mapping exists in f2 but not f1
func PrintDiff(w io.Writer, diffs []Diff) {
	for _, diff := range diffs {
		if diff.Deleted {
			fmt.Fprintln(w, fmt.Sprintf("-%s", diff.Type), fmt.Sprintf("%s=%s", diff.KV.Key, diff.KV.Value))
		} else {
			fmt.Fprintln(w, fmt.Sprintf("+%s", diff.Type), fmt.Sprintf("%s=%s", diff.KV.Key, diff.KV.Value))
		}
	}
}
