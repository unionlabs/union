package graphviz

import (
	"fmt"
	"strings"

	"cosmossdk.io/depinject/internal/util"
)

// Attributes represents a graphviz attributes map.
type Attributes struct {
	attrs map[string]string
}

// NewAttributes creates a new Attributes instance.
func NewAttributes() *Attributes {
	return &Attributes{attrs: map[string]string{}}
}

// SetAttr sets the graphviz attribute to the provided value.
func (a *Attributes) SetAttr(name, value string) { a.attrs[name] = value }

// SetShape sets the shape attribute.
func (a *Attributes) SetShape(shape string) { a.SetAttr("shape", shape) }

// SetColor sets the color attribute.
func (a *Attributes) SetColor(color string) { a.SetAttr("color", color) }

// SetBgColor sets the bgcolor attribute.
func (a *Attributes) SetBgColor(color string) { a.SetAttr("bgcolor", color) }

// SetLabel sets the label attribute.
func (a *Attributes) SetLabel(label string) { a.SetAttr("label", label) }

// SetComment sets the comment attribute.
func (a *Attributes) SetComment(comment string) { a.SetAttr("comment", comment) }

// SetPenWidth sets the penwidth attribute.
func (a *Attributes) SetPenWidth(w string) { a.SetAttr("penwidth", w) }

// SetFontColor sets the fontcolor attribute.
func (a *Attributes) SetFontColor(color string) { a.SetAttr("fontcolor", color) }

// SetFontSize sets the fontsize attribute.
func (a *Attributes) SetFontSize(size string) { a.SetAttr("fontsize", size) }

// SetStyle sets the style attribute.
func (a *Attributes) SetStyle(style string) { a.SetAttr("style", style) }

// String returns the attributes graphviz string in the format [name = "value", ...].
func (a *Attributes) String() string {
	if len(a.attrs) == 0 {
		return ""
	}
	var attrStrs []string
	for _, k := range util.OrderedMapKeys(a.attrs) {
		attrStrs = append(attrStrs, fmt.Sprintf("%s=%q", k, a.attrs[k]))
	}
	return fmt.Sprintf("[%s]", strings.Join(attrStrs, ", "))
}
