package dot

// HTML renders the provided content as graphviz HTML. Use of this
// type is only valid for some attributes, like the 'label' attribute.
type HTML string

// Literal renders the provided value as is, without adding enclosing
// quotes, escaping newlines, quotations marks or any other characters.
// For example:
//     node.Attr("label", Literal(`"left-justified text\l"`))
// allows you to left-justify the label (due to the \l at the end).
// The caller is responsible for enclosing the value in quotes and for
// proper escaping of special characters.
type Literal string

// AttributesMap holds attribute=value pairs.
type AttributesMap struct {
	attributes map[string]interface{}
}

// Attrs sets multiple values for attributes (unless empty) taking a label,value list
// E.g  Attrs("style","filled","fillcolor","red")
func (a AttributesMap) Attrs(labelvalues ...interface{}) {
	if len(labelvalues)%2 != 0 {
		panic("missing label or value ; must provide pairs")
	}
	for i := 0; i < len(labelvalues); i += 2 {
		label := labelvalues[i].(string)
		value := labelvalues[i+1]
		a.Attr(label, value)
	}
}

// Attr sets the value for an attribute (unless empty).
func (a AttributesMap) Attr(label string, value interface{}) {
	if len(label) == 0 || value == nil {
		return
	}
	if s, ok := value.(string); ok {
		if len(s) > 0 {
			a.attributes[label] = s
			return
		}
	}
	a.attributes[label] = value
}

// Value return the value added for this label.
func (a AttributesMap) Value(label string) interface{} {
	return a.attributes[label]
}

// Delete removes the attribute value at key, if any
func (a AttributesMap) Delete(key string) {
	delete(a.attributes, key)
}
