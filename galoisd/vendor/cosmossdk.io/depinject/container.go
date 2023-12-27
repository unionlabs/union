package depinject

import (
	"bytes"
	stderrors "errors"
	"fmt"
	"reflect"

	"github.com/cockroachdb/errors"

	"cosmossdk.io/depinject/internal/graphviz"
)

type container struct {
	*debugConfig

	resolvers         map[string]resolver
	interfaceBindings map[string]interfaceBinding
	invokers          []invoker

	moduleKeyContext *ModuleKeyContext

	resolveStack []resolveFrame
	callerStack  []Location
	callerMap    map[Location]bool
}

type invoker struct {
	fn     *providerDescriptor
	modKey *moduleKey
}

type resolveFrame struct {
	loc Location
	typ reflect.Type
}

// interfaceBinding defines a type binding for interfaceName to type implTypeName when being provided as a
// dependency to the module identified by moduleKey.  If moduleKey is nil then the type binding is applied globally,
// not module-scoped.
type interfaceBinding struct {
	interfaceName string
	implTypeName  string
	moduleKey     *moduleKey
	resolver      resolver
}

func newContainer(cfg *debugConfig) *container {
	return &container{
		debugConfig:       cfg,
		resolvers:         map[string]resolver{},
		moduleKeyContext:  &ModuleKeyContext{},
		interfaceBindings: map[string]interfaceBinding{},
		callerStack:       nil,
		callerMap:         map[Location]bool{},
	}
}

func (c *container) call(provider *providerDescriptor, moduleKey *moduleKey) ([]reflect.Value, error) {
	loc := provider.Location
	graphNode := c.locationGraphNode(loc, moduleKey)

	markGraphNodeAsFailed(graphNode)

	if c.callerMap[loc] {
		return nil, errors.Errorf("cyclic dependency: %s -> %s", loc.Name(), loc.Name())
	}

	c.callerMap[loc] = true
	c.callerStack = append(c.callerStack, loc)

	c.logf("Resolving dependencies for %s", loc)
	c.indentLogger()
	inVals := make([]reflect.Value, len(provider.Inputs))
	for i, in := range provider.Inputs {
		val, err := c.resolve(in, moduleKey, loc)
		if err != nil {
			return nil, err
		}
		inVals[i] = val
	}
	c.dedentLogger()
	c.logf("Calling %s", loc)

	delete(c.callerMap, loc)
	c.callerStack = c.callerStack[0 : len(c.callerStack)-1]

	out, err := provider.Fn(inVals)
	if err != nil {
		return nil, errors.Wrapf(err, "error calling provider %s", loc)
	}

	markGraphNodeAsUsed(graphNode)

	return out, nil
}

func (c *container) getResolver(typ reflect.Type, key *moduleKey) (resolver, error) {
	pr, err := c.getExplicitResolver(typ, key)
	if err != nil {
		return nil, err
	}
	if pr != nil {
		return pr, nil
	}

	if vr, ok := c.resolverByType(typ); ok {
		return vr, nil
	}

	elemType := typ
	if isManyPerContainerSliceType(elemType) || isOnePerModuleMapType(elemType) {
		elemType = elemType.Elem()
	}

	var typeGraphNode *graphviz.Node

	if isManyPerContainerType(elemType) {
		c.logf("Registering resolver for many-per-container type %v", elemType)
		sliceType := reflect.SliceOf(elemType)

		typeGraphNode = c.typeGraphNode(sliceType)
		typeGraphNode.SetComment("many-per-container")

		r := &groupResolver{
			typ:       elemType,
			sliceType: sliceType,
			graphNode: typeGraphNode,
		}

		c.addResolver(elemType, r)
		c.addResolver(sliceType, &sliceGroupResolver{r})
	} else if isOnePerModuleType(elemType) {
		c.logf("Registering resolver for one-per-module type %v", elemType)
		mapType := reflect.MapOf(stringType, elemType)

		typeGraphNode = c.typeGraphNode(mapType)
		typeGraphNode.SetComment("one-per-module")

		r := &onePerModuleResolver{
			typ:       elemType,
			mapType:   mapType,
			providers: map[*moduleKey]*simpleProvider{},
			idxMap:    map[*moduleKey]int{},
			graphNode: typeGraphNode,
		}

		c.addResolver(elemType, r)
		c.addResolver(mapType, &mapOfOnePerModuleResolver{r})
	}

	res, found := c.resolverByType(typ)

	if !found && typ.Kind() == reflect.Interface {
		matches := map[reflect.Type]reflect.Type{}
		var resolverType reflect.Type
		for _, r := range c.resolvers {
			if r.getType().Kind() != reflect.Interface && r.getType().Implements(typ) {
				resolverType = r.getType()
				matches[resolverType] = resolverType
			}
		}

		if len(matches) == 1 {
			res, _ = c.resolverByType(resolverType)
			c.logf("Implicitly registering resolver %v for interface type %v", resolverType, typ)
			c.addResolver(typ, res)
		} else if len(matches) > 1 {
			return nil, newErrMultipleImplicitInterfaceBindings(typ, matches)
		}
	}

	return res, nil
}

func (c *container) getExplicitResolver(typ reflect.Type, key *moduleKey) (resolver, error) {
	var pref interfaceBinding
	var found bool

	// module scoped binding takes precedence
	pref, found = c.interfaceBindings[bindingKeyFromType(typ, key)]

	// fallback to global scope binding
	if !found {
		pref, found = c.interfaceBindings[bindingKeyFromType(typ, nil)]
	}

	if !found {
		return nil, nil
	}

	if pref.resolver != nil {
		return pref.resolver, nil
	}

	res, ok := c.resolverByTypeName(pref.implTypeName)
	if ok {
		c.logf("Registering resolver %v for interface type %v by explicit binding", res.getType(), typ)
		pref.resolver = res
		return res, nil

	}

	return nil, newErrNoTypeForExplicitBindingFound(pref)
}

var stringType = reflect.TypeOf("")

func (c *container) addNode(provider *providerDescriptor, key *moduleKey) (interface{}, error) {
	providerGraphNode := c.locationGraphNode(provider.Location, key)
	hasModuleKeyParam := false
	hasOwnModuleKeyParam := false
	for _, in := range provider.Inputs {
		typ := in.Type
		if typ == moduleKeyType {
			hasModuleKeyParam = true
		}

		if typ == ownModuleKeyType {
			hasOwnModuleKeyParam = true
		}

		if isManyPerContainerType(typ) {
			return nil, fmt.Errorf("many-per-container type %v can't be used as an input parameter", typ)
		} else if isOnePerModuleType(typ) {
			return nil, fmt.Errorf("one-per-module type %v can't be used as an input parameter", typ)
		}

		vr, err := c.getResolver(typ, key)
		if err != nil {
			return nil, err
		}

		var typeGraphNode *graphviz.Node
		if vr != nil {
			typeGraphNode = vr.typeGraphNode()
		} else {
			typeGraphNode = c.typeGraphNode(typ)
			if err != nil {
				return nil, err
			}
		}

		c.addGraphEdge(typeGraphNode, providerGraphNode)
	}

	if !hasModuleKeyParam {
		c.logf("Registering %s", provider.Location.String())
		c.indentLogger()
		defer c.dedentLogger()

		sp := &simpleProvider{
			provider:  provider,
			moduleKey: key,
		}

		for i, out := range provider.Outputs {
			typ := out.Type

			// one-per-module maps can't be used as a return type
			if isOnePerModuleMapType(typ) {
				return nil, fmt.Errorf("%v cannot be used as a return type because %v is a one-per-module type",
					typ, typ.Elem())
			}

			// many-per-container slices of many-per-container types
			if isManyPerContainerSliceType(typ) {
				typ = typ.Elem()
			}

			vr, err := c.getResolver(typ, key)
			if err != nil {
				return nil, err
			}

			if vr != nil {
				c.logf("Found resolver for %v: %T", typ, vr)
				err := vr.addNode(sp, i)
				if err != nil {
					return nil, err
				}
			} else {
				c.logf("Registering resolver for simple type %v", typ)

				typeGraphNode := c.typeGraphNode(typ)
				vr = &simpleResolver{
					node:        sp,
					typ:         typ,
					graphNode:   typeGraphNode,
					idxInValues: i,
				}
				c.addResolver(typ, vr)
			}

			c.addGraphEdge(providerGraphNode, vr.typeGraphNode())
		}

		return sp, nil
	}

	if hasOwnModuleKeyParam {
		return nil, errors.Errorf("%T and %T must not be declared as dependencies on the same provided",
			ModuleKey{}, OwnModuleKey{})
	}

	c.logf("Registering module-scoped provider: %s", provider.Location.String())
	c.indentLogger()
	defer c.dedentLogger()

	node := &moduleDepProvider{
		provider:        provider,
		calledForModule: map[*moduleKey]bool{},
		valueMap:        map[*moduleKey][]reflect.Value{},
	}

	for i, out := range provider.Outputs {
		typ := out.Type

		c.logf("Registering resolver for module-scoped type %v", typ)

		existing, ok := c.resolverByType(typ)
		if ok {
			return nil, errors.Errorf("duplicate provision of type %v by module-scoped provider %s\n\talready provided by %s",
				typ, provider.Location, existing.describeLocation())
		}

		typeGraphNode := c.typeGraphNode(typ)
		c.addResolver(typ, &moduleDepResolver{
			typ:         typ,
			idxInValues: i,
			node:        node,
			valueMap:    map[*moduleKey]reflect.Value{},
			graphNode:   typeGraphNode,
		})

		c.addGraphEdge(providerGraphNode, typeGraphNode)
	}

	return node, nil
}

func (c *container) supply(value reflect.Value, location Location) error {
	typ := value.Type()
	locGrapNode := c.locationGraphNode(location, nil)
	markGraphNodeAsUsed(locGrapNode)
	typeGraphNode := c.typeGraphNode(typ)
	c.addGraphEdge(locGrapNode, typeGraphNode)

	if existing, ok := c.resolverByType(typ); ok {
		return duplicateDefinitionError(typ, location, existing.describeLocation())
	}

	c.addResolver(typ, &supplyResolver{
		typ:       typ,
		value:     value,
		loc:       location,
		graphNode: typeGraphNode,
	})

	return nil
}

func (c *container) addInvoker(provider *providerDescriptor, key *moduleKey) error {
	// make sure there are no outputs
	if len(provider.Outputs) > 0 {
		return fmt.Errorf("invoker function %s should not return any outputs", provider.Location)
	}

	c.invokers = append(c.invokers, invoker{
		fn:     provider,
		modKey: key,
	})

	return nil
}

func (c *container) resolve(in providerInput, moduleKey *moduleKey, caller Location) (reflect.Value, error) {
	c.resolveStack = append(c.resolveStack, resolveFrame{loc: caller, typ: in.Type})

	typeGraphNode := c.typeGraphNode(in.Type)

	if in.Type == moduleKeyType {
		if moduleKey == nil {
			return reflect.Value{}, errors.Errorf("trying to resolve %T for %s but not inside of any module's scope", moduleKey, caller)
		}
		c.logf("Providing ModuleKey %s", moduleKey.name)
		markGraphNodeAsUsed(typeGraphNode)
		return reflect.ValueOf(ModuleKey{moduleKey}), nil
	}

	if in.Type == ownModuleKeyType {
		if moduleKey == nil {
			return reflect.Value{}, errors.Errorf("trying to resolve %T for %s but not inside of any module's scope", moduleKey, caller)
		}
		c.logf("Providing OwnModuleKey %s", moduleKey.name)
		markGraphNodeAsUsed(typeGraphNode)
		return reflect.ValueOf(OwnModuleKey{moduleKey}), nil
	}

	vr, err := c.getResolver(in.Type, moduleKey)
	if err != nil {
		return reflect.Value{}, err
	}

	if vr == nil {
		if in.Optional {
			c.logf("Providing zero value for optional dependency %v", in.Type)
			return reflect.Zero(in.Type), nil
		}

		markGraphNodeAsFailed(typeGraphNode)
		return reflect.Value{}, errors.Errorf("can't resolve type %v for %s:\n%s",
			fullyQualifiedTypeName(in.Type), caller, c.formatResolveStack())
	}

	res, err := vr.resolve(c, moduleKey, caller)
	if err != nil {
		markGraphNodeAsFailed(typeGraphNode)
		return reflect.Value{}, err
	}

	markGraphNodeAsUsed(typeGraphNode)

	c.resolveStack = c.resolveStack[:len(c.resolveStack)-1]

	return res, nil
}

func (c *container) build(loc Location, outputs ...interface{}) error {
	var providerIn []providerInput
	for _, output := range outputs {
		typ := reflect.TypeOf(output)
		if typ.Kind() != reflect.Pointer {
			return fmt.Errorf("output type must be a pointer, %s is invalid", typ)
		}

		providerIn = append(providerIn, providerInput{Type: typ.Elem()})
	}

	desc := providerDescriptor{
		Inputs:  providerIn,
		Outputs: nil,
		Fn: func(values []reflect.Value) ([]reflect.Value, error) {
			if len(values) != len(outputs) {
				return nil, stderrors.New("internal error, unexpected number of values")
			}

			for i, output := range outputs {
				val := reflect.ValueOf(output)

				if !values[i].CanInterface() {
					return []reflect.Value{}, fmt.Errorf("depinject.Out struct %s on package can't have unexported field", values[i].String())
				}
				val.Elem().Set(values[i])
			}

			return nil, nil
		},
		Location: loc,
	}
	callerGraphNode := c.locationGraphNode(loc, nil)
	callerGraphNode.SetShape("hexagon")

	desc, err := expandStructArgsProvider(desc)
	if err != nil {
		return err
	}

	c.logf("Registering outputs")
	c.indentLogger()

	node, err := c.addNode(&desc, nil)
	if err != nil {
		return err
	}

	c.dedentLogger()

	sn, ok := node.(*simpleProvider)
	if !ok {
		return errors.Errorf("cannot run module-scoped provider as an invoker")
	}

	c.logf("Building container")
	_, err = sn.resolveValues(c)
	if err != nil {
		return err
	}
	c.logf("Done building container")
	c.logf("Calling invokers")
	for _, inv := range c.invokers {
		_, err := c.call(inv.fn, inv.modKey)
		if err != nil {
			return err
		}
	}
	c.logf("Done calling invokers")

	return nil
}

func (c container) formatResolveStack() string {
	buf := &bytes.Buffer{}
	_, _ = fmt.Fprintf(buf, "\twhile resolving:\n")
	n := len(c.resolveStack)
	for i := n - 1; i >= 0; i-- {
		rk := c.resolveStack[i]
		_, _ = fmt.Fprintf(buf, "\t\t%v for %s\n", rk.typ, rk.loc)
	}
	return buf.String()
}

func fullyQualifiedTypeName(typ reflect.Type) string {
	pkgType := typ
	if typ.Kind() == reflect.Pointer || typ.Kind() == reflect.Slice || typ.Kind() == reflect.Map || typ.Kind() == reflect.Array {
		pkgType = typ.Elem()
	}
	pkgPath := pkgType.PkgPath()
	if pkgPath == "" {
		return fmt.Sprintf("%v", typ)
	}

	return fmt.Sprintf("%s/%v", pkgPath, typ)
}

func bindingKeyFromTypeName(typeName string, key *moduleKey) string {
	if key == nil {
		return fmt.Sprintf("%s;", typeName)
	}
	return fmt.Sprintf("%s;%s", typeName, key.name)
}

func bindingKeyFromType(typ reflect.Type, key *moduleKey) string {
	return bindingKeyFromTypeName(fullyQualifiedTypeName(typ), key)
}

func (c *container) addBinding(p interfaceBinding) {
	c.interfaceBindings[bindingKeyFromTypeName(p.interfaceName, p.moduleKey)] = p
}

func (c *container) addResolver(typ reflect.Type, r resolver) {
	c.resolvers[fullyQualifiedTypeName(typ)] = r
}

func (c *container) resolverByType(typ reflect.Type) (resolver, bool) {
	return c.resolverByTypeName(fullyQualifiedTypeName(typ))
}

func (c *container) resolverByTypeName(typeName string) (resolver, bool) {
	res, found := c.resolvers[typeName]
	return res, found
}

func markGraphNodeAsUsed(node *graphviz.Node) {
	node.SetColor("black")
	node.SetPenWidth("1.5")
	node.SetFontColor("black")
}

func markGraphNodeAsFailed(node *graphviz.Node) {
	node.SetColor("red")
	node.SetFontColor("red")
}
