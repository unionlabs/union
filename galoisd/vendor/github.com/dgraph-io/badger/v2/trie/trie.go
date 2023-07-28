/*
 * Copyright 2019 Dgraph Labs, Inc. and Contributors
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

package trie

type node struct {
	children map[byte]*node
	ids      []uint64
}

func newNode() *node {
	return &node{
		children: make(map[byte]*node),
		ids:      []uint64{},
	}
}

// Trie datastructure.
type Trie struct {
	root *node
}

// NewTrie returns Trie.
func NewTrie() *Trie {
	return &Trie{
		root: newNode(),
	}
}

// Add adds the id in the trie for the given prefix path.
func (t *Trie) Add(prefix []byte, id uint64) {
	node := t.root
	for _, val := range prefix {
		child, ok := node.children[val]
		if !ok {
			child = newNode()
			node.children[val] = child
		}
		node = child
	}
	// We only need to add the id to the last node of the given prefix.
	node.ids = append(node.ids, id)
}

// Get returns prefix matched ids for the given key.
func (t *Trie) Get(key []byte) map[uint64]struct{} {
	out := make(map[uint64]struct{})
	node := t.root
	// If root has ids that means we have subscribers for "nil/[]byte{}"
	// prefix. Add them to the list.
	if len(node.ids) > 0 {
		for _, i := range node.ids {
			out[i] = struct{}{}
		}
	}
	for _, val := range key {
		child, ok := node.children[val]
		if !ok {
			break
		}
		// We need ids of the all the node in the matching key path.
		for _, id := range child.ids {
			out[id] = struct{}{}
		}
		node = child
	}
	return out
}

// Delete will delete the id if the id exist in the given index path.
func (t *Trie) Delete(index []byte, id uint64) {
	node := t.root
	for _, val := range index {
		child, ok := node.children[val]
		if !ok {
			return
		}
		node = child
	}
	// We're just removing the id not the hanging path.
	out := node.ids[:0]
	for _, val := range node.ids {
		if val != id {
			out = append(out, val)
		}
	}
	for i := len(out); i < len(node.ids); i++ {
		node.ids[i] = 0 // garbage collecting
	}
	node.ids = out
}
