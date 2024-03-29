package dot

import (
	"fmt"
	"html"
	"strings"
)

const (
	MermaidTopToBottom = iota
	MermaidTopDown
	MermaidBottomToTop
	MermaidRightToLeft
	MermaidLeftToRight
)

var (
	MermaidShapeRound        = shape{"(", ")"}
	MermaidShapeStadium      = shape{"([", "])"}
	MermaidShapeSubroutine   = shape{"[[", "]]"}
	MermaidShapeCylinder     = shape{"[(", ")]"}
	MermaidShapeCirle        = shape{"((", "))"} // Deprecated: use MermaidShapeCircle instead
	MermaidShapeCircle       = shape{"((", "))"}
	MermaidShapeAsymmetric   = shape{">", "]"}
	MermaidShapeRhombus      = shape{"{", "}"}
	MermaidShapeTrapezoid    = shape{"[/", "\\]"}
	MermaidShapeTrapezoidAlt = shape{"[\\", "/]"}
)

type shape struct {
	open, close string
}

func MermaidGraph(g *Graph, orientation int) string {
	return diagram(g, "graph", orientation)
}

func MermaidFlowchart(g *Graph, orientation int) string {
	return diagram(g, "flowchart", orientation)
}

func escape(value string) string {
	return fmt.Sprintf(`"%s"`, html.EscapeString(value))
}

func diagram(g *Graph, diagramType string, orientation int) string {
	sb := new(strings.Builder)
	sb.WriteString(diagramType)
	sb.WriteRune(' ')
	switch orientation {
	case MermaidTopDown, MermaidTopToBottom:
		sb.WriteString("TD")
	case MermaidBottomToTop:
		sb.WriteString("BT")
	case MermaidRightToLeft:
		sb.WriteString("RL")
	case MermaidLeftToRight:
		sb.WriteString("LR")
	default:
		sb.WriteString("TD")
	}
	writeEnd(sb)
	// graph nodes
	for _, key := range g.sortedNodesKeys() {
		nodeShape := MermaidShapeRound
		each := g.nodes[key]
		if s := each.GetAttr("shape"); s != nil {
			nodeShape = s.(shape)
		}
		txt := "?"
		if label := each.GetAttr("label"); label != nil {
			txt = label.(string)
		}
		fmt.Fprintf(sb, "\tn%d%s%s%s;\n", each.seq, nodeShape.open, escape(txt), nodeShape.close)
		if style := each.GetAttr("style"); style != nil {
			fmt.Fprintf(sb, "\tstyle n%d %s\n", each.seq, style.(string))
		}
	}
	// all edges
	// graph edges
	denoteEdge := "-->"
	if g.graphType == "graph" {
		denoteEdge = "---"
	}
	for _, each := range g.sortedEdgesFromKeys() {
		all := g.edgesFrom[each]
		for _, each := range all {
			if label := each.GetAttr("label"); label != nil {
				fmt.Fprintf(sb, "\tn%d%s|%s|n%d;\n", each.from.seq, denoteEdge, escape(label.(string)), each.to.seq)
			} else {
				fmt.Fprintf(sb, "\tn%d%sn%d;\n", each.from.seq, denoteEdge, each.to.seq)
			}
		}
	}
	return sb.String()
}

func writeEnd(sb *strings.Builder) {
	sb.WriteString(";\n")
}
