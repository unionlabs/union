package xml

import (
	"encoding/xml"
	"fmt"
	"io"
	"sort"
	"strings"
)

// A Node contains the values to be encoded or decoded.
type Node struct {
	Name     xml.Name           `json:",omitempty"`
	Children map[string][]*Node `json:",omitempty"`
	Text     string             `json:",omitempty"`
	Attr     []xml.Attr         `json:",omitempty"`

	namespaces map[string]string
	parent     *Node
}

// NewXMLElement returns a pointer to a new Node initialized to default values.
func NewXMLElement(name xml.Name) *Node {
	return &Node{
		Name:     name,
		Children: map[string][]*Node{},
		Attr:     []xml.Attr{},
	}
}

// AddChild adds child to the Node.
func (n *Node) AddChild(child *Node) {
	child.parent = n
	if _, ok := n.Children[child.Name.Local]; !ok {
		// flattened will have multiple children with same tag name
		n.Children[child.Name.Local] = []*Node{}
	}
	n.Children[child.Name.Local] = append(n.Children[child.Name.Local], child)
}

// ToStruct converts a xml.Decoder stream to Node with nested values.
func ToStruct(d *xml.Decoder, s *xml.StartElement, ignoreIndentation bool) (*Node, error) {
	out := &Node{}

	for {
		tok, err := d.Token()
		if err != nil {
			if err == io.EOF {
				break
			} else {
				return out, err
			}
		}

		if tok == nil {
			break
		}

		switch typed := tok.(type) {
		case xml.CharData:
			text := string(typed.Copy())
			if ignoreIndentation {
				text = strings.TrimSpace(text)
			}
			if len(text) != 0 {
				out.Text = text
			}
		case xml.StartElement:
			el := typed.Copy()
			out.Attr = el.Attr
			if out.Children == nil {
				out.Children = map[string][]*Node{}
			}

			name := typed.Name.Local
			slice := out.Children[name]
			if slice == nil {
				slice = []*Node{}
			}
			node, e := ToStruct(d, &el, ignoreIndentation)
			out.findNamespaces()
			if e != nil {
				return out, e
			}

			node.Name = typed.Name
			node.findNamespaces()

			// Add attributes onto the node
			node.Attr = el.Attr

			tempOut := *out
			// Save into a temp variable, simply because out gets squashed during
			// loop iterations
			node.parent = &tempOut
			slice = append(slice, node)
			out.Children[name] = slice
		case xml.EndElement:
			if s != nil && s.Name.Local == typed.Name.Local { // matching end token
				return out, nil
			}
			out = &Node{}
		}
	}
	return out, nil
}

func (n *Node) findNamespaces() {
	ns := map[string]string{}
	for _, a := range n.Attr {
		if a.Name.Space == "xmlns" {
			ns[a.Value] = a.Name.Local
		}
	}

	n.namespaces = ns
}

func (n *Node) findElem(name string) (string, bool) {
	for node := n; node != nil; node = node.parent {
		for _, a := range node.Attr {
			namespace := a.Name.Space
			if v, ok := node.namespaces[namespace]; ok {
				namespace = v
			}
			if name == fmt.Sprintf("%s:%s", namespace, a.Name.Local) {
				return a.Value, true
			}
		}
	}
	return "", false
}

// StructToXML writes an Node to a xml.Encoder as tokens.
func StructToXML(e *xml.Encoder, node *Node, sorted bool) error {
	var err error
	// Sort Attributes
	attrs := node.Attr
	if sorted {
		sortedAttrs := make([]xml.Attr, len(attrs))
		for _, k := range node.Attr {
			sortedAttrs = append(sortedAttrs, k)
		}
		sort.Sort(xmlAttrSlice(sortedAttrs))
		attrs = sortedAttrs
	}

	st := xml.StartElement{Name: node.Name, Attr: attrs}
	e.EncodeToken(st)
	// return fmt.Errorf("encoder string : %s, %s, %s", node.Name.Local, node.Name.Space, st.Attr)

	if node.Text != "" {
		e.EncodeToken(xml.CharData([]byte(node.Text)))
	} else if sorted {
		sortedNames := []string{}
		for k := range node.Children {
			sortedNames = append(sortedNames, k)
		}
		sort.Strings(sortedNames)

		for _, k := range sortedNames {
			// we should sort the []*xml.Node for each key if len >1
			flattenedNodes := node.Children[k]
			// Meaning this has multiple nodes
			if len(flattenedNodes) > 1 {
				// sort flattened nodes
				flattenedNodes, err = sortFlattenedNodes(flattenedNodes)
				if err != nil {
					return err
				}
			}

			for _, v := range flattenedNodes {
				err = StructToXML(e, v, sorted)
				if err != nil {
					return err
				}
			}
		}
	} else {
		for _, c := range node.Children {
			for _, v := range c {
				err = StructToXML(e, v, sorted)
				if err != nil {
					return err
				}
			}
		}
	}

	e.EncodeToken(xml.EndElement{Name: node.Name})
	return e.Flush()
}

// sortFlattenedNodes sorts nodes with nodes having same element tag
// but overall different values. The function will return list of pointer to
// Node and an error.
//
// Overall sort order is followed is:
// Nodes with concrete value (no nested node as value) are given precedence
// and are added to list after sorting them
//
// Next nested nodes within a flattened list are given precedence.
//
// Next nodes within a flattened map are sorted based on either key or value
// which ever has lower value and then added to the global sorted list.
// If value was initially chosen, but has nested nodes; key will be chosen as comparable
// as it is unique and will always have concrete data ie. string.
func sortFlattenedNodes(nodes []*Node) ([]*Node, error) {
	var sortedNodes []*Node

	// concreteNodeMap stores concrete value associated with a list of nodes
	// This is possible in case multiple members of a flatList has same values.
	concreteNodeMap := make(map[string][]*Node, 0)

	// flatListNodeMap stores flat list or wrapped list members associated with a list of nodes
	// This will have only flattened list with members that are Nodes and not concrete values.
	flatListNodeMap := make(map[string][]*Node, 0)

	// flatMapNodeMap stores flat map or map entry members associated with a list of nodes
	// This will have only flattened map concrete value members. It is possible to limit this
	// to concrete value as map key is expected to be concrete.
	flatMapNodeMap := make(map[string][]*Node, 0)

	// nodes with concrete value are prioritized and appended based on sorting order
	sortedNodesWithConcreteValue := []string{}

	// list with nested nodes are second in priority and appended based on sorting order
	sortedNodesWithListValue := []string{}

	// map are last in priority and appended based on sorting order
	sortedNodesWithMapValue := []string{}

	for _, node := range nodes {
		// node has no children element, then we consider it as having concrete value
		if len(node.Children) == 0 {
			sortedNodesWithConcreteValue = append(sortedNodesWithConcreteValue, node.Text)
			if v, ok := concreteNodeMap[node.Text]; ok {
				concreteNodeMap[node.Text] = append(v, node)
			} else {
				concreteNodeMap[node.Text] = []*Node{node}
			}
		}

		// if node has a single child, then it is a flattened list node
		if len(node.Children) == 1 {
			for _, nestedNodes := range node.Children {
				nestedNodeName := nestedNodes[0].Name.Local

				// append to sorted node name for list value
				sortedNodesWithListValue = append(sortedNodesWithListValue, nestedNodeName)

				if v, ok := flatListNodeMap[nestedNodeName]; ok {
					flatListNodeMap[nestedNodeName] = append(v, nestedNodes[0])
				} else {
					flatListNodeMap[nestedNodeName] = []*Node{nestedNodes[0]}
				}
			}
		}

		// if node has two children, then it is a flattened map node
		if len(node.Children) == 2 {
			nestedPair := []*Node{}
			for _, k := range node.Children {
				nestedPair = append(nestedPair, k[0])
			}

			comparableValues := []string{nestedPair[0].Name.Local, nestedPair[1].Name.Local}
			sort.Strings(comparableValues)

			comparableValue := comparableValues[0]
			for _, nestedNode := range nestedPair {
				if comparableValue == nestedNode.Name.Local && len(nestedNode.Children) != 0 {
					// if value was selected and is nested node, skip it and use key instead
					comparableValue = comparableValues[1]
					continue
				}

				// now we are certain there is no nested node
				if comparableValue == nestedNode.Name.Local {
					// get chardata for comparison
					comparableValue = nestedNode.Text
					sortedNodesWithMapValue = append(sortedNodesWithMapValue, comparableValue)

					if v, ok := flatMapNodeMap[comparableValue]; ok {
						flatMapNodeMap[comparableValue] = append(v, node)
					} else {
						flatMapNodeMap[comparableValue] = []*Node{node}
					}
					break
				}
			}
		}

		// we don't support multiple same name nodes in an xml doc except for in flattened maps, list.
		if len(node.Children) > 2 {
			return nodes, fmt.Errorf("malformed xml: multiple nodes with same key name exist, " +
				"but are not associated with flattened maps (2 children) or list (0 or 1 child)")
		}
	}

	// sort concrete value node name list and append corresponding nodes
	// to sortedNodes
	sort.Strings(sortedNodesWithConcreteValue)
	for _, name := range sortedNodesWithConcreteValue {
		for _, node := range concreteNodeMap[name] {
			sortedNodes = append(sortedNodes, node)
		}
	}

	// sort nested nodes with a list and append corresponding nodes
	// to sortedNodes
	sort.Strings(sortedNodesWithListValue)
	for _, name := range sortedNodesWithListValue {
		// if two nested nodes have same name, then sort them separately.
		if len(flatListNodeMap[name]) > 1 {
			// return nodes, fmt.Errorf("flat list node name are %s %v", flatListNodeMap[name][0].Name.Local, len(flatListNodeMap[name]))
			nestedFlattenedList, err := sortFlattenedNodes(flatListNodeMap[name])
			if err != nil {
				return nodes, err
			}
			// append the identical but sorted nodes
			for _, nestedNode := range nestedFlattenedList {
				sortedNodes = append(sortedNodes, nestedNode)
			}
		} else {
			// append the sorted nodes
			sortedNodes = append(sortedNodes, flatListNodeMap[name][0])
		}
	}

	// sorted nodes with a map and append corresponding nodes to sortedNodes
	sort.Strings(sortedNodesWithMapValue)
	for _, name := range sortedNodesWithMapValue {
		sortedNodes = append(sortedNodes, flatMapNodeMap[name][0])
	}

	return sortedNodes, nil
}
