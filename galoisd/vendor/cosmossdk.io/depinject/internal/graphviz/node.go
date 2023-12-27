package graphviz

import (
	"fmt"
	"io"
)

// Node represents a graphviz node.
type Node struct {
	*Attributes
	name string
}

func (n Node) render(w io.Writer, indent string) error {
	_, err := fmt.Fprintf(w, "%s%q%s;\n", indent, n.name, n.Attributes.String())
	return err
}
