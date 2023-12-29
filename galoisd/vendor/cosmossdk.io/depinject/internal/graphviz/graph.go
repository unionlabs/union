// Package graphviz
package graphviz

import (
	"bytes"
	"fmt"
	"io"

	"cosmossdk.io/depinject/internal/util"
)

// Graph represents a graphviz digraph.
type Graph struct {
	*Attributes

	// name is the optional name of this graph
	name string

	// parent is non-nil if this is a sub-graph
	parent *Graph

	// allNodes includes all nodes in the graph and its sub-graphs.
	// It is set to the same map in parent and sub-graphs.
	allNodes map[string]*Node

	// myNodes are the nodes in this graph (whether it's a root or sub-graph)
	myNodes map[string]*Node

	subgraphs map[string]*Graph

	edges []*Edge
}

// NewGraph creates a new Graph instance.
func NewGraph() *Graph {
	return &Graph{
		Attributes: NewAttributes(),
		name:       "",
		parent:     nil,
		allNodes:   map[string]*Node{},
		myNodes:    map[string]*Node{},
		subgraphs:  map[string]*Graph{},
		edges:      nil,
	}
}

// FindOrCreateNode finds or creates the node with the provided name.
func (g *Graph) FindOrCreateNode(name string) (node *Node, found bool) {
	if node, ok := g.allNodes[name]; ok {
		return node, true
	}

	node = &Node{
		Attributes: NewAttributes(),
		name:       name,
	}
	g.allNodes[name] = node
	g.myNodes[name] = node
	return node, false
}

// FindOrCreateSubGraph finds or creates the subgraph with the provided name.
func (g *Graph) FindOrCreateSubGraph(name string) (graph *Graph, found bool) {
	if sub, ok := g.subgraphs[name]; ok {
		return sub, true
	}

	n := &Graph{
		Attributes: NewAttributes(),
		name:       name,
		parent:     g,
		allNodes:   g.allNodes,
		myNodes:    map[string]*Node{},
		subgraphs:  map[string]*Graph{},
		edges:      nil,
	}
	g.subgraphs[name] = n
	return n, false
}

// CreateEdge creates a new graphviz edge.
func (g *Graph) CreateEdge(from, to *Node) *Edge {
	edge := &Edge{
		Attributes: NewAttributes(),
		from:       from,
		to:         to,
	}
	g.edges = append(g.edges, edge)
	return edge
}

// RenderDOT renders the graph to DOT format.
func (g *Graph) RenderDOT(w io.Writer) error {
	return g.render(w, "")
}

func (g *Graph) render(w io.Writer, indent string) error {
	if g.parent == nil {
		_, err := fmt.Fprintf(w, "%sdigraph %q {\n", indent, g.name)
		if err != nil {
			return err
		}
	} else {
		_, err := fmt.Fprintf(w, "%ssubgraph %q {\n", indent, g.name)
		if err != nil {
			return err
		}
	}

	{
		subIndent := indent + "  "

		if attrStr := g.Attributes.String(); attrStr != "" {
			_, err := fmt.Fprintf(w, "%sgraph %s;\n", subIndent, attrStr)
			if err != nil {
				return err
			}
		}

		// we do map iteration in sorted order so that outputs are stable and
		// can be used in tests
		err := util.IterateMapOrdered(g.subgraphs, func(_ string, subgraph *Graph) error {
			return subgraph.render(w, subIndent+"  ")
		})
		if err != nil {
			return err
		}

		err = util.IterateMapOrdered(g.myNodes, func(_ string, node *Node) error {
			return node.render(w, subIndent)
		})
		if err != nil {
			return err
		}

		for _, edge := range g.edges {
			err := edge.render(w, subIndent)
			if err != nil {
				return err
			}
		}
	}

	_, err := fmt.Fprintf(w, "%s}\n\n", indent)
	return err
}

// String returns the graph in DOT format.
func (g *Graph) String() string {
	buf := &bytes.Buffer{}
	err := g.RenderDOT(buf)
	if err != nil {
		panic(err)
	}
	return buf.String()
}
