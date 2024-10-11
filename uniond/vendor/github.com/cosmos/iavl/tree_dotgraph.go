package iavl

import (
	"bufio"
	"bytes"
	"fmt"
	"io"
	"os"
	"text/template"

	"github.com/emicklei/dot"
)

type graphEdge struct {
	From, To string
}

type graphNode struct {
	Hash  string
	Label string
	Value string
	Attrs map[string]string
}

type graphContext struct {
	Edges []*graphEdge
	Nodes []*graphNode
}

var graphTemplate = `
strict graph {
	{{- range $i, $edge := $.Edges}}
	"{{ $edge.From }}" -- "{{ $edge.To }}";
	{{- end}}

	{{range $i, $node := $.Nodes}}
	"{{ $node.Hash }}" [label=<{{ $node.Label }}>,{{ range $k, $v := $node.Attrs }}{{ $k }}={{ $v }},{{end}}];
	{{- end}}
}
`

var tpl = template.Must(template.New("iavl").Parse(graphTemplate))

var defaultGraphNodeAttrs = map[string]string{
	"shape": "circle",
}

func WriteDOTGraph(w io.Writer, tree *ImmutableTree, paths []PathToLeaf) {
	ctx := &graphContext{}

	tree.root.hashWithCount(tree.version + 1)
	tree.root.traverse(tree, true, func(node *Node) bool {
		graphNode := &graphNode{
			Attrs: map[string]string{},
			Hash:  fmt.Sprintf("%x", node.hash),
		}
		for k, v := range defaultGraphNodeAttrs {
			graphNode.Attrs[k] = v
		}
		shortHash := graphNode.Hash[:7]

		graphNode.Label = mkLabel(string(node.key), 16, "sans-serif")
		graphNode.Label += mkLabel(shortHash, 10, "monospace")
		graphNode.Label += mkLabel(fmt.Sprintf("nodeKey=%v", node.nodeKey), 10, "monospace")

		if node.value != nil {
			graphNode.Label += mkLabel(string(node.value), 10, "sans-serif")
		}

		if node.subtreeHeight == 0 {
			graphNode.Attrs["fillcolor"] = "lightgrey"
			graphNode.Attrs["style"] = "filled"
		}

		for _, path := range paths {
			for _, n := range path {
				if bytes.Equal(n.Left, node.hash) || bytes.Equal(n.Right, node.hash) {
					graphNode.Attrs["peripheries"] = "2"
					graphNode.Attrs["style"] = "filled"
					graphNode.Attrs["fillcolor"] = "lightblue"
					break
				}
			}
		}
		ctx.Nodes = append(ctx.Nodes, graphNode)

		if node.leftNode != nil {
			ctx.Edges = append(ctx.Edges, &graphEdge{
				From: graphNode.Hash,
				To:   fmt.Sprintf("%x", node.leftNode.hash),
			})
		}
		if node.rightNode != nil {
			ctx.Edges = append(ctx.Edges, &graphEdge{
				From: graphNode.Hash,
				To:   fmt.Sprintf("%x", node.rightNode.hash),
			})
		}
		return false
	})

	if err := tpl.Execute(w, ctx); err != nil {
		panic(err)
	}
}

func mkLabel(label string, pt int, face string) string {
	return fmt.Sprintf("<font face='%s' point-size='%d'>%s</font><br />", face, pt, label)
}

// WriteDOTGraphToFile writes the DOT graph to the given filename. Read like:
// $ dot /tmp/tree_one.dot -Tpng | display
func WriteDOTGraphToFile(filename string, tree *ImmutableTree) {
	f1, _ := os.Create(filename)
	defer f1.Close()
	writer := bufio.NewWriter(f1)
	WriteDotGraphv2(writer, tree)
	err := writer.Flush()
	if err != nil {
		panic(err)
	}
}

// WriteDotGraphv2 writes a DOT graph to the given writer. WriteDOTGraph failed to produce valid DOT
// graphs for large trees. This function is a rewrite of WriteDOTGraph that produces valid DOT graphs
func WriteDotGraphv2(w io.Writer, tree *ImmutableTree) {
	graph := dot.NewGraph(dot.Directed)

	var traverse func(node *Node, parent *dot.Node, direction string)
	traverse = func(node *Node, parent *dot.Node, direction string) {
		var label string
		if node.isLeaf() {
			label = fmt.Sprintf("%v:%v\nv%v", node.key, node.value, node.nodeKey.version)
		} else {
			label = fmt.Sprintf("%v:%v\nv%v", node.subtreeHeight, node.key, node.nodeKey.version)
		}

		n := graph.Node(label)
		if parent != nil {
			parent.Edge(n, direction)
		}

		var leftNode, rightNode *Node

		if node.leftNode != nil {
			leftNode = node.leftNode
		} else if node.leftNodeKey != nil {
			in, err := node.getLeftNode(tree)
			if err == nil {
				leftNode = in
			}
		}

		if node.rightNode != nil {
			rightNode = node.rightNode
		} else if node.rightNodeKey != nil {
			in, err := node.getRightNode(tree)
			if err == nil {
				rightNode = in
			}
		}

		if leftNode != nil {
			traverse(leftNode, &n, "l")
		}
		if rightNode != nil {
			traverse(rightNode, &n, "r")
		}
	}

	traverse(tree.root, nil, "")
	_, err := w.Write([]byte(graph.String()))
	if err != nil {
		panic(err)
	}
}
