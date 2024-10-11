package dot

import (
	"testing"
)

func TestNode_Box(t *testing.T) {
	g := NewGraph(Directed)
	n := g.Node("A")
	n.Box()
	if n.Value("shape") != "box" {
		t.Fail()
	}
}

func TestNode_Label(t *testing.T) {
	g := NewGraph(Directed)
	n := g.Node("A")
	n.Label("42")
	if n.Value("label") != "42" {
		t.Fail()
	}
}

func TestNodesWithBidirectionalEdge(t *testing.T) {
	g := NewGraph(Directed)
	a := g.Node("A")
	b := g.Node("B")
	e := a.BidirectionalEdge(b)
	if got, want := len(e), 2; got != want {
		t.Errorf("got [%v]:%T want [%v]:%T", got, got, want, want)
	}
	if got, want := e[0].from, a; got.id != want.id {
		t.Errorf("got [%v]:%T want [%v]:%T", got, got, want, want)
	}
	if got, want := e[0].to, b; got.id != want.id {
		t.Errorf("got [%v]:%T want [%v]:%T", got, got, want, want)
	}
	if got, want := e[1].from, b; got.id != want.id {
		t.Errorf("got [%v]:%T want [%v]:%T", got, got, want, want)
	}
	if got, want := e[1].to, a; got.id != want.id {
		t.Errorf("got [%v]:%T want [%v]:%T", got, got, want, want)
	}
	if got, want := e[0].attributes["style"], "invis"; got != want {
		t.Errorf("got [%v]:%T want [%v]:%T", got, got, want, want)
	}
	if got, want := e[1].attributes["dir"], "both"; got != want {
		t.Errorf("got [%v]:%T want [%v]:%T", got, got, want, want)
	}
}
