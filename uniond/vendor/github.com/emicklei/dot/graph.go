package dot

import (
	"bytes"
	"fmt"
	"io"
	"sort"
	"strings"
)

// Graph represents a dot graph with nodes and edges.
type Graph struct {
	AttributesMap
	id        string
	isStrict  bool
	graphType string
	seq       int
	nodes     map[string]Node
	edgesFrom map[string][]Edge
	subgraphs map[string]*Graph
	parent    *Graph
	sameRank  map[string][]Node
	//
	nodeInitializer func(Node)
	edgeInitializer func(Edge)
}

// NewGraph return a new initialized Graph.
func NewGraph(options ...GraphOption) *Graph {
	graph := &Graph{
		AttributesMap: AttributesMap{attributes: map[string]interface{}{}},
		isStrict:      false,
		graphType:     Directed.Name,
		nodes:         map[string]Node{},
		edgesFrom:     map[string][]Edge{},
		subgraphs:     map[string]*Graph{},
		sameRank:      map[string][]Node{},
	}
	for _, each := range options {
		each.Apply(graph)
	}
	return graph
}

// GetID returns the identifier of the graph.
func (g *Graph) GetID() string {
	return g.id
}

// ID sets the identifier of the graph.
func (g *Graph) ID(newID string) *Graph {
	if len(g.id) > 0 {
		panic("cannot overwrite non-empty id ; both the old and the new could be in use and we cannot tell")
	}
	g.id = newID
	return g
}

// Label sets the "label" attribute value.
func (g *Graph) Label(label string) *Graph {
	g.AttributesMap.Attr("label", label)
	return g
}

func (g *Graph) beCluster() {
	g.id = "cluster_" + g.id
}

// Root returns the top-level graph if this was a subgraph.
func (g *Graph) Root() *Graph {
	if g.parent == nil {
		return g
	}
	return g.parent.Root()
}

func (g *Graph) FindNodeWithLabel(label string) (Node, bool) {
	for _, each := range g.nodes {
		if eachLabel, ok := each.attributes["label"]; ok {
			if eachLabel == label {
				return each, true
			}
		}
	}
	// TODO search subgraphs too?
	if g.parent == nil {
		return Node{id: "void"}, false
	}
	return g.parent.FindNodeWithLabel(label)

}

// FindSubgraph returns the subgraph of the graph or one from its parents.
func (g *Graph) FindSubgraph(id string) (*Graph, bool) {
	sub, ok := g.subgraphs[id]
	if !ok {
		if g.parent != nil {
			return g.parent.FindSubgraph(id)
		}
	}
	return sub, ok
}

// Subgraph returns the Graph with the given id ; creates one if absent.
// The label attribute is also set to the id ; use Label() to overwrite it.
func (g *Graph) Subgraph(id string, options ...GraphOption) *Graph {
	sub, ok := g.subgraphs[id]
	if ok {
		return sub
	}
	sub = NewGraph(Sub)
	sub.Attr("label", id) // for consistency with Node creation behavior.
	sub.id = fmt.Sprintf("s%d", g.nextSeq())
	for _, each := range options {
		each.Apply(sub)
	}
	sub.parent = g
	sub.edgeInitializer = g.edgeInitializer
	sub.nodeInitializer = g.nodeInitializer
	g.subgraphs[id] = sub
	return sub
}

func (g *Graph) findNode(id string) (Node, bool) {
	if n, ok := g.nodes[id]; ok {
		return n, ok
	}
	if g.parent == nil {
		return Node{id: "void"}, false
	}
	return g.parent.findNode(id)
}

// nextSeq takes the next sequence number from the root graph
func (g *Graph) nextSeq() int {
	root := g.Root()
	root.seq++
	return root.seq
}

// NodeInitializer sets a function that is called (if not nil) when a Node is implicitly created.
func (g *Graph) NodeInitializer(callback func(n Node)) {
	g.nodeInitializer = callback
}

// EdgeInitializer sets a function that is called (if not nil) when an Edge is implicitly created.
func (g *Graph) EdgeInitializer(callback func(e Edge)) {
	g.edgeInitializer = callback
}

// Node returns the node created with this id or creates a new node if absent.
// The node will have a label attribute with the id as its value. Use Label() to overwrite this.
// This method can be used as both a constructor and accessor.
// not thread safe!
func (g *Graph) Node(id string) Node {
	if n, ok := g.findNode(id); ok {
		return n
	}
	n := Node{
		id:  id,
		seq: g.nextSeq(), // create a new, use root sequence
		AttributesMap: AttributesMap{attributes: map[string]interface{}{
			"label": id}},
		graph: g,
	}
	if g.nodeInitializer != nil {
		g.nodeInitializer(n)
	}
	// store local
	g.nodes[id] = n
	return n
}

// DeleteNode deletes a node and all the edges associated to the node
// Returns false if the node wasn't found, true otherwise
func (g *Graph) DeleteNode(id string) bool {
	if _, ok := g.findNode(id); ok {
		// Remove Node
		delete(g.nodes, id)
		// Remove all the edges from the Node
		delete(g.edgesFrom, id)
		// Remove all the edges to the Node
		for parent, edgeList := range g.edgesFrom {
			for i, edge := range edgeList {
				if edge.to.id == id {
					g.edgesFrom[parent] = append(g.edgesFrom[parent][:i], g.edgesFrom[parent][i+1:]...)
					break
				}
			}
		}
		return true
	}
	return false
}

// Edge creates a new edge between two nodes.
// Nodes can have multiple edges to the same other node (or itself).
// If one or more labels are given then the "label" attribute is set to the edge.
func (g *Graph) Edge(fromNode, toNode Node, labels ...string) Edge {
	return g.EdgeWithPorts(fromNode, toNode, "", "", labels...)
}

// EdgeWithPorts creates a new edge between two nodes with ports.
// Other functionality are the same
func (g *Graph) EdgeWithPorts(fromNode, toNode Node, fromNodePort, toNodePort string, labels ...string) Edge {
	// assume fromNode owner == toNode owner
	edgeOwner := g
	if fromNode.graph != toNode.graph { // 1 or 2 are subgraphs
		edgeOwner = commonParentOf(fromNode.graph, toNode.graph)
	}
	e := Edge{
		from:          fromNode,
		to:            toNode,
		AttributesMap: AttributesMap{attributes: map[string]interface{}{}},
		graph:         edgeOwner}
	if fromNodePort != "" {
		e.fromPort = fromNodePort
	}
	if toNodePort != "" {
		e.toPort = toNodePort
	}
	if len(labels) > 0 {
		e.Attr("label", strings.Join(labels, ","))
	}
	if g.edgeInitializer != nil {
		g.edgeInitializer(e)
	}
	edgeOwner.edgesFrom[fromNode.id] = append(edgeOwner.edgesFrom[fromNode.id], e)
	return e
}

// FindEdges finds all edges in the graph that go from the fromNode to the toNode.
// Otherwise, returns an empty slice.
func (g *Graph) FindEdges(fromNode, toNode Node) (found []Edge) {
	found = make([]Edge, 0)
	edgeOwner := g
	if fromNode.graph != toNode.graph {
		edgeOwner = commonParentOf(fromNode.graph, toNode.graph)
	}
	if edges, ok := edgeOwner.edgesFrom[fromNode.id]; ok {
		for _, e := range edges {
			if e.to.id == toNode.id {
				found = append(found, e)
			}
		}
	}
	return found
}

func commonParentOf(one *Graph, two *Graph) *Graph {
	// TODO
	return one.Root()
}

// AddToSameRank adds the given nodes to the specified rank group, forcing them to be rendered in the same row
func (g *Graph) AddToSameRank(group string, nodes ...Node) {
	g.sameRank[group] = append(g.sameRank[group], nodes...)
}

// String returns the source in dot notation.
func (g *Graph) String() string {
	b := new(bytes.Buffer)
	g.Write(b)
	return b.String()
}

func (g *Graph) Write(w io.Writer) {
	g.IndentedWrite(NewIndentWriter(w))
}

// IndentedWrite write the graph to a writer using simple TAB indentation.
func (g *Graph) IndentedWrite(w *IndentWriter) {
	if g.isStrict && g.graphType != Sub.Name {
		fmt.Fprintf(w, "strict ")
	}
	fmt.Fprintf(w, "%s %s {", g.graphType, g.id)
	w.NewLineIndentWhile(func() {
		// subgraphs
		for _, key := range g.sortedSubgraphsKeys() {
			each := g.subgraphs[key]
			each.IndentedWrite(w)
		}
		// graph attributes
		appendSortedMap(g.AttributesMap.attributes, false, w)
		w.NewLine()
		// graph nodes
		for _, key := range g.sortedNodesKeys() {
			each := g.nodes[key]
			fmt.Fprintf(w, "n%d", each.seq)
			appendSortedMap(each.attributes, true, w)
			fmt.Fprintf(w, ";")
			w.NewLine()
		}
		// graph edges
		denoteEdge := "->"
		if g.graphType == "graph" {
			denoteEdge = "--"
		}
		for _, each := range g.sortedEdgesFromKeys() {
			all := g.edgesFrom[each]
			for _, each := range all {
				fromPort := ""
				if each.fromPort != "" {
					fromPort = ":" + each.fromPort
				}
				toPort := ""
				if each.toPort != "" {
					toPort = ":" + each.toPort
				}
				fmt.Fprintf(w, "n%d%s%sn%d%s", each.from.seq, fromPort, denoteEdge, each.to.seq, toPort)
				appendSortedMap(each.attributes, true, w)
				fmt.Fprint(w, ";")
				w.NewLine()
			}
		}
		for _, nodes := range g.sameRank {
			str := ""
			for _, n := range nodes {
				str += fmt.Sprintf("n%d;", n.seq)
			}
			fmt.Fprintf(w, "{rank=same; %s};", str)
			w.NewLine()
		}
	})
	fmt.Fprintf(w, "}")
	w.NewLine()
}

func appendSortedMap(m map[string]interface{}, mustBracket bool, b io.Writer) {
	if len(m) == 0 {
		return
	}
	if mustBracket {
		fmt.Fprint(b, "[")
	}
	first := true
	// first collect keys
	keys := []string{}
	for k := range m {
		keys = append(keys, k)
	}
	sort.StringSlice(keys).Sort()

	for _, k := range keys {
		if !first {
			if mustBracket {
				fmt.Fprint(b, ",")
			} else {
				fmt.Fprintf(b, ";")
			}
		}
		if html, isHTML := m[k].(HTML); isHTML {
			fmt.Fprintf(b, "%s=<%s>", k, html)
		} else if literal, isLiteral := m[k].(Literal); isLiteral {
			fmt.Fprintf(b, "%s=%s", k, literal)
		} else if str, ok := m[k].(string); ok {
			fmt.Fprintf(b, "%s=%q", k, str)
		} else {
			fmt.Fprintf(b, "%s=\"%v\"", k, m[k])
		}
		first = false
	}
	if mustBracket {
		fmt.Fprint(b, "]")
	} else {
		fmt.Fprint(b, ";")
	}
}

// VisitNodes visits all nodes recursively
func (g *Graph) VisitNodes(callback func(node Node) (done bool)) {
	for _, node := range g.nodes {
		done := callback(node)
		if done {
			return
		}
	}

	for _, subGraph := range g.subgraphs {
		subGraph.VisitNodes(callback)
	}
}

// FindNodeById return node by id
func (g *Graph) FindNodeById(id string) (foundNode Node, found bool) {
	g.VisitNodes(func(node Node) (done bool) {
		if node.id == id {
			found = true
			foundNode = node
			return true
		}
		return false
	})
	return
}

// FindNodes returns all nodes recursively
func (g *Graph) FindNodes() (nodes []Node) {
	var foundNodes []Node
	g.VisitNodes(func(node Node) (done bool) {
		foundNodes = append(foundNodes, node)
		return false
	})
	return foundNodes
}

// IsDirected returns info about the graph type
func (g *Graph) IsDirected() bool {
	return g.graphType == Directed.Name
}

// EdgesMap returns a map with Node.id -> []Edge
func (g *Graph) EdgesMap() map[string][]Edge {
	return g.edgesFrom
}

// HasNode returns whether the node was created in this graph (does not look for it in subgraphs).
func (g *Graph) HasNode(n Node) bool {
	return g == n.graph
}
