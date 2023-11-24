package iavl

import (
	"bytes"

	"github.com/cosmos/iavl/proto"
)

type (
	KVPair    = proto.KVPair
	ChangeSet = proto.ChangeSet
)

// KVPairReceiver is callback parameter of method `extractStateChanges` to receive stream of `KVPair`s.
type KVPairReceiver func(pair *KVPair) error

// extractStateChanges extracts the state changes by between two versions of the tree.
// it first traverse the `root` tree until the first `sharedNode` and record the new leave nodes,
// then traverse the `prevRoot` tree until the current `sharedNode` to find out orphaned leave nodes,
// compare orphaned leave nodes and new leave nodes to produce stream of `KVPair`s and passed to callback.
//
// The algorithm don't run in constant memory strictly, but it tried the best the only
// keep minimal intermediate states in memory.
func (ndb *nodeDB) extractStateChanges(prevVersion int64, prevRoot, root []byte, receiver KVPairReceiver) error {
	curIter, err := NewNodeIterator(root, ndb)
	if err != nil {
		return err
	}

	prevIter, err := NewNodeIterator(prevRoot, ndb)
	if err != nil {
		return err
	}

	var (
		// current shared node between two versions
		sharedNode *Node
		// record the newly added leaf nodes during the traversal to the `sharedNode`,
		// will be compared with found orphaned nodes to produce change set stream.
		newLeaves []*Node
	)

	// consumeNewLeaves concumes remaining `newLeaves` nodes and produce insertion `KVPair`.
	consumeNewLeaves := func() error {
		for _, node := range newLeaves {
			if err := receiver(&KVPair{
				Key:   node.key,
				Value: node.value,
			}); err != nil {
				return err
			}
		}

		newLeaves = newLeaves[:0]
		return nil
	}

	// advanceSharedNode forward `curIter` until the next `sharedNode`,
	// `sharedNode` will be `nil` if the new version is exhausted.
	// it also records the new leaf nodes during the traversal.
	advanceSharedNode := func() error {
		if err := consumeNewLeaves(); err != nil {
			return err
		}

		sharedNode = nil
		for curIter.Valid() {
			node := curIter.GetNode()
			shared := node.nodeKey.version <= prevVersion
			curIter.Next(shared)
			if shared {
				sharedNode = node
				break
			} else if node.isLeaf() {
				newLeaves = append(newLeaves, node)
			}
		}

		return nil
	}
	if err := advanceSharedNode(); err != nil {
		return err
	}

	// addOrphanedLeave receives a new orphaned leave node found in previous version,
	// compare with the current newLeaves, to produce `iavl.KVPair` stream.
	addOrphanedLeave := func(orphaned *Node) error {
		for len(newLeaves) > 0 {
			newLeave := newLeaves[0]
			switch bytes.Compare(orphaned.key, newLeave.key) {
			case 1:
				// consume a new node as insertion and continue
				newLeaves = newLeaves[1:]
				if err := receiver(&KVPair{
					Key:   newLeave.key,
					Value: newLeave.value,
				}); err != nil {
					return err
				}
				continue

			case -1:
				// removal, don't consume new nodes
				return receiver(&KVPair{
					Delete: true,
					Key:    orphaned.key,
				})

			case 0:
				// update, consume the new node and stop
				newLeaves = newLeaves[1:]
				return receiver(&KVPair{
					Key:   newLeave.key,
					Value: newLeave.value,
				})
			}
		}

		// removal
		return receiver(&KVPair{
			Delete: true,
			Key:    orphaned.key,
		})
	}

	// Traverse `prevIter` to find orphaned nodes in the previous version,
	// and compare them with newLeaves to generate `KVPair` stream.
	for prevIter.Valid() {
		node := prevIter.GetNode()
		shared := sharedNode != nil && (node == sharedNode || bytes.Equal(node.hash, sharedNode.hash))
		// skip sub-tree of shared nodes
		prevIter.Next(shared)
		if shared {
			if err := advanceSharedNode(); err != nil {
				return err
			}
		} else if node.isLeaf() {
			if err := addOrphanedLeave(node); err != nil {
				return err
			}
		}
	}

	if err := consumeNewLeaves(); err != nil {
		return err
	}

	if err := curIter.Error(); err != nil {
		return err
	}
	return prevIter.Error()
}
