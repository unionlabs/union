package rapidproto

import (
	"fmt"
	"math"

	cosmos_proto "github.com/cosmos/cosmos-proto"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/reflect/protoreflect"
	"google.golang.org/protobuf/reflect/protoregistry"
	"gotest.tools/v3/assert"
	"pgregory.net/rapid"
)

func MessageGenerator[T proto.Message](x T, options GeneratorOptions) *rapid.Generator[T] {
	msgType := x.ProtoReflect().Type()
	return rapid.Custom(func(t *rapid.T) T {
		msg := msgType.New()

		options.setFields(t, nil, msg, 0)

		return msg.Interface().(T)
	})
}

// FieldMapper is a function that can be used to override the default behavior of the generator for a specific field.
// The first argument is the rapid.T, the second is the field descriptor, and the third is the field name.
// If the function returns nil, the default behavior will be used.
type FieldMapper func(*rapid.T, protoreflect.FieldDescriptor, string) (protoreflect.Value, bool)

type GeneratorOptions struct {
	AnyTypeURLs    []string
	InterfaceHints map[string]string
	Resolver       protoregistry.MessageTypeResolver

	// NoEmptyLists will cause the generator to not generate empty lists
	// Recall that an empty list will marshal (and unmarshal) to null. Some encodings may treat these states
	// differently.  For example, in JSON, an empty list is encoded as [], while null is encoded as null.
	NoEmptyLists bool

	// DisallowNilMessages will cause the generator to not generate nil messages to protoreflect.MessageKind fields
	DisallowNilMessages bool

	// FieldMaps is a list of FieldMapper functions that can be used to override the default behavior of the generator
	// for a specific field.
	FieldMaps []FieldMapper
}

const depthLimit = 10

func (opts GeneratorOptions) WithAnyTypes(anyTypes ...proto.Message) GeneratorOptions {
	for _, a := range anyTypes {
		opts.AnyTypeURLs = append(opts.AnyTypeURLs, fmt.Sprintf("/%s", a.ProtoReflect().Descriptor().FullName()))
	}
	return opts
}

func (opts GeneratorOptions) WithDisallowNil() GeneratorOptions {
	o := &opts
	o.DisallowNilMessages = true
	return *o
}

func (opts GeneratorOptions) WithInterfaceHint(i string, impl proto.Message) GeneratorOptions {
	if opts.InterfaceHints == nil {
		opts.InterfaceHints = make(map[string]string)
	}
	opts.InterfaceHints[i] = string(impl.ProtoReflect().Descriptor().FullName())
	return opts
}

func (opts GeneratorOptions) setFields(
	t *rapid.T, field protoreflect.FieldDescriptor, msg protoreflect.Message, depth int) bool {
	// to avoid stack overflow we limit the depth of nested messages
	if depth > depthLimit {
		return false
	}

	descriptor := msg.Descriptor()
	fullName := descriptor.FullName()
	switch fullName {
	case timestampFullName:
		opts.genTimestamp(t, msg)
		return true
	case durationFullName:
		opts.genDuration(t, msg)
		return true
	case anyFullName:
		opts.genAny(t, field, msg, depth)
		return true
	case fieldMaskFullName:
		opts.genFieldMask(t, msg)
		return true
	default:
		fields := descriptor.Fields()
		n := fields.Len()
		for i := 0; i < n; i++ {
			f := fields.Get(i)
			if !rapid.Bool().Draw(t, fmt.Sprintf("gen-%s", f.Name())) {
				if (f.Kind() == protoreflect.MessageKind) && !opts.DisallowNilMessages {
					continue
				}
			}

			opts.setFieldValue(t, msg, f, depth)
		}
		return true
	}
}

const (
	timestampFullName = "google.protobuf.Timestamp"
	durationFullName  = "google.protobuf.Duration"
	anyFullName       = "google.protobuf.Any"
	fieldMaskFullName = "google.protobuf.FieldMask"
)

func (opts GeneratorOptions) setFieldValue(t *rapid.T, msg protoreflect.Message, field protoreflect.FieldDescriptor, depth int) {
	name := string(field.Name())
	kind := field.Kind()

	switch {
	case field.IsList():
		list := msg.Mutable(field).List()
		min := 0
		if opts.NoEmptyLists {
			min = 1
		}
		n := rapid.IntRange(min, 10).Draw(t, fmt.Sprintf("%sN", name))
		for i := 0; i < n; i++ {
			if kind == protoreflect.MessageKind || kind == protoreflect.GroupKind {
				if !opts.setFields(t, field, list.AppendMutable().Message(), depth+1) {
					list.Truncate(i)
				}
			} else {
				list.Append(opts.genScalarFieldValue(t, field, fmt.Sprintf("%s%d", name, i)))
			}
		}
	case field.IsMap():
		m := msg.Mutable(field).Map()
		n := rapid.IntRange(0, 10).Draw(t, fmt.Sprintf("%sN", name))
		for i := 0; i < n; i++ {
			keyField := field.MapKey()
			valueField := field.MapValue()
			valueKind := valueField.Kind()
			key := opts.genScalarFieldValue(t, keyField, fmt.Sprintf("%s%d-key", name, i))
			if valueKind == protoreflect.MessageKind || valueKind == protoreflect.GroupKind {
				if !opts.setFields(t, field, m.Mutable(key.MapKey()).Message(), depth+1) {
					m.Clear(key.MapKey())
				}
			} else {
				value := opts.genScalarFieldValue(t, valueField, fmt.Sprintf("%s%d-key", name, i))
				m.Set(key.MapKey(), value)
			}
		}
	case kind == protoreflect.MessageKind:
		mutableField := msg.Mutable(field)
		if mutableField.Message().Descriptor().FullName() == anyFullName {
			if !opts.genAny(t, field, mutableField.Message(), depth+1) {
				msg.Clear(field)
			}
		} else if !opts.setFields(t, field, mutableField.Message(), depth+1) {
			msg.Clear(field)
		}
	case kind == protoreflect.GroupKind:
		if !opts.setFields(t, field, msg.Mutable(field).Message(), depth+1) {
			msg.Clear(field)
		}
	default:
		msg.Set(field, opts.genScalarFieldValue(t, field, name))
	}
}

func (opts GeneratorOptions) genScalarFieldValue(t *rapid.T, field protoreflect.FieldDescriptor, name string) protoreflect.Value {
	for _, fm := range opts.FieldMaps {
		if v, ok := fm(t, field, name); ok {
			return v
		}
	}

	switch field.Kind() {
	case protoreflect.Int32Kind, protoreflect.Sint32Kind, protoreflect.Sfixed32Kind:
		return protoreflect.ValueOfInt32(rapid.Int32().Draw(t, name))
	case protoreflect.Uint32Kind, protoreflect.Fixed32Kind:
		return protoreflect.ValueOfUint32(rapid.Uint32().Draw(t, name))
	case protoreflect.Int64Kind, protoreflect.Sint64Kind, protoreflect.Sfixed64Kind:
		return protoreflect.ValueOfInt64(rapid.Int64().Draw(t, name))
	case protoreflect.Uint64Kind, protoreflect.Fixed64Kind:
		return protoreflect.ValueOfUint64(rapid.Uint64().Draw(t, name))
	case protoreflect.BoolKind:
		return protoreflect.ValueOfBool(rapid.Bool().Draw(t, name))
	case protoreflect.BytesKind:
		return protoreflect.ValueOfBytes(rapid.SliceOf(rapid.Byte()).Draw(t, name))
	case protoreflect.FloatKind:
		return protoreflect.ValueOfFloat32(rapid.Float32().Draw(t, name))
	case protoreflect.DoubleKind:
		return protoreflect.ValueOfFloat64(rapid.Float64().Draw(t, name))
	case protoreflect.EnumKind:
		enumValues := field.Enum().Values()
		val := rapid.Int32Range(0, int32(enumValues.Len()-1)).Draw(t, name)
		return protoreflect.ValueOfEnum(protoreflect.EnumNumber(val))
	case protoreflect.StringKind:
		return protoreflect.ValueOfString(rapid.String().Draw(t, name))
	default:
		t.Fatalf("unexpected %v", field)
		return protoreflect.Value{}
	}
}

const (
	// MaxDurationSeconds the maximum number of seconds (when expressed as nanoseconds) which can fit in an int64.
	// gogoproto encodes google.protobuf.Duration as a time.Duration, which is 64-bit signed integer.
	MaxDurationSeconds = int64(math.MaxInt64/int(1e9)) - 1
	secondsName        = "seconds"
	nanosName          = "nanos"
)

func (opts GeneratorOptions) genTimestamp(t *rapid.T, msg protoreflect.Message) {
	seconds := rapid.Int64Range(-9999999999, 9999999999).Draw(t, "seconds")
	nanos := rapid.Int32Range(0, 999999999).Draw(t, "nanos")
	setSecondsNanosFields(t, msg, seconds, nanos)
}

func (opts GeneratorOptions) genDuration(t *rapid.T, msg protoreflect.Message) {
	seconds := rapid.Int64Range(0, int64(MaxDurationSeconds)).Draw(t, "seconds")
	nanos := rapid.Int32Range(0, 999999999).Draw(t, "nanos")
	setSecondsNanosFields(t, msg, seconds, nanos)
}

func setSecondsNanosFields(t *rapid.T, message protoreflect.Message, seconds int64, nanos int32) {
	fields := message.Descriptor().Fields()

	secondsField := fields.ByName(secondsName)
	assert.Assert(t, secondsField != nil)
	message.Set(secondsField, protoreflect.ValueOfInt64(seconds))

	nanosField := fields.ByName(nanosName)
	assert.Assert(t, nanosField != nil)
	message.Set(nanosField, protoreflect.ValueOfInt32(nanos))
}

const (
	typeURLName = "type_url"
	valueName   = "value"
)

func (opts GeneratorOptions) genAny(
	t *rapid.T, field protoreflect.FieldDescriptor, msg protoreflect.Message, depth int) bool {
	if len(opts.AnyTypeURLs) == 0 {
		return false
	}

	var typeURL string
	fopts := field.Options()
	if proto.HasExtension(fopts, cosmos_proto.E_AcceptsInterface) {
		ai := proto.GetExtension(fopts, cosmos_proto.E_AcceptsInterface).(string)
		if impl, found := opts.InterfaceHints[ai]; found {
			typeURL = fmt.Sprintf("/%s", impl)
		} else {
			panic(fmt.Sprintf("no implementation found for interface %s", ai))
		}
	} else {
		typeURL = rapid.SampledFrom(opts.AnyTypeURLs).Draw(t, "type_url")
	}

	typ, err := opts.Resolver.FindMessageByURL(typeURL)
	assert.NilError(t, err)
	fields := msg.Descriptor().Fields()

	typeURLField := fields.ByName(typeURLName)
	assert.Assert(t, typeURLField != nil)
	msg.Set(typeURLField, protoreflect.ValueOfString(typeURL))

	valueMsg := typ.New()
	opts.setFields(t, nil, valueMsg, depth+1)
	valueBz, err := proto.Marshal(valueMsg.Interface())
	assert.NilError(t, err)

	valueField := fields.ByName(valueName)
	assert.Assert(t, valueField != nil)
	msg.Set(valueField, protoreflect.ValueOfBytes(valueBz))

	return true
}

const (
	pathsName = "paths"
)

func (opts GeneratorOptions) genFieldMask(t *rapid.T, msg protoreflect.Message) {
	paths := rapid.SliceOfN(rapid.StringMatching("[a-z]+([.][a-z]+){0,2}"), 1, 5).Draw(t, "paths")
	pathsField := msg.Descriptor().Fields().ByName(pathsName)
	assert.Assert(t, pathsField != nil)
	pathsList := msg.NewField(pathsField).List()
	for _, path := range paths {
		pathsList.Append(protoreflect.ValueOfString(path))
	}
}
