package depinject

import (
	"reflect"

	"cosmossdk.io/depinject/internal/graphviz"
)

type moduleDepProvider struct {
	provider        *providerDescriptor
	calledForModule map[*moduleKey]bool
	valueMap        map[*moduleKey][]reflect.Value
}

type moduleDepResolver struct {
	typ         reflect.Type
	idxInValues int
	node        *moduleDepProvider
	valueMap    map[*moduleKey]reflect.Value
	graphNode   *graphviz.Node
}

func (s moduleDepResolver) getType() reflect.Type {
	return s.typ
}

func (s moduleDepResolver) describeLocation() string {
	return s.node.provider.Location.String()
}

func (s moduleDepResolver) resolve(ctr *container, moduleKey *moduleKey, caller Location) (reflect.Value, error) {
	// Log
	ctr.logf("Providing %v from %s to %s", s.typ, s.node.provider.Location, caller.Name())

	// Resolve
	if val, ok := s.valueMap[moduleKey]; ok {
		return val, nil
	}

	if !s.node.calledForModule[moduleKey] {
		values, err := ctr.call(s.node.provider, moduleKey)
		if err != nil {
			return reflect.Value{}, err
		}

		s.node.valueMap[moduleKey] = values
		s.node.calledForModule[moduleKey] = true
	}

	value := s.node.valueMap[moduleKey][s.idxInValues]
	s.valueMap[moduleKey] = value
	return value, nil
}

func (s moduleDepResolver) addNode(p *simpleProvider, _ int) error {
	return duplicateDefinitionError(s.typ, p.provider.Location, s.node.provider.Location.String())
}

func (s moduleDepResolver) typeGraphNode() *graphviz.Node {
	return s.graphNode
}
