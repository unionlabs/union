## dot - little helper package in Go for the graphviz dot language

[![Build Status](https://travis-ci.org/emicklei/proto.png)](https://travis-ci.org/emicklei/dot)
[![Go Report Card](https://goreportcard.com/badge/github.com/emicklei/dot)](https://goreportcard.com/report/github.com/emicklei/dot)
[![GoDoc](https://pkg.go.dev/badge/github.com/emicklei/dot)](https://pkg.go.dev/github.com/emicklei/dot)
[![codecov](https://codecov.io/gh/emicklei/dot/branch/master/graph/badge.svg)](https://codecov.io/gh/emicklei/dot)

[DOT language](http://www.graphviz.org/doc/info/lang.html)

	package main
	
	import (
		"fmt"	
		"github.com/emicklei/dot"
	)
	
	// go run main.go | dot -Tpng  > test.png && open test.png
	
	func main() {
		g := dot.NewGraph(dot.Directed)
		n1 := g.Node("coding")
		n2 := g.Node("testing a little").Box()
	
		g.Edge(n1, n2)
		g.Edge(n2, n1, "back").Attr("color", "red")
	
		fmt.Println(g.String())
	}

Output

	digraph {
		node [label="coding"]; n1;
		node [label="testing a little",shape="box"]; n2;
		n1 -> n2;
		n2 -> n1 [color="red", label="back"];
	}

Chaining edges

	g.Node("A").Edge(g.Node("B")).Edge(g.Node("C"))
	
	A -> B -> C

	g.Node("D").BidirectionalEdge(g.Node("E"))

	D <-> E

Subgraphs

	s := g.Subgraph("cluster")
	s.Attr("style", "filled")


Initializers

	g := dot.NewGraph(dot.Directed)
	g.NodeInitializer(func(n dot.Node) {
		n.Attr("shape", "rectangle")
		n.Attr("fontname", "arial")
		n.Attr("style", "rounded,filled")
	})

	g.EdgeInitializer(func(e dot.Edge) {
		e.Attr("fontname", "arial")
		e.Attr("fontsize", "9")
		e.Attr("arrowsize", "0.8")
		e.Attr("arrowhead", "open")
	})

HTML and Literal values

	node.Attr("label", Literal(`"left-justified text\l"`))
	graph.Attr("label", HTML("<B>Hi</B>"))

## cluster example

![](./doc/cluster.png)

	di := dot.NewGraph(dot.Directed)
	outside := di.Node("Outside")

	// A
	clusterA := di.Subgraph("Cluster A", dot.ClusterOption{})
	insideOne := clusterA.Node("one")
	insideTwo := clusterA.Node("two")
	
	// B
	clusterB := di.Subgraph("Cluster B", dot.ClusterOption{})
	insideThree := clusterB.Node("three")
	insideFour := clusterB.Node("four")

	// edges
	outside.Edge(insideFour).Edge(insideOne).Edge(insideTwo).Edge(insideThree).Edge(outside)

See also `ext/Subsystem` type for creating composition hierarchies.


## record example

See `record_test.go#ExampleNode_NewRecordBuilder`.

## About dot attributes

https://graphviz.gitlab.io/doc/info/attrs.html

## display your graph

	go run main.go | dot -Tpng  > test.png && open test.png

## mermaid

Output a dot Graph using the [mermaid](https://mermaid-js.github.io/mermaid/#/README) syntax.
Only Graph and Flowchart are supported. See MermaidGraph and MermaidFlowchart.

```
g := dot.NewGraph(dot.Directed)
...
fmt.Println(dot.MermaidGraph(g, dot.MermaidTopToBottom))
```

## extensions

See also package `dot/dotx` for types that can help in constructing complex graphs.

![](./doc/TestExampleSubsystemSameGraph.png)

### testing

	go test -coverprofile=coverage.out ./...
	go tool cover -html=coverage.out

(c) 2015-2023, http://ernestmicklei.com. MIT License.
