package lv

import "sync"

// NewSpace returns an N-dimensional vector space.
func NewSpace() *Space {
	return &Space{}
}

// Space represents an N-dimensional vector space. Each name and unique label
// value pair establishes a new dimension and point within that dimension. Order
// matters, i.e. [a=1 b=2] identifies a different timeseries than [b=2 a=1].
type Space struct {
	mtx   sync.RWMutex
	nodes map[string]*node
}

// Observe locates the time series identified by the name and label values in
// the vector space, and appends the value to the list of observations.
func (s *Space) Observe(name string, lvs LabelValues, value float64) {
	s.nodeFor(name).observe(lvs, value)
}

// Add locates the time series identified by the name and label values in
// the vector space, and appends the delta to the last value in the list of
// observations.
func (s *Space) Add(name string, lvs LabelValues, delta float64) {
	s.nodeFor(name).add(lvs, delta)
}

// Walk traverses the vector space and invokes fn for each non-empty time series
// which is encountered. Return false to abort the traversal.
func (s *Space) Walk(fn func(name string, lvs LabelValues, observations []float64) bool) {
	s.mtx.RLock()
	defer s.mtx.RUnlock()
	for name, node := range s.nodes {
		f := func(lvs LabelValues, observations []float64) bool { return fn(name, lvs, observations) }
		if !node.walk(LabelValues{}, f) {
			return
		}
	}
}

// Reset empties the current space and returns a new Space with the old
// contents. Reset a Space to get an immutable copy suitable for walking.
func (s *Space) Reset() *Space {
	s.mtx.Lock()
	defer s.mtx.Unlock()
	n := NewSpace()
	n.nodes, s.nodes = s.nodes, n.nodes
	return n
}

func (s *Space) nodeFor(name string) *node {
	s.mtx.Lock()
	defer s.mtx.Unlock()
	if s.nodes == nil {
		s.nodes = map[string]*node{}
	}
	n, ok := s.nodes[name]
	if !ok {
		n = &node{}
		s.nodes[name] = n
	}
	return n
}

// node exists at a specific point in the N-dimensional vector space of all
// possible label values. The node collects observations and has child nodes
// with greater specificity.
type node struct {
	mtx          sync.RWMutex
	observations []float64
	children     map[pair]*node
}

type pair struct{ label, value string }

func (n *node) observe(lvs LabelValues, value float64) {
	n.mtx.Lock()
	defer n.mtx.Unlock()
	if len(lvs) <= 0 {
		n.observations = append(n.observations, value)
		return
	}
	if len(lvs) < 2 {
		panic("too few LabelValues; programmer error!")
	}
	head, tail := pair{lvs[0], lvs[1]}, lvs[2:]
	if n.children == nil {
		n.children = map[pair]*node{}
	}
	child, ok := n.children[head]
	if !ok {
		child = &node{}
		n.children[head] = child
	}
	child.observe(tail, value)
}

func (n *node) add(lvs LabelValues, delta float64) {
	n.mtx.Lock()
	defer n.mtx.Unlock()
	if len(lvs) <= 0 {
		var value float64
		if len(n.observations) > 0 {
			value = last(n.observations) + delta
		} else {
			value = delta
		}
		n.observations = append(n.observations, value)
		return
	}
	if len(lvs) < 2 {
		panic("too few LabelValues; programmer error!")
	}
	head, tail := pair{lvs[0], lvs[1]}, lvs[2:]
	if n.children == nil {
		n.children = map[pair]*node{}
	}
	child, ok := n.children[head]
	if !ok {
		child = &node{}
		n.children[head] = child
	}
	child.add(tail, delta)
}

func (n *node) walk(lvs LabelValues, fn func(LabelValues, []float64) bool) bool {
	n.mtx.RLock()
	defer n.mtx.RUnlock()
	if len(n.observations) > 0 && !fn(lvs, n.observations) {
		return false
	}
	for p, child := range n.children {
		if !child.walk(append(lvs, p.label, p.value), fn) {
			return false
		}
	}
	return true
}

func last(a []float64) float64 {
	return a[len(a)-1]
}
