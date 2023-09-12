package depinject

import (
	"fmt"
	"reflect"

	"github.com/cockroachdb/errors"

	"cosmossdk.io/depinject/internal/graphviz"
)

// ManyPerContainerType marks a type which automatically gets grouped together. For an ManyPerContainerType T,
// T and []T can be declared as output parameters for providers as many times within the container
// as desired. All of the provided values for T can be retrieved by declaring an
// []T input parameter.
type ManyPerContainerType interface {
	// IsManyPerContainerType is a marker function which just indicates that this is a many-per-container type.
	IsManyPerContainerType()
}

var manyPerContainerTypeType = reflect.TypeOf((*ManyPerContainerType)(nil)).Elem()

func isManyPerContainerType(t reflect.Type) bool {
	return t.Implements(manyPerContainerTypeType)
}

func isManyPerContainerSliceType(typ reflect.Type) bool {
	return typ.Kind() == reflect.Slice && isManyPerContainerType(typ.Elem())
}

type groupResolver struct {
	typ          reflect.Type
	sliceType    reflect.Type
	idxsInValues []int
	providers    []*simpleProvider
	resolved     bool
	values       reflect.Value
	graphNode    *graphviz.Node
}

func (g *groupResolver) getType() reflect.Type {
	return g.sliceType
}

type sliceGroupResolver struct {
	*groupResolver
}

func (g *groupResolver) describeLocation() string {
	return fmt.Sprintf("many-per-container type %v", g.typ)
}

func (g *sliceGroupResolver) resolve(c *container, _ *moduleKey, caller Location) (reflect.Value, error) {
	// Log
	c.logf("Providing many-per-container type slice %v to %s from:", g.sliceType, caller.Name())
	c.indentLogger()
	for _, node := range g.providers {
		c.logf(node.provider.Location.String())
	}
	c.dedentLogger()

	// Resolve
	if !g.resolved {
		res := reflect.MakeSlice(g.sliceType, 0, 0)
		for i, node := range g.providers {
			values, err := node.resolveValues(c)
			if err != nil {
				return reflect.Value{}, err
			}
			value := values[g.idxsInValues[i]]
			if value.Kind() == reflect.Slice {
				n := value.Len()
				for j := 0; j < n; j++ {
					res = reflect.Append(res, value.Index(j))
				}
			} else {
				res = reflect.Append(res, value)
			}
		}
		g.values = res
		g.resolved = true
	}

	return g.values, nil
}

func (g *groupResolver) resolve(_ *container, _ *moduleKey, _ Location) (reflect.Value, error) {
	return reflect.Value{}, errors.Errorf("%v is an many-per-container type and cannot be used as an input value, instead use %v", g.typ, g.sliceType)
}

func (g *groupResolver) addNode(n *simpleProvider, i int) error {
	g.providers = append(g.providers, n)
	g.idxsInValues = append(g.idxsInValues, i)
	return nil
}

func (g groupResolver) typeGraphNode() *graphviz.Node {
	return g.graphNode
}
