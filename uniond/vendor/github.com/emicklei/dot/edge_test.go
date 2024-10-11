package dot

import (
	"testing"
)

func TestEdgeStyleHelpers(t *testing.T) {

	type test struct {
		input string
		want  string
	}

	tests := []test{
		{input: "solid", want: `digraph  {n1[label="A"];n2[label="B"];n1->n2[style="solid"];}`},
		{input: "bold", want: `digraph  {n1[label="A"];n2[label="B"];n1->n2[style="bold"];}`},
		{input: "dashed", want: `digraph  {n1[label="A"];n2[label="B"];n1->n2[style="dashed"];}`},
		{input: "dotted", want: `digraph  {n1[label="A"];n2[label="B"];n1->n2[style="dotted"];}`},
	}

	for _, tc := range tests {

		di := NewGraph(Directed)
		n1 := di.Node("A")
		n2 := di.Node("B")

		switch tc.input {
		case "solid":
			di.Edge(n1, n2).Solid()
		case "bold":
			di.Edge(n1, n2).Bold()
		case "dashed":
			di.Edge(n1, n2).Dashed()
		case "dotted":
			di.Edge(n1, n2).Dotted()
		}

		if got, want := flatten(di.String()), tc.want; got != want {
			t.Errorf("got [%v] want [%v]", got, want)
		}
	}
}

func TestEdgeWithTwoPorts(t *testing.T) {
	di := NewGraph(Directed)
	n1 := di.Node("A")
	n1.Attr("label", HTML("<table><tr><td port='port_a'>A</td></tr></table>"))
	n2 := di.Node("B")
	n2.Attr("label", HTML("<table><tr><td port='port_b'>B</td></tr></table>"))
	di.EdgeWithPorts(n1, n2, "port_a", "port_b")

	want := "digraph  {n1[label=<<table><tr><td port='port_a'>A</td></tr></table>>];n2[label=<<table><tr><td port='port_b'>B</td></tr></table>>];n1:port_a->n2:port_b;}"
	if got, want := flatten(di.String()), want; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}
}

func TestEdgeWithNoPorts(t *testing.T) {
	di := NewGraph(Directed)
	n1 := di.Node("A")
	n1.Attr("label", HTML("<table><tr><td>A</td></tr></table>"))
	n2 := di.Node("B")
	n2.Attr("label", HTML("<table><tr><td>B</td></tr></table>"))
	di.EdgeWithPorts(n1, n2, "", "")

	want := "digraph  {n1[label=<<table><tr><td>A</td></tr></table>>];n2[label=<<table><tr><td>B</td></tr></table>>];n1->n2;}"
	if got, want := flatten(di.String()), want; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}
}

func TestEdgeWithFirstPort(t *testing.T) {
	di := NewGraph(Directed)
	n1 := di.Node("A")
	n1.Attr("label", HTML("<table><tr><td port='port_a'>A</td></tr></table>"))
	n2 := di.Node("B")
	n2.Attr("label", HTML("<table><tr><td>B</td></tr></table>"))
	di.EdgeWithPorts(n1, n2, "port_a", "")

	want := "digraph  {n1[label=<<table><tr><td port='port_a'>A</td></tr></table>>];n2[label=<<table><tr><td>B</td></tr></table>>];n1:port_a->n2;}"
	if got, want := flatten(di.String()), want; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}
}

func TestEdgeWithSecondPort(t *testing.T) {
	di := NewGraph(Directed)
	n1 := di.Node("A")
	n1.Attr("label", HTML("<table><tr><td>A</td></tr></table>"))
	n2 := di.Node("B")
	n2.Attr("label", HTML("<table><tr><td port='port_b'>B</td></tr></table>"))
	di.EdgeWithPorts(n1, n2, "", "port_b")

	want := "digraph  {n1[label=<<table><tr><td>A</td></tr></table>>];n2[label=<<table><tr><td port='port_b'>B</td></tr></table>>];n1->n2:port_b;}"
	if got, want := flatten(di.String()), want; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}
}

func TestEdgeSetLabel(t *testing.T) {
	di := NewGraph(Directed)
	n1 := di.Node("A")
	n2 := di.Node("B")
	e := n1.Edge(n2).Label("ab")
	v := e.Value("label")
	if s, ok := v.(string); !ok {
		t.Fail()
	} else {
		if s != "ab" {
			t.Fail()
		}
	}
}

func TestNonStringAttribute(t *testing.T) {
	di := NewGraph(Directed)
	di.Node("A").Attr("shoesize", 42)
	if got, want := flatten(di.String()), `digraph  {n1[label="A",shoesize="42"];}`; got != want {
		t.Errorf("got [%v] want [%v]", got, want)
	}
}
