package confix

import (
	"sort"

	"github.com/creachadair/tomledit"
	"github.com/creachadair/tomledit/transform"
)

// MatchKeys diffs the keyspaces of the TOML documents in files lhs and rhs.
// Comments, order, and values are ignored for comparison purposes.
// It will return in the format of map[oldKey]newKey
func MatchKeys(lhs, rhs *tomledit.Document) map[string]string {
	matches := matchDocs(map[string]string{}, allKVs(lhs.Global), allKVs(rhs.Global))

	lsec, rsec := lhs.Sections, rhs.Sections
	transform.SortSectionsByName(lsec)
	transform.SortSectionsByName(rsec)

	i, j := 0, 0
	for i < len(lsec) && j < len(rsec) {
		switch {
		case lsec[i].Name.Before(rsec[j].Name):
			i++
		case rsec[j].Name.Before(lsec[i].Name):
			j++
		default:
			matches = matchDocs(matches, allKVs(lsec[i]), allKVs(rsec[j]))
			i++
			j++
		}
	}

	return matches
}

// matchDocs get all the keys matching in lhs and rhs.
// value of keys are ignored
func matchDocs(matchesMap map[string]string, lhs, rhs []KV) map[string]string {
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
			i++
		case lhs[i].Key > rhs[j].Key:
			j++
		default:
			// key exists in both lhs and rhs
			matchesMap[lhs[i].Key] = rhs[j].Key
			i++
			j++
		}
	}

	return matchesMap
}
