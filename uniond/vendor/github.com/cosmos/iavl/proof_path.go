package iavl

import (
	"fmt"
	"strings"
)

//----------------------------------------

// PathToLeaf represents an inner path to a leaf node.
// Note that the nodes are ordered such that the last one is closest
// to the root of the tree.
type PathToLeaf []ProofInnerNode

func (pl PathToLeaf) String() string {
	return pl.stringIndented("")
}

func (pl PathToLeaf) stringIndented(indent string) string {
	if len(pl) == 0 {
		return "empty-PathToLeaf"
	}
	strs := make([]string, 0, len(pl))
	for i, pin := range pl {
		if i == 20 {
			strs = append(strs, fmt.Sprintf("... (%v total)", len(pl)))
			break
		}
		strs = append(strs, fmt.Sprintf("%v:%v", i, pin.stringIndented(indent+"  ")))
	}
	return fmt.Sprintf(`PathToLeaf{
%s  %v
%s}`,
		indent, strings.Join(strs, "\n"+indent+"  "),
		indent)
}

// returns -1 if invalid.
func (pl PathToLeaf) Index() (idx int64) {
	for i, node := range pl {
		switch {
		case node.Left == nil:
			continue
		case node.Right == nil:
			if i < len(pl)-1 {
				idx += node.Size - pl[i+1].Size
			} else {
				idx += node.Size - 1
			}
		default:
			return -1
		}
	}
	return idx
}
