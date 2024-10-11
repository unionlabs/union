package doublylinkedtree

import (
	"github.com/prysmaticlabs/prysm/v4/consensus-types/primitives"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

// LastRoot returns the last canonical block root in the given epoch
func (f *ForkChoice) LastRoot(epoch primitives.Epoch) [32]byte {
	head := f.store.headNode
	headEpoch := slots.ToEpoch(head.slot)
	epochEnd, err := slots.EpochEnd(epoch)
	if err != nil {
		return [32]byte{}
	}
	if headEpoch <= epoch {
		return head.root
	}
	for head != nil && head.slot > epochEnd {
		head = head.parent
	}
	if head == nil {
		return [32]byte{}
	}
	return head.root
}
