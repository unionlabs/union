package depinject

import (
	"reflect"

	"cosmossdk.io/depinject/internal/graphviz"
)

type resolver interface {
	addNode(*simpleProvider, int) error
	resolve(*container, *moduleKey, Location) (reflect.Value, error)
	describeLocation() string
	typeGraphNode() *graphviz.Node
	getType() reflect.Type
}
