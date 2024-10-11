package dot

import (
	"os"
	"reflect"
	"strings"
	"testing"
)

func TestEmpty(t *testing.T) {
	di := NewGraph(Directed)
	if got, want := flatten(di.String()), `digraph  {}`; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}
}

func TestEmptyStrictDirected(t *testing.T) {
	di := NewGraph(Directed, Strict)
	if got, want := flatten(di.String()), `strict digraph  {}`; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}
	di2 := NewGraph(Strict, Directed)
	if got, want := flatten(di2.String()), `strict digraph  {}`; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}

}

func TestOverrideID(t *testing.T) {
	caught := false
	defer func() {
		if r := recover(); r != nil {
			caught = true
		}
	}()
	di := NewGraph(Directed)
	di.ID("one")
	if got, want := di.GetID(), "one"; got != want {
		t.Errorf("got [%v]:%T want [%v]:%T", got, got, want, want)
	}
	di.ID("two")
	if !caught {
		t.Fail()
	}
}

func TestEmptyWithIDAndAttributes(t *testing.T) {
	di := NewGraph(Directed)
	di.ID("test")
	di.Attr("style", "filled")
	di.Attr("color", "lightgrey")
	if got, want := flatten(di.String()), `digraph test {color="lightgrey";style="filled";}`; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}
}

func TestEmptyWithHTMLLabel(t *testing.T) {
	di := NewGraph(Directed)
	di.ID("test")
	di.Attr("label", HTML("<B>Hi</B>"))
	if got, want := flatten(di.String()), `digraph test {label=<<B>Hi</B>>;}`; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}
}

func TestDeleteNode(t *testing.T) {
	di := NewGraph(Undirected)
	n1 := di.Node("A")
	n2 := di.Node("B")
	n3 := di.Node("C")
	di.Edge(n1, n2) // Will be deleted
	di.Edge(n2, n3) // Will also be deleted
	di.Edge(n1, n3) // Must not be deleted
	wasDeleted := di.DeleteNode("B")
	if got, want := flatten(di.String()), `graph  {n1[label="A"];n3[label="C"];n1--n3;}`; wasDeleted && got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}
}

func TestDeleteNodeWhenNodeDoesNotExist(t *testing.T) {
	di := NewGraph(Directed)
	n1 := di.Node("A")
	n2 := di.Node("B")
	n3 := di.Node("C")
	di.Edge(n1, n2)
	di.Edge(n2, n3)
	wasDeleted := di.DeleteNode("D")

	if got, want := flatten(di.String()), `digraph  {n1[label="A"];n2[label="B"];n3[label="C"];n1->n2;n2->n3;}`; !wasDeleted && got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}
}

func TestEmptyWithLiteralValueLabel(t *testing.T) {
	di := NewGraph(Directed)
	di.ID("test")
	di.Attr("label", Literal(`"left-justified text\l"`))
	if got, want := flatten(di.String()), `digraph test {label="left-justified text\l";}`; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}
}

func TestTwoConnectedNodes(t *testing.T) {
	di := NewGraph(Directed)
	n1 := di.Node("A")
	n2 := di.Node("B")
	if !di.HasNode(n1) {
		t.Fail()
	}
	di.Edge(n1, n2)
	if got, want := flatten(di.String()), `digraph  {n1[label="A"];n2[label="B"];n1->n2;}`; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}
}

func TestGraph_FindEdges(t *testing.T) {
	di := NewGraph(Directed)
	n1 := di.Node("A")
	n2 := di.Node("B")
	want := []Edge{di.Edge(n1, n2)}
	got := di.FindEdges(n1, n2)
	if !reflect.DeepEqual(got, want) {
		t.Errorf("TestGraph.FindEdges() = %v, want %v", got, want)
	}
	n3 := di.Node("C")
	n2.Edge(n3)
	list := want[0].EdgesTo(n3)
	if len(list) != 1 {
		t.Fail()
	}
}

func TestSubgraph(t *testing.T) {
	di := NewGraph(Directed)
	sub := di.Subgraph("test-id")
	if second := di.Subgraph("test-id"); second != sub {
		t.Fatal()
	}
	sub.Attr("style", "filled")
	if got, want := flatten(di.String()), `digraph  {subgraph s1 {label="test-id";style="filled";}}`; got != want {
		t.Errorf("got\n[%v] want\n[%v]", got, want)
	}
	sub.Label("new-label")
	if got, want := flatten(di.String()), `digraph  {subgraph s1 {label="new-label";style="filled";}}`; got != want {
		t.Errorf("got\n[%v] want\n[%v]", got, want)
	}
	found, _ := di.FindSubgraph("test-id")
	if got, want := found, sub; got != want {
		t.Errorf("got [%v:%T] want [%v:%T]", got, got, want, want)
	}
	subsub := sub.Subgraph("sub-test-id")
	found, _ = subsub.FindSubgraph("test-id")
	if got, want := found, sub; got != want {
		t.Errorf("got [%v:%T] want [%v:%T]", got, got, want, want)
	}

}

func TestSubgraphIgnoreStrict(t *testing.T) {
	di := NewGraph()
	_ = di.Subgraph("test", ClusterOption{}, Strict)
	if got, want := flatten(di.String()), `digraph  {subgraph cluster_s1 {label="test";}}`; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}
	di2 := NewGraph()
	_ = di2.Subgraph("test", Strict, ClusterOption{})
	if got, want := flatten(di2.String()), `digraph  {subgraph cluster_s1 {label="test";}}`; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}
}

func TestSubgraphClusterOption(t *testing.T) {
	di := NewGraph(Directed)
	sub := di.Subgraph("test", ClusterOption{})
	if got, want := sub.id, "cluster_s1"; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}
}

func TestEdgeLabel(t *testing.T) {
	di := NewGraph(Directed)
	n1 := di.Node("e1")
	n2 := di.Node("e2")
	n1.Edge(n2, "what").Attr("x", "y")
	if got, want := flatten(di.String()), `digraph  {n1[label="e1"];n2[label="e2"];n1->n2[label="what",x="y"];}`; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}
}

func TestSameRank(t *testing.T) {
	di := NewGraph(Directed)
	foo1 := di.Node("foo1")
	foo2 := di.Node("foo2")
	bar := di.Node("bar")
	foo1.Edge(foo2)
	foo1.Edge(bar)
	di.AddToSameRank("top-row", foo1, foo2)
	if got, want := flatten(di.String()), `digraph  {n3[label="bar"];n1[label="foo1"];n2[label="foo2"];n1->n2;n1->n3;{rank=same; n1;n2;};}`; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}
}

// dot -Tpng cluster.dot > cluster.png && open cluster.png
func TestCluster(t *testing.T) {
	di := NewGraph(Directed)
	outside := di.Node("Outside")
	clusterA := di.Subgraph("Cluster A", ClusterOption{})
	insideOne := clusterA.Node("one")
	insideTwo := clusterA.Node("two")
	clusterB := di.Subgraph("Cluster B", ClusterOption{})
	insideThree := clusterB.Node("three")
	insideFour := clusterB.Node("four")
	outside.Edge(insideFour).Edge(insideOne).Edge(insideTwo).Edge(insideThree).Edge(outside)
	os.WriteFile("doc/cluster.dot", []byte(di.String()), os.ModePerm)
}

// remove tabs and newlines and spaces
func flatten(s string) string {
	return strings.Replace((strings.Replace(s, "\n", "", -1)), "\t", "", -1)
}

func TestDeleteLabel(t *testing.T) {
	g := NewGraph()
	n := g.Node("my-id")
	n.AttributesMap.Delete("label")
	if got, want := flatten(g.String()), `digraph  {n1;}`; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}
}

func TestGraph_FindNodeById_emptyGraph(t *testing.T) {
	di := NewGraph(Directed)

	_, found := di.FindNodeById("F")

	if got, want := found, false; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}
}

func TestGraph_FindNodeById_multiNodeGraph(t *testing.T) {
	di := NewGraph(Directed)
	di.Node("A")
	di.Node("B")

	node, found := di.FindNodeById("A")

	if got, want := node.id, "A"; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}

	if got, want := found, true; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}
}

func TestGraph_FindNodeById_multiNodesInSubGraphs(t *testing.T) {
	di := NewGraph(Directed)
	di.Node("A")
	di.Node("B")
	sub := di.Subgraph("new subgraph")
	sub.Node("C")

	node, found := di.FindNodeById("C")

	if got, want := node.id, "C"; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}

	if got, want := found, true; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}
}

func TestGraph_FindNodes_multiNodesInSubGraphs(t *testing.T) {
	di := NewGraph(Directed)
	di.Node("A")
	di.Node("B")
	sub := di.Subgraph("new subgraph")
	sub.Node("C")

	nodes := di.FindNodes()

	if got, want := len(nodes), 3; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}
}

func TestLabelWithEscaping(t *testing.T) {
	di := NewGraph(Directed)
	n := di.Node("without linefeed")
	n.Attr("label", Literal(`"with \l linefeed"`))
	if got, want := flatten(di.String()), `digraph  {n1[label="with \l linefeed"];}`; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}
}

func TestGraphNodeInitializer(t *testing.T) {
	di := NewGraph(Directed)
	di.NodeInitializer(func(n Node) {
		n.Attr("test", "test")
	})
	n := di.Node("A")
	if got, want := n.attributes["test"], "test"; got != want {
		t.Errorf("got [%v:%T] want [%v:%T]", got, got, want, want)
	}
}

func TestGraphEdgeInitializer(t *testing.T) {
	di := NewGraph(Directed)
	di.EdgeInitializer(func(e Edge) {
		e.Attr("test", "test")
	})
	e := di.Node("A").Edge(di.Node("B"))
	if got, want := e.attributes["test"], "test"; got != want {
		t.Errorf("got [%v:%T] want [%v:%T]", got, got, want, want)
	}
}

func TestGraphCreateNodeOnce(t *testing.T) {
	di := NewGraph(Undirected)
	n1 := di.Node("A")
	n2 := di.Node("A")
	if got, want := n1, n2; &n1 == &n2 {
		t.Errorf("got [%v:%T] want [%v:%T]", got, got, want, want)
	}
}

func TestGraphCommonParent(t *testing.T) {
	di := NewGraph(Directed)
	a := di.Node("a")
	b := di.Node("b")
	s1 := di.Subgraph("s1")
	a1 := s1.Node("a1")
	b1 := s1.Node("b1")
	s2 := di.Subgraph("s2")
	a2 := s2.Node("a2")
	b2 := s2.Node("b2")
	a.Edge(a1)
	b.Edge(b2)
	e := a2.Edge(b1)
	if got, want := flatten(di.String()), `digraph  {subgraph s3 {label="s1";n4[label="a1"];n5[label="b1"];}subgraph s6 {label="s2";n7[label="a2"];n8[label="b2"];}n1[label="a"];n2[label="b"];n1->n4;n7->n5;n2->n8;}`; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}
	list := di.FindEdges(a2, b1)
	if len(list) != 1 {
		t.Fail()
	}
	if !reflect.DeepEqual(list[0], e) {
		t.Fail()
	}
	same := a2.EdgesTo(b1)
	if !reflect.DeepEqual(list, same) {
		t.Fail()
	}
}

func TestReverseEdge(t *testing.T) {
	di := NewGraph(Directed)
	if !di.IsDirected() {
		t.Fail()
	}
	a := di.Node("a")
	b := di.Node("b")
	e := a.ReverseEdge(b)
	if got, want := flatten(di.String()), `digraph  {n1[label="a"];n2[label="b"];n2->n1;}`; got != want {
		t.Errorf("got [%v]:%T want [%v]:%T", got, got, want, want)
	}
	if got, want := e.From().ID(), b.ID(); got != want {
		t.Errorf("got [%v]:%T want [%v]:%T", got, got, want, want)
	}
	if got, want := e.To().id, a.id; got != want {
		t.Errorf("got [%v]:%T want [%v]:%T", got, got, want, want)
	}
	m := di.EdgesMap()
	if got, want := len(m), 1; got != want {
		t.Errorf("got [%v]:%T want [%v]:%T", got, got, want, want)
	}
	if got, want := len(m["b"]), 1; got != want {
		t.Errorf("got [%v]:%T want [%v]:%T", got, got, want, want)
	}
	c := di.Node("c")
	e.ReverseEdge(c)
	if got, want := flatten(di.String()), `digraph  {n1[label="a"];n2[label="b"];n3[label="c"];n2->n1;n3->n1;}`; got != want {
		t.Errorf("got [%v]:%T want [%v]:%T", got, got, want, want)
	}
}

func TestFindNodeWithLabel(t *testing.T) {
	di := NewGraph(Directed)
	di.Node("A")
	di.Node("B")
	sub := di.Subgraph("new subgraph")
	sub.Node("C")

	n1, ok := di.FindNodeWithLabel("B")
	if !ok {
		t.Fail()
	}
	if l := n1.GetAttr("label"); l != "B" {
		t.Fail()
	}

	n2, ok := sub.FindNodeWithLabel("A")
	if !ok {
		t.Fail()
	}
	if l := n2.GetAttr("label"); l != "A" {
		t.Fail()
	}

	_, ok = sub.FindNodeWithLabel("D")
	if ok {
		t.Fail()
	}
}
