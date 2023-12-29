package depinject

import (
	"reflect"

	"cosmossdk.io/depinject/internal/graphviz"
)

type supplyResolver struct {
	typ       reflect.Type
	value     reflect.Value
	loc       Location
	graphNode *graphviz.Node
}

func (s supplyResolver) getType() reflect.Type {
	return s.typ
}

func (s supplyResolver) describeLocation() string {
	return s.loc.String()
}

func (s supplyResolver) addNode(provider *simpleProvider, _ int) error {
	return duplicateDefinitionError(s.typ, provider.provider.Location, s.loc.String())
}

func (s supplyResolver) resolve(c *container, _ *moduleKey, caller Location) (reflect.Value, error) {
	c.logf("Supplying %v from %s to %s", s.typ, s.loc, caller.Name())
	return s.value, nil
}

func (s supplyResolver) typeGraphNode() *graphviz.Node {
	return s.graphNode
}
