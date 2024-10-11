package textual_test

import (
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
)

var _ protoreflect.List = (*genericList[proto.Message])(nil)

// NewGenericList creates a empty list that satisfies the protoreflect.List
// interface.
func NewGenericList[T proto.Message](list []T) protoreflect.List {
	return &genericList[T]{&list}
}

// genericList is an implementation of protoreflect.List for a generic
type genericList[T proto.Message] struct {
	list *[]T
}

func (x *genericList[T]) Len() int {
	if x.list == nil {
		return 0
	}
	return len(*x.list)
}

func (x *genericList[T]) Get(i int) protoreflect.Value {
	if x.Len() == 0 {
		return protoreflect.Value{}
	}
	return protoreflect.ValueOfMessage((*x.list)[i].ProtoReflect())
}

func (x *genericList[T]) Set(i int, value protoreflect.Value) {
	valueUnwrapped := value.Message()
	concreteValue := valueUnwrapped.Interface().(T)
	(*x.list)[i] = concreteValue
}

func (x *genericList[T]) Append(value protoreflect.Value) {
	valueUnwrapped := value.Message()
	concreteValue := valueUnwrapped.Interface().(T)
	*x.list = append(*x.list, concreteValue)
}

func (x *genericList[T]) AppendMutable() protoreflect.Value {
	v := *new(T)
	*x.list = append(*x.list, v)
	return protoreflect.ValueOfMessage(v.ProtoReflect())
}

func (x *genericList[T]) Truncate(n int) {
	for i := n; i < len(*x.list); i++ {
		(*x.list)[i] = *new(T)
	}
	*x.list = (*x.list)[:n]
}

func (x *genericList[T]) NewElement() protoreflect.Value {
	v := *new(T)
	return protoreflect.ValueOfMessage(v.ProtoReflect())
}

func (x *genericList[T]) IsValid() bool {
	return x.list != nil
}
