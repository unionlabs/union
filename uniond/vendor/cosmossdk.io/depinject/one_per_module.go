package depinject

import (
	"fmt"
	"reflect"

	"cosmossdk.io/depinject/internal/graphviz"
)

// OnePerModuleType marks a type which
// can have up to one value per module. All of the values for a one-per-module type T
// and their respective modules, can be retrieved by declaring an input parameter map[string]T.
type OnePerModuleType interface {
	// IsOnePerModuleType is a marker function just indicates that this is a one-per-module type.
	IsOnePerModuleType()
}

var onePerModuleTypeType = reflect.TypeOf((*OnePerModuleType)(nil)).Elem()

func isOnePerModuleType(t reflect.Type) bool {
	return t.Implements(onePerModuleTypeType)
}

func isOnePerModuleMapType(typ reflect.Type) bool {
	return typ.Kind() == reflect.Map && isOnePerModuleType(typ.Elem()) && typ.Key().Kind() == reflect.String
}

type onePerModuleResolver struct {
	typ       reflect.Type
	mapType   reflect.Type
	providers map[*moduleKey]*simpleProvider
	idxMap    map[*moduleKey]int
	resolved  bool
	values    reflect.Value
	graphNode *graphviz.Node
}

func (o *onePerModuleResolver) getType() reflect.Type {
	return o.mapType
}

type mapOfOnePerModuleResolver struct {
	*onePerModuleResolver
}

func (o *onePerModuleResolver) resolve(_ *container, _ *moduleKey, _ Location) (reflect.Value, error) {
	return reflect.Value{}, fmt.Errorf("%v is a one-per-module type and thus can't be used as an input parameter, instead use %v", o.typ, o.mapType)
}

func (o *onePerModuleResolver) describeLocation() string {
	return fmt.Sprintf("one-per-module type %v", o.typ)
}

func (o *mapOfOnePerModuleResolver) resolve(c *container, _ *moduleKey, caller Location) (reflect.Value, error) {
	// Log
	c.logf("Providing one-per-module type map %v to %s from:", o.mapType, caller.Name())
	c.indentLogger()
	for key, node := range o.providers {
		c.logf("%s: %s", key.name, node.provider.Location)
	}
	c.dedentLogger()

	// Resolve
	if !o.resolved {
		res := reflect.MakeMap(o.mapType)
		for key, node := range o.providers {
			values, err := node.resolveValues(c)
			if err != nil {
				return reflect.Value{}, err
			}
			idx := o.idxMap[key]
			if len(values) <= idx {
				return reflect.Value{}, fmt.Errorf("expected value of type %T at index %d", o.typ, idx)
			}
			value := values[idx]
			res.SetMapIndex(reflect.ValueOf(key.name), value)
		}

		o.values = res
		o.resolved = true
	}

	return o.values, nil
}

func (o *onePerModuleResolver) addNode(n *simpleProvider, i int) error {
	if n.moduleKey == nil {
		return fmt.Errorf("cannot define a provider with one-per-module dependency %v which isn't provided in a module", o.typ)
	}

	if existing, ok := o.providers[n.moduleKey]; ok {
		return fmt.Errorf("duplicate provision for one-per-module type %v in module %s: %s\n\talready provided by %s",
			o.typ, n.moduleKey.name, n.provider.Location, existing.provider.Location)
	}

	o.providers[n.moduleKey] = n
	o.idxMap[n.moduleKey] = i

	return nil
}

func (o *mapOfOnePerModuleResolver) addNode(s *simpleProvider, _ int) error {
	return fmt.Errorf("%v is a one-per-module type and thus %v can't be used as an output parameter in %s", o.typ, o.mapType, s.provider.Location)
}

func (o onePerModuleResolver) typeGraphNode() *graphviz.Node {
	return o.graphNode
}
