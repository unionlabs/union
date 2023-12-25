package graphviz

import (
	"fmt"
	"io"
)

// Edge represents a graphviz edge.
type Edge struct {
	*Attributes
	from, to *Node
}

func (e Edge) render(w io.Writer, indent string) error {
	_, err := fmt.Fprintf(w, "%s%q -> %q%s;\n", indent, e.from.name, e.to.name, e.Attributes.String())
	return err
}
