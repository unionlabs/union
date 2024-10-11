package dot

import (
	"testing"
)

func TestMermaidSimple(t *testing.T) {
	di := NewGraph(Directed)
	n1 := di.Node("e1").Label("E1")
	n2 := di.Node("e2").Attr("shape", MermaidShapeRound).Attr("style", "fill:#90EE90")
	n1.Edge(n2, "what").Attr("x", "y")
	out := flatten(MermaidGraph(di, MermaidTopDown))
	if got, want := out, `graph TD;n1("E1");n2("e2");style n2 fill:#90EE90n1-->|"what"|n2;`; got != want {
		t.Errorf("got [%v]:%T want [%v]:%T", got, got, want, want)
	}
}

func TestEmptyFlow(t *testing.T) {
	di := NewGraph(Directed)
	s := MermaidFlowchart(di, MermaidTopDown)
	if got, want := s, "flowchart TD;\n"; got != want {
		t.Errorf("got [%v]:%T want [%v]:%T", got, got, want, want)
	}
}
func TestEmptyGraphLR(t *testing.T) {
	di := NewGraph(Directed)
	s := MermaidGraph(di, MermaidLeftToRight)
	if got, want := s, "graph LR;\n"; got != want {
		t.Errorf("got [%v]:%T want [%v]:%T", got, got, want, want)
	}
	s = MermaidGraph(di, MermaidRightToLeft)
	if got, want := s, "graph RL;\n"; got != want {
		t.Errorf("got [%v]:%T want [%v]:%T", got, got, want, want)
	}
	s = MermaidGraph(di, MermaidBottomToTop)
	if got, want := s, "graph BT;\n"; got != want {
		t.Errorf("got [%v]:%T want [%v]:%T", got, got, want, want)
	}
	s = MermaidGraph(di, 42)
	if got, want := s, "graph TD;\n"; got != want {
		t.Errorf("got [%v]:%T want [%v]:%T", got, got, want, want)
	}
}
func TestMermaidShapes(t *testing.T) {
	di := NewGraph(Directed)
	di.Node("round").Attr("shape", MermaidShapeRound)
	di.Node("asym").Attr("shape", MermaidShapeAsymmetric)
	di.Node("circ").Attr("shape", MermaidShapeCircle)
	di.Node("cyl").Attr("shape", MermaidShapeCylinder)
	di.Node("rhom").Attr("shape", MermaidShapeRhombus)
	di.Node("stad").Attr("shape", MermaidShapeStadium)
	di.Node("sub").Attr("shape", MermaidShapeSubroutine)
	di.Node("trap").Attr("shape", MermaidShapeTrapezoid)
	di.Node("trapalt").Attr("shape", MermaidShapeTrapezoidAlt)
	s := MermaidGraph(di, MermaidLeftToRight)
	// t.Log(s)
	if got, want := flatten(s), `graph LR;n2>"asym"];n3(("circ"));n4[("cyl")];n5{"rhom"};n1("round");n6(["stad"]);n7[["sub"]];n8[/"trap"\];n9[\"trapalt"/];`; got != want {
		t.Errorf("got [%v]:%T want [%v]:%T", got, got, want, want)
	}
}

// Deprecated: Use MermaidShapeCircle instead of MermaidShapeCirle
func TestMermaidShapeCirle(t *testing.T) {
	di := NewGraph(Directed)
	di.Node("circ").Attr("shape", MermaidShapeCirle)
	s := MermaidGraph(di, MermaidLeftToRight)
	// t.Log(s)
	if got, want := flatten(s), `graph LR;n1(("circ"));`; got != want {
		t.Errorf("got [%v]:%T want [%v]:%T", got, got, want, want)
	}
}

func TestUndirectedMermaid(t *testing.T) {
	un := NewGraph(Undirected)
	un.Node("love").Edge(un.Node("happinez"))
	s := MermaidFlowchart(un, MermaidLeftToRight)
	//t.Log(s)
	if got, want := flatten(s), `flowchart LR;n2("happinez");n1("love");n1---n2;`; got != want {
		t.Errorf("got [%v]:%T want [%v]:%T", got, got, want, want)
	}
}
