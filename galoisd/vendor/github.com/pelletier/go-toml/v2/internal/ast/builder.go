package ast

type Reference int

const InvalidReference Reference = -1

func (r Reference) Valid() bool {
	return r != InvalidReference
}

type Builder struct {
	tree    Root
	lastIdx int
}

func (b *Builder) Tree() *Root {
	return &b.tree
}

func (b *Builder) NodeAt(ref Reference) *Node {
	return b.tree.at(ref)
}

func (b *Builder) Reset() {
	b.tree.nodes = b.tree.nodes[:0]
	b.lastIdx = 0
}

func (b *Builder) Push(n Node) Reference {
	b.lastIdx = len(b.tree.nodes)
	b.tree.nodes = append(b.tree.nodes, n)
	return Reference(b.lastIdx)
}

func (b *Builder) PushAndChain(n Node) Reference {
	newIdx := len(b.tree.nodes)
	b.tree.nodes = append(b.tree.nodes, n)
	if b.lastIdx >= 0 {
		b.tree.nodes[b.lastIdx].next = newIdx - b.lastIdx
	}
	b.lastIdx = newIdx
	return Reference(b.lastIdx)
}

func (b *Builder) AttachChild(parent Reference, child Reference) {
	b.tree.nodes[parent].child = int(child) - int(parent)
}

func (b *Builder) Chain(from Reference, to Reference) {
	b.tree.nodes[from].next = int(to) - int(from)
}
