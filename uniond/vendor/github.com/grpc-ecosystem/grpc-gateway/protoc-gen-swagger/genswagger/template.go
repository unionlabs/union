package genswagger

import (
	"bytes"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"math"
	"net/textproto"
	"os"
	"reflect"
	"regexp"
	"sort"
	"strconv"
	"strings"
	"sync"
	"text/template"
	"time"

	"github.com/golang/glog"
	"github.com/golang/protobuf/jsonpb"
	"github.com/golang/protobuf/proto"
	pbdescriptor "github.com/golang/protobuf/protoc-gen-go/descriptor"
	structpb "github.com/golang/protobuf/ptypes/struct"
	"github.com/grpc-ecosystem/grpc-gateway/internal/casing"
	"github.com/grpc-ecosystem/grpc-gateway/protoc-gen-grpc-gateway/descriptor"
	swagger_options "github.com/grpc-ecosystem/grpc-gateway/protoc-gen-swagger/options"
)

var wktSchemas = map[string]schemaCore{
	".google.protobuf.Timestamp": schemaCore{
		Type:   "string",
		Format: "date-time",
	},
	".google.protobuf.Duration": schemaCore{
		Type: "string",
	},
	".google.protobuf.StringValue": schemaCore{
		Type: "string",
	},
	".google.protobuf.BytesValue": schemaCore{
		Type:   "string",
		Format: "byte",
	},
	".google.protobuf.Int32Value": schemaCore{
		Type:   "integer",
		Format: "int32",
	},
	".google.protobuf.UInt32Value": schemaCore{
		Type:   "integer",
		Format: "int64",
	},
	".google.protobuf.Int64Value": schemaCore{
		Type:   "string",
		Format: "int64",
	},
	".google.protobuf.UInt64Value": schemaCore{
		Type:   "string",
		Format: "uint64",
	},
	".google.protobuf.FloatValue": schemaCore{
		Type:   "number",
		Format: "float",
	},
	".google.protobuf.DoubleValue": schemaCore{
		Type:   "number",
		Format: "double",
	},
	".google.protobuf.BoolValue": schemaCore{
		Type: "boolean",
	},
	".google.protobuf.Empty": schemaCore{},
	".google.protobuf.Struct": schemaCore{
		Type: "object",
	},
	".google.protobuf.Value": schemaCore{
		Type: "object",
	},
	".google.protobuf.ListValue": schemaCore{
		Type: "array",
		Items: (*swaggerItemsObject)(&schemaCore{
			Type: "object",
		}),
	},
	".google.protobuf.NullValue": schemaCore{
		Type: "string",
	},
}

func listEnumNames(enum *descriptor.Enum) (names []string) {
	for _, value := range enum.GetValue() {
		names = append(names, value.GetName())
	}
	return names
}

func listEnumNumbers(enum *descriptor.Enum) (numbers []string) {
	for _, value := range enum.GetValue() {
		numbers = append(numbers, strconv.Itoa(int(value.GetNumber())))
	}
	return
}

func getEnumDefault(enum *descriptor.Enum) string {
	for _, value := range enum.GetValue() {
		if value.GetNumber() == 0 {
			return value.GetName()
		}
	}
	return ""
}

// messageToQueryParameters converts a message to a list of swagger query parameters.
func messageToQueryParameters(message *descriptor.Message, reg *descriptor.Registry, pathParams []descriptor.Parameter, body *descriptor.Body) (params []swaggerParameterObject, err error) {
	for _, field := range message.Fields {
		p, err := queryParams(message, field, "", reg, pathParams, body)
		if err != nil {
			return nil, err
		}
		params = append(params, p...)
	}
	return params, nil
}

// queryParams converts a field to a list of swagger query parameters recursively through the use of nestedQueryParams.
func queryParams(message *descriptor.Message, field *descriptor.Field, prefix string, reg *descriptor.Registry, pathParams []descriptor.Parameter, body *descriptor.Body) (params []swaggerParameterObject, err error) {
	return nestedQueryParams(message, field, prefix, reg, pathParams, body, map[string]bool{})
}

// nestedQueryParams converts a field to a list of swagger query parameters recursively.
// This function is a helper function for queryParams, that keeps track of cyclical message references
//  through the use of
//      touched map[string]bool
// If a cycle is discovered, an error is returned, as cyclical data structures aren't allowed
//  in query parameters.
func nestedQueryParams(message *descriptor.Message, field *descriptor.Field, prefix string, reg *descriptor.Registry, pathParams []descriptor.Parameter, body *descriptor.Body, touchedIn map[string]bool) (params []swaggerParameterObject, err error) {
	// make sure the parameter is not already listed as a path parameter
	for _, pathParam := range pathParams {
		if pathParam.Target == field {
			return nil, nil
		}
	}
	// make sure the parameter is not already listed as a body parameter
	if body != nil {
		if body.FieldPath == nil {
			return nil, nil
		}
		for _, fieldPath := range body.FieldPath {
			if fieldPath.Target == field {
				return nil, nil
			}
		}
	}
	schema := schemaOfField(field, reg, nil)
	fieldType := field.GetTypeName()
	if message.File != nil {
		comments := fieldProtoComments(reg, message, field)
		if err := updateSwaggerDataFromComments(reg, &schema, message, comments, false); err != nil {
			return nil, err
		}
	}

	isEnum := field.GetType() == pbdescriptor.FieldDescriptorProto_TYPE_ENUM
	items := schema.Items
	if schema.Type != "" || isEnum {
		if schema.Type == "object" {
			return nil, nil // TODO: currently, mapping object in query parameter is not supported
		}
		if items != nil && (items.Type == "" || items.Type == "object") && !isEnum {
			return nil, nil // TODO: currently, mapping object in query parameter is not supported
		}
		desc := schema.Description
		if schema.Title != "" { // merge title because title of parameter object will be ignored
			desc = strings.TrimSpace(schema.Title + ". " + schema.Description)
		}

		// verify if the field is required
		required := false
		for _, fieldName := range schema.Required {
			if fieldName == field.GetName() {
				required = true
				break
			}
		}

		param := swaggerParameterObject{
			Description: desc,
			In:          "query",
			Default:     schema.Default,
			Type:        schema.Type,
			Items:       schema.Items,
			Format:      schema.Format,
			Required:    required,
		}
		if param.Type == "array" {
			param.CollectionFormat = "multi"
		}

		if reg.GetUseJSONNamesForFields() {
			param.Name = prefix + field.GetJsonName()
		} else {
			param.Name = prefix + field.GetName()
		}

		if isEnum {
			enum, err := reg.LookupEnum("", fieldType)
			if err != nil {
				return nil, fmt.Errorf("unknown enum type %s", fieldType)
			}
			if items != nil { // array
				param.Items = &swaggerItemsObject{
					Type: "string",
					Enum: listEnumNames(enum),
				}
				if reg.GetEnumsAsInts() {
					param.Items.Type = "integer"
					param.Items.Enum = listEnumNumbers(enum)
				}
			} else {
				param.Type = "string"
				param.Enum = listEnumNames(enum)
				param.Default = getEnumDefault(enum)
				if reg.GetEnumsAsInts() {
					param.Type = "integer"
					param.Enum = listEnumNumbers(enum)
					param.Default = "0"
				}
			}
			valueComments := enumValueProtoComments(reg, enum)
			if valueComments != "" {
				param.Description = strings.TrimLeft(param.Description+"\n\n "+valueComments, "\n")
			}
		}
		return []swaggerParameterObject{param}, nil
	}

	// nested type, recurse
	msg, err := reg.LookupMsg("", fieldType)
	if err != nil {
		return nil, fmt.Errorf("unknown message type %s", fieldType)
	}

	// Check for cyclical message reference:
	isCycle := touchedIn[*msg.Name]
	if isCycle {
		return nil, fmt.Errorf("Recursive types are not allowed for query parameters, cycle found on %q", fieldType)
	}

	// Construct a new map with the message name so a cycle further down the recursive path can be detected.
	// Do not keep anything in the original touched reference and do not pass that reference along.  This will
	// prevent clobbering adjacent records while recursing.
	touchedOut := make(map[string]bool)
	for k, v := range touchedIn {
		touchedOut[k] = v
	}
	touchedOut[*msg.Name] = true

	for _, nestedField := range msg.Fields {
		var fieldName string
		if reg.GetUseJSONNamesForFields() {
			fieldName = field.GetJsonName()
		} else {
			fieldName = field.GetName()
		}
		p, err := nestedQueryParams(msg, nestedField, prefix+fieldName+".", reg, pathParams, body, touchedOut)
		if err != nil {
			return nil, err
		}
		params = append(params, p...)
	}
	return params, nil
}

// findServicesMessagesAndEnumerations discovers all messages and enums defined in the RPC methods of the service.
func findServicesMessagesAndEnumerations(s []*descriptor.Service, reg *descriptor.Registry, m messageMap, ms messageMap, e enumMap, refs refMap) {
	for _, svc := range s {
		for _, meth := range svc.Methods {
			// Request may be fully included in query
			{
				swgReqName, ok := fullyQualifiedNameToSwaggerName(meth.RequestType.FQMN(), reg)
				if !ok {
					glog.Errorf("couldn't resolve swagger name for FQMN '%v'", meth.RequestType.FQMN())
					continue
				}
				if _, ok := refs[fmt.Sprintf("#/definitions/%s", swgReqName)]; ok {
					if !skipRenderingRef(meth.RequestType.FQMN()) {
						m[swgReqName] = meth.RequestType
					}
				}
			}

			swgRspName, ok := fullyQualifiedNameToSwaggerName(meth.ResponseType.FQMN(), reg)
			if !ok && !skipRenderingRef(meth.ResponseType.FQMN()) {
				glog.Errorf("couldn't resolve swagger name for FQMN '%v'", meth.ResponseType.FQMN())
				continue
			}

			findNestedMessagesAndEnumerations(meth.RequestType, reg, m, e)

			if !skipRenderingRef(meth.ResponseType.FQMN()) {
				m[swgRspName] = meth.ResponseType
				if meth.GetServerStreaming() {
					streamError, runtimeStreamError, err := lookupMsgAndSwaggerName(".grpc.gateway.runtime", "StreamError", reg)
					if err != nil {
						glog.Error(err)
					} else {
						glog.V(1).Infof("StreamError: %v", streamError)
						glog.V(1).Infof("StreamError FQMN: %s", runtimeStreamError)
						m[runtimeStreamError] = streamError
						findNestedMessagesAndEnumerations(streamError, reg, m, e)
					}
					ms[swgRspName] = meth.ResponseType
				}
			}
			findNestedMessagesAndEnumerations(meth.ResponseType, reg, m, e)
		}
	}
}

// findNestedMessagesAndEnumerations those can be generated by the services.
func findNestedMessagesAndEnumerations(message *descriptor.Message, reg *descriptor.Registry, m messageMap, e enumMap) {
	// Iterate over all the fields that
	for _, t := range message.Fields {
		fieldType := t.GetTypeName()
		// If the type is an empty string then it is a proto primitive
		if fieldType != "" {
			if _, ok := m[fieldType]; !ok {
				msg, err := reg.LookupMsg("", fieldType)
				if err != nil {
					enum, err := reg.LookupEnum("", fieldType)
					if err != nil {
						panic(err)
					}
					e[fieldType] = enum
					continue
				}
				m[fieldType] = msg
				findNestedMessagesAndEnumerations(msg, reg, m, e)
			}
		}
	}
}

func skipRenderingRef(refName string) bool {
	_, ok := wktSchemas[refName]
	return ok
}

func renderMessagesAsDefinition(messages messageMap, d swaggerDefinitionsObject, reg *descriptor.Registry, customRefs refMap) {
	for name, msg := range messages {
		swgName, ok := fullyQualifiedNameToSwaggerName(msg.FQMN(), reg)
		if !ok {
			panic(fmt.Sprintf("can't resolve swagger name from '%v'", msg.FQMN()))
		}
		if skipRenderingRef(name) {
			continue
		}

		if opt := msg.GetOptions(); opt != nil && opt.MapEntry != nil && *opt.MapEntry {
			continue
		}
		schema := swaggerSchemaObject{
			schemaCore: schemaCore{
				Type: "object",
			},
		}
		msgComments := protoComments(reg, msg.File, msg.Outers, "MessageType", int32(msg.Index))
		if err := updateSwaggerDataFromComments(reg, &schema, msg, msgComments, false); err != nil {
			panic(err)
		}
		opts, err := extractSchemaOptionFromMessageDescriptor(msg.DescriptorProto)
		if err != nil {
			panic(err)
		}
		if opts != nil {
			protoSchema := swaggerSchemaFromProtoSchema(opts, reg, customRefs, msg)

			// Warning: Make sure not to overwrite any fields already set on the schema type.
			schema.ExternalDocs = protoSchema.ExternalDocs
			schema.ReadOnly = protoSchema.ReadOnly
			schema.MultipleOf = protoSchema.MultipleOf
			schema.Maximum = protoSchema.Maximum
			schema.ExclusiveMaximum = protoSchema.ExclusiveMaximum
			schema.Minimum = protoSchema.Minimum
			schema.ExclusiveMinimum = protoSchema.ExclusiveMinimum
			schema.MaxLength = protoSchema.MaxLength
			schema.MinLength = protoSchema.MinLength
			schema.Pattern = protoSchema.Pattern
			schema.Default = protoSchema.Default
			schema.MaxItems = protoSchema.MaxItems
			schema.MinItems = protoSchema.MinItems
			schema.UniqueItems = protoSchema.UniqueItems
			schema.MaxProperties = protoSchema.MaxProperties
			schema.MinProperties = protoSchema.MinProperties
			schema.Required = protoSchema.Required
			if protoSchema.schemaCore.Type != "" || protoSchema.schemaCore.Ref != "" {
				schema.schemaCore = protoSchema.schemaCore
			}
			if protoSchema.Title != "" {
				schema.Title = protoSchema.Title
			}
			if protoSchema.Description != "" {
				schema.Description = protoSchema.Description
			}
			if protoSchema.Example != nil {
				schema.Example = protoSchema.Example
			}
		}

		for _, f := range msg.Fields {
			fieldValue := schemaOfField(f, reg, customRefs)
			comments := fieldProtoComments(reg, msg, f)
			if err := updateSwaggerDataFromComments(reg, &fieldValue, f, comments, false); err != nil {
				panic(err)
			}

			kv := keyVal{Value: fieldValue}
			if reg.GetUseJSONNamesForFields() {
				kv.Key = f.GetJsonName()
			} else {
				kv.Key = f.GetName()
			}
			if schema.Properties == nil {
				schema.Properties = &swaggerSchemaObjectProperties{}
			}
			*schema.Properties = append(*schema.Properties, kv)
		}
		d[swgName] = schema
	}
}

// schemaOfField returns a swagger Schema Object for a protobuf field.
func schemaOfField(f *descriptor.Field, reg *descriptor.Registry, refs refMap) swaggerSchemaObject {
	const (
		singular = 0
		array    = 1
		object   = 2
	)
	var (
		core      schemaCore
		aggregate int
	)

	fd := f.FieldDescriptorProto
	if m, err := reg.LookupMsg("", f.GetTypeName()); err == nil {
		if opt := m.GetOptions(); opt != nil && opt.MapEntry != nil && *opt.MapEntry {
			fd = m.GetField()[1]
			aggregate = object
		}
	}
	if fd.GetLabel() == pbdescriptor.FieldDescriptorProto_LABEL_REPEATED {
		aggregate = array
	}

	var props *swaggerSchemaObjectProperties

	switch ft := fd.GetType(); ft {
	case pbdescriptor.FieldDescriptorProto_TYPE_ENUM, pbdescriptor.FieldDescriptorProto_TYPE_MESSAGE, pbdescriptor.FieldDescriptorProto_TYPE_GROUP:
		if wktSchema, ok := wktSchemas[fd.GetTypeName()]; ok {
			core = wktSchema

			if fd.GetTypeName() == ".google.protobuf.Empty" {
				props = &swaggerSchemaObjectProperties{}
			}
		} else {
			swgRef, ok := fullyQualifiedNameToSwaggerName(fd.GetTypeName(), reg)
			if !ok {
				panic(fmt.Sprintf("can't resolve swagger ref from typename '%v'", fd.GetTypeName()))
			}
			core = schemaCore{
				Ref: "#/definitions/" + swgRef,
			}
			if refs != nil {
				refs[fd.GetTypeName()] = struct{}{}
			}
		}
	default:
		ftype, format, ok := primitiveSchema(ft)
		if ok {
			core = schemaCore{Type: ftype, Format: format}
		} else {
			core = schemaCore{Type: ft.String(), Format: "UNKNOWN"}
		}
	}

	ret := swaggerSchemaObject{}

	switch aggregate {
	case array:
		ret = swaggerSchemaObject{
			schemaCore: schemaCore{
				Type:  "array",
				Items: (*swaggerItemsObject)(&core),
			},
		}
	case object:
		ret = swaggerSchemaObject{
			schemaCore: schemaCore{
				Type: "object",
			},
			AdditionalProperties: &swaggerSchemaObject{Properties: props, schemaCore: core},
		}
	default:
		ret = swaggerSchemaObject{
			schemaCore: core,
			Properties: props,
		}
	}

	if j, err := extractJSONSchemaFromFieldDescriptor(f.FieldDescriptorProto); err == nil {
		updateSwaggerObjectFromJSONSchema(&ret, j, reg, f)
	}

	return ret
}

// primitiveSchema returns a pair of "Type" and "Format" in JSON Schema for
// the given primitive field type.
// The last return parameter is true iff the field type is actually primitive.
func primitiveSchema(t pbdescriptor.FieldDescriptorProto_Type) (ftype, format string, ok bool) {
	switch t {
	case pbdescriptor.FieldDescriptorProto_TYPE_DOUBLE:
		return "number", "double", true
	case pbdescriptor.FieldDescriptorProto_TYPE_FLOAT:
		return "number", "float", true
	case pbdescriptor.FieldDescriptorProto_TYPE_INT64:
		return "string", "int64", true
	case pbdescriptor.FieldDescriptorProto_TYPE_UINT64:
		// 64bit integer types are marshaled as string in the default JSONPb marshaler.
		// TODO(yugui) Add an option to declare 64bit integers as int64.
		//
		// NOTE: uint64 is not a predefined format of integer type in Swagger spec.
		// So we cannot expect that uint64 is commonly supported by swagger processor.
		return "string", "uint64", true
	case pbdescriptor.FieldDescriptorProto_TYPE_INT32:
		return "integer", "int32", true
	case pbdescriptor.FieldDescriptorProto_TYPE_FIXED64:
		// Ditto.
		return "string", "uint64", true
	case pbdescriptor.FieldDescriptorProto_TYPE_FIXED32:
		// Ditto.
		return "integer", "int64", true
	case pbdescriptor.FieldDescriptorProto_TYPE_BOOL:
		// NOTE: in swagger specification, format should be empty on boolean type
		return "boolean", "", true
	case pbdescriptor.FieldDescriptorProto_TYPE_STRING:
		// NOTE: in swagger specification, format should be empty on string type
		return "string", "", true
	case pbdescriptor.FieldDescriptorProto_TYPE_BYTES:
		return "string", "byte", true
	case pbdescriptor.FieldDescriptorProto_TYPE_UINT32:
		// Ditto.
		return "integer", "int64", true
	case pbdescriptor.FieldDescriptorProto_TYPE_SFIXED32:
		return "integer", "int32", true
	case pbdescriptor.FieldDescriptorProto_TYPE_SFIXED64:
		return "string", "int64", true
	case pbdescriptor.FieldDescriptorProto_TYPE_SINT32:
		return "integer", "int32", true
	case pbdescriptor.FieldDescriptorProto_TYPE_SINT64:
		return "string", "int64", true
	default:
		return "", "", false
	}
}

// renderEnumerationsAsDefinition inserts enums into the definitions object.
func renderEnumerationsAsDefinition(enums enumMap, d swaggerDefinitionsObject, reg *descriptor.Registry) {
	for _, enum := range enums {
		swgName, ok := fullyQualifiedNameToSwaggerName(enum.FQEN(), reg)
		if !ok {
			panic(fmt.Sprintf("can't resolve swagger name from FQEN '%v'", enum.FQEN()))
		}
		enumComments := protoComments(reg, enum.File, enum.Outers, "EnumType", int32(enum.Index))

		// it may be necessary to sort the result of the GetValue function.
		enumNames := listEnumNames(enum)
		defaultValue := getEnumDefault(enum)
		valueComments := enumValueProtoComments(reg, enum)
		if valueComments != "" {
			enumComments = strings.TrimLeft(enumComments+"\n\n "+valueComments, "\n")
		}
		enumSchemaObject := swaggerSchemaObject{
			schemaCore: schemaCore{
				Type:    "string",
				Enum:    enumNames,
				Default: defaultValue,
			},
		}
		if reg.GetEnumsAsInts() {
			enumSchemaObject.Type = "integer"
			enumSchemaObject.Format = "int32"
			enumSchemaObject.Default = "0"
			enumSchemaObject.Enum = listEnumNumbers(enum)
		}
		if err := updateSwaggerDataFromComments(reg, &enumSchemaObject, enum, enumComments, false); err != nil {
			panic(err)
		}

		d[swgName] = enumSchemaObject
	}
}

// Take in a FQMN or FQEN and return a swagger safe version of the FQMN and
// a boolean indicating if FQMN was properly resolved.
func fullyQualifiedNameToSwaggerName(fqn string, reg *descriptor.Registry) (string, bool) {
	registriesSeenMutex.Lock()
	defer registriesSeenMutex.Unlock()
	if mapping, present := registriesSeen[reg]; present {
		ret, ok := mapping[fqn]
		return ret, ok
	}
	mapping := resolveFullyQualifiedNameToSwaggerNames(append(reg.GetAllFQMNs(), reg.GetAllFQENs()...), reg.GetUseFQNForSwaggerName())
	registriesSeen[reg] = mapping
	ret, ok := mapping[fqn]
	return ret, ok
}

// Lookup message type by location.name and return a swagger-safe version
// of its FQMN.
func lookupMsgAndSwaggerName(location, name string, reg *descriptor.Registry) (*descriptor.Message, string, error) {
	msg, err := reg.LookupMsg(location, name)
	if err != nil {
		return nil, "", err
	}
	swgName, ok := fullyQualifiedNameToSwaggerName(msg.FQMN(), reg)
	if !ok {
		return nil, "", fmt.Errorf("can't map swagger name from FQMN '%v'", msg.FQMN())
	}
	return msg, swgName, nil
}

// registriesSeen is used to memoise calls to resolveFullyQualifiedNameToSwaggerNames so
// we don't repeat it unnecessarily, since it can take some time.
var registriesSeen = map[*descriptor.Registry]map[string]string{}
var registriesSeenMutex sync.Mutex

// Take the names of every proto and "uniq-ify" them. The idea is to produce a
// set of names that meet a couple of conditions. They must be stable, they
// must be unique, and they must be shorter than the FQN.
//
// This likely could be made better. This will always generate the same names
// but may not always produce optimal names. This is a reasonably close
// approximation of what they should look like in most cases.
func resolveFullyQualifiedNameToSwaggerNames(messages []string, useFQNForSwaggerName bool) map[string]string {
	packagesByDepth := make(map[int][][]string)
	uniqueNames := make(map[string]string)

	hierarchy := func(pkg string) []string {
		return strings.Split(pkg, ".")
	}

	for _, p := range messages {
		h := hierarchy(p)
		for depth := range h {
			if _, ok := packagesByDepth[depth]; !ok {
				packagesByDepth[depth] = make([][]string, 0)
			}
			packagesByDepth[depth] = append(packagesByDepth[depth], h[len(h)-depth:])
		}
	}

	count := func(list [][]string, item []string) int {
		i := 0
		for _, element := range list {
			if reflect.DeepEqual(element, item) {
				i++
			}
		}
		return i
	}

	for _, p := range messages {
		if useFQNForSwaggerName {
			// strip leading dot from proto fqn
			uniqueNames[p] = p[1:]
		} else {
			h := hierarchy(p)
			for depth := 0; depth < len(h); depth++ {
				if count(packagesByDepth[depth], h[len(h)-depth:]) == 1 {
					uniqueNames[p] = strings.Join(h[len(h)-depth-1:], "")
					break
				}
				if depth == len(h)-1 {
					uniqueNames[p] = strings.Join(h, "")
				}
			}
		}
	}
	return uniqueNames
}

var canRegexp = regexp.MustCompile("{([a-zA-Z][a-zA-Z0-9_.]*).*}")

// Swagger expects paths of the form /path/{string_value} but grpc-gateway paths are expected to be of the form /path/{string_value=strprefix/*}. This should reformat it correctly.
func templateToSwaggerPath(path string, reg *descriptor.Registry, fields []*descriptor.Field, msgs []*descriptor.Message) string {
	// It seems like the right thing to do here is to just use
	// strings.Split(path, "/") but that breaks badly when you hit a url like
	// /{my_field=prefix/*}/ and end up with 2 sections representing my_field.
	// Instead do the right thing and write a small pushdown (counter) automata
	// for it.
	var parts []string
	depth := 0
	buffer := ""
	jsonBuffer := ""
	for _, char := range path {
		switch char {
		case '{':
			// Push on the stack
			depth++
			buffer += string(char)
			jsonBuffer = ""
			jsonBuffer += string(char)
			break
		case '}':
			if depth == 0 {
				panic("Encountered } without matching { before it.")
			}
			// Pop from the stack
			depth--
			buffer += string(char)
			if reg.GetUseJSONNamesForFields() &&
				len(jsonBuffer) > 1 {
				jsonSnakeCaseName := string(jsonBuffer[1:])
				jsonCamelCaseName := string(lowerCamelCase(jsonSnakeCaseName, fields, msgs))
				prev := string(buffer[:len(buffer)-len(jsonSnakeCaseName)-2])
				buffer = strings.Join([]string{prev, "{", jsonCamelCaseName, "}"}, "")
				jsonBuffer = ""
			}
		case '/':
			if depth == 0 {
				parts = append(parts, buffer)
				buffer = ""
				// Since the stack was empty when we hit the '/' we are done with this
				// section.
				continue
			}
			buffer += string(char)
			jsonBuffer += string(char)
		default:
			buffer += string(char)
			jsonBuffer += string(char)
			break
		}
	}

	// Now append the last element to parts
	parts = append(parts, buffer)

	// Parts is now an array of segments of the path. Interestingly, since the
	// syntax for this subsection CAN be handled by a regexp since it has no
	// memory.
	for index, part := range parts {
		// If part is a resource name such as "parent", "name", "user.name", the format info must be retained.
		prefix := canRegexp.ReplaceAllString(part, "$1")
		if isResourceName(prefix) {
			continue
		}
		parts[index] = canRegexp.ReplaceAllString(part, "{$1}")
	}

	return strings.Join(parts, "/")
}

func isResourceName(prefix string) bool {
	words := strings.Split(prefix, ".")
	l := len(words)
	field := words[l-1]
	words = strings.Split(field, ":")
	field = words[0]
	return field == "parent" || field == "name"
}

func renderServices(services []*descriptor.Service, paths swaggerPathsObject, reg *descriptor.Registry, requestResponseRefs, customRefs refMap, msgs []*descriptor.Message) error {
	// Correctness of svcIdx and methIdx depends on 'services' containing the services in the same order as the 'file.Service' array.
	svcBaseIdx := 0
	var lastFile *descriptor.File = nil
	for svcIdx, svc := range services {
		if svc.File != lastFile {
			lastFile = svc.File
			svcBaseIdx = svcIdx
		}
		for methIdx, meth := range svc.Methods {
			for bIdx, b := range meth.Bindings {
				// Iterate over all the swagger parameters
				parameters := swaggerParametersObject{}
				for _, parameter := range b.PathParams {

					var paramType, paramFormat, desc, collectionFormat, defaultValue string
					var enumNames []string
					var items *swaggerItemsObject
					var minItems *int
					switch pt := parameter.Target.GetType(); pt {
					case pbdescriptor.FieldDescriptorProto_TYPE_GROUP, pbdescriptor.FieldDescriptorProto_TYPE_MESSAGE:
						if descriptor.IsWellKnownType(parameter.Target.GetTypeName()) {
							if parameter.IsRepeated() {
								return fmt.Errorf("only primitive and enum types are allowed in repeated path parameters")
							}
							schema := schemaOfField(parameter.Target, reg, customRefs)
							paramType = schema.Type
							paramFormat = schema.Format
							desc = schema.Description
							defaultValue = schema.Default
						} else {
							return fmt.Errorf("only primitive and well-known types are allowed in path parameters")
						}
					case pbdescriptor.FieldDescriptorProto_TYPE_ENUM:
						enum, err := reg.LookupEnum("", parameter.Target.GetTypeName())
						if err != nil {
							return err
						}
						paramType = "string"
						paramFormat = ""
						enumNames = listEnumNames(enum)
						if reg.GetEnumsAsInts() {
							paramType = "integer"
							paramFormat = ""
							enumNames = listEnumNumbers(enum)
						}
						schema := schemaOfField(parameter.Target, reg, customRefs)
						desc = schema.Description
						defaultValue = schema.Default
					default:
						var ok bool
						paramType, paramFormat, ok = primitiveSchema(pt)
						if !ok {
							return fmt.Errorf("unknown field type %v", pt)
						}

						schema := schemaOfField(parameter.Target, reg, customRefs)
						desc = schema.Description
						defaultValue = schema.Default
					}

					if parameter.IsRepeated() {
						core := schemaCore{Type: paramType, Format: paramFormat}
						if parameter.IsEnum() {
							var s []string
							core.Enum = enumNames
							enumNames = s
						}
						items = (*swaggerItemsObject)(&core)
						paramType = "array"
						paramFormat = ""
						collectionFormat = reg.GetRepeatedPathParamSeparatorName()
						minItems = new(int)
						*minItems = 1
					}

					if desc == "" {
						desc = fieldProtoComments(reg, parameter.Target.Message, parameter.Target)
					}
					parameterString := parameter.String()
					if reg.GetUseJSONNamesForFields() {
						parameterString = lowerCamelCase(parameterString, meth.RequestType.Fields, msgs)
					}
					parameters = append(parameters, swaggerParameterObject{
						Name:        parameterString,
						Description: desc,
						In:          "path",
						Required:    true,
						Default:     defaultValue,
						// Parameters in gRPC-Gateway can only be strings?
						Type:             paramType,
						Format:           paramFormat,
						Enum:             enumNames,
						Items:            items,
						CollectionFormat: collectionFormat,
						MinItems:         minItems,
					})
				}
				// Now check if there is a body parameter
				if b.Body != nil {
					var schema swaggerSchemaObject
					desc := ""

					if len(b.Body.FieldPath) == 0 {
						schema = swaggerSchemaObject{
							schemaCore: schemaCore{},
						}

						wknSchemaCore, isWkn := wktSchemas[meth.RequestType.FQMN()]
						if !isWkn {
							err := schema.setRefFromFQN(meth.RequestType.FQMN(), reg)
							if err != nil {
								return err
							}
						} else {
							schema.schemaCore = wknSchemaCore

							// Special workaround for Empty: it's well-known type but wknSchemas only returns schema.schemaCore; but we need to set schema.Properties which is a level higher.
							if meth.RequestType.FQMN() == ".google.protobuf.Empty" {
								schema.Properties = &swaggerSchemaObjectProperties{}
							}
						}
					} else {
						lastField := b.Body.FieldPath[len(b.Body.FieldPath)-1]
						schema = schemaOfField(lastField.Target, reg, customRefs)
						if schema.Description != "" {
							desc = schema.Description
						} else {
							desc = fieldProtoComments(reg, lastField.Target.Message, lastField.Target)
						}
					}

					if meth.GetClientStreaming() {
						desc += " (streaming inputs)"
					}
					parameters = append(parameters, swaggerParameterObject{
						Name:        "body",
						Description: desc,
						In:          "body",
						Required:    true,
						Schema:      &schema,
					})
					// add the parameters to the query string
					queryParams, err := messageToQueryParameters(meth.RequestType, reg, b.PathParams, b.Body)
					if err != nil {
						return err
					}
					parameters = append(parameters, queryParams...)
				} else if b.HTTPMethod == "GET" || b.HTTPMethod == "DELETE" {
					// add the parameters to the query string
					queryParams, err := messageToQueryParameters(meth.RequestType, reg, b.PathParams, b.Body)
					if err != nil {
						return err
					}
					parameters = append(parameters, queryParams...)
				}

				pathItemObject, ok := paths[templateToSwaggerPath(b.PathTmpl.Template, reg, meth.RequestType.Fields, msgs)]
				if !ok {
					pathItemObject = swaggerPathItemObject{}
				}

				methProtoPath := protoPathIndex(reflect.TypeOf((*pbdescriptor.ServiceDescriptorProto)(nil)), "Method")
				desc := "A successful response."
				var responseSchema swaggerSchemaObject

				if b.ResponseBody == nil || len(b.ResponseBody.FieldPath) == 0 {
					responseSchema = swaggerSchemaObject{
						schemaCore: schemaCore{},
					}

					// Don't link to a full definition for
					// empty; it's overly verbose.
					// schema.Properties{} renders it as
					// well, without a definition
					wknSchemaCore, isWkn := wktSchemas[meth.ResponseType.FQMN()]
					if !isWkn {
						err := responseSchema.setRefFromFQN(meth.ResponseType.FQMN(), reg)
						if err != nil {
							return err
						}
					} else {
						responseSchema.schemaCore = wknSchemaCore

						// Special workaround for Empty: it's well-known type but wknSchemas only returns schema.schemaCore; but we need to set schema.Properties which is a level higher.
						if meth.ResponseType.FQMN() == ".google.protobuf.Empty" {
							responseSchema.Properties = &swaggerSchemaObjectProperties{}
						}
					}
				} else {
					// This is resolving the value of response_body in the google.api.HttpRule
					lastField := b.ResponseBody.FieldPath[len(b.ResponseBody.FieldPath)-1]
					responseSchema = schemaOfField(lastField.Target, reg, customRefs)
					if responseSchema.Description != "" {
						desc = responseSchema.Description
					} else {
						desc = fieldProtoComments(reg, lastField.Target.Message, lastField.Target)
					}
				}
				if meth.GetServerStreaming() {
					desc += "(streaming responses)"
					responseSchema.Type = "object"
					swgRef, _ := fullyQualifiedNameToSwaggerName(meth.ResponseType.FQMN(), reg)
					responseSchema.Title = "Stream result of " + swgRef

					props := swaggerSchemaObjectProperties{
						keyVal{
							Key: "result",
							Value: swaggerSchemaObject{
								schemaCore: schemaCore{
									Ref: responseSchema.Ref,
								},
							},
						},
					}
					streamErrDef, hasStreamError := fullyQualifiedNameToSwaggerName(".grpc.gateway.runtime.StreamError", reg)
					if hasStreamError {
						props = append(props, keyVal{
							Key: "error",
							Value: swaggerSchemaObject{
								schemaCore: schemaCore{
									Ref: fmt.Sprintf("#/definitions/%s", streamErrDef)},
							},
						})
					}
					responseSchema.Properties = &props
					responseSchema.Ref = ""
				}

				tag := svc.GetName()
				if pkg := svc.File.GetPackage(); pkg != "" && reg.IsIncludePackageInTags() {
					tag = pkg + "." + tag
				}
				operationObject := &swaggerOperationObject{
					Tags:       []string{tag},
					Parameters: parameters,
					Responses: swaggerResponsesObject{
						"200": swaggerResponseObject{
							Description: desc,
							Schema:      responseSchema,
							Headers:     swaggerHeadersObject{},
						},
					},
				}
				if !reg.GetDisableDefaultErrors() {
					errDef, hasErrDef := fullyQualifiedNameToSwaggerName(".grpc.gateway.runtime.Error", reg)
					if hasErrDef {
						// https://github.com/OAI/OpenAPI-Specification/blob/3.0.0/versions/2.0.md#responses-object
						operationObject.Responses["default"] = swaggerResponseObject{
							Description: "An unexpected error response.",
							Schema: swaggerSchemaObject{
								schemaCore: schemaCore{
									Ref: fmt.Sprintf("#/definitions/%s", errDef),
								},
							},
						}
					}
				}
				operationObject.OperationID = fmt.Sprintf("%s_%s", svc.GetName(), meth.GetName())
				if reg.GetSimpleOperationIDs() {
					operationObject.OperationID = fmt.Sprintf("%s", meth.GetName())
				}
				if bIdx != 0 {
					// OperationID must be unique in an OpenAPI v2 definition.
					operationObject.OperationID += strconv.Itoa(bIdx + 1)
				}

				// Fill reference map with referenced request messages
				for _, param := range operationObject.Parameters {
					if param.Schema != nil && param.Schema.Ref != "" {
						requestResponseRefs[param.Schema.Ref] = struct{}{}
					}
				}

				methComments := protoComments(reg, svc.File, nil, "Service", int32(svcIdx-svcBaseIdx), methProtoPath, int32(methIdx))
				if err := updateSwaggerDataFromComments(reg, operationObject, meth, methComments, false); err != nil {
					panic(err)
				}

				opts, err := extractOperationOptionFromMethodDescriptor(meth.MethodDescriptorProto)
				if opts != nil {
					if err != nil {
						panic(err)
					}
					operationObject.ExternalDocs = protoExternalDocumentationToSwaggerExternalDocumentation(opts.ExternalDocs, reg, meth)
					// TODO(ivucica): this would be better supported by looking whether the method is deprecated in the proto file
					operationObject.Deprecated = opts.Deprecated

					if opts.Summary != "" {
						operationObject.Summary = opts.Summary
					}
					if opts.Description != "" {
						operationObject.Description = opts.Description
					}
					if len(opts.Tags) > 0 {
						operationObject.Tags = make([]string, len(opts.Tags))
						copy(operationObject.Tags, opts.Tags)
					}
					if opts.OperationId != "" {
						operationObject.OperationID = opts.OperationId
					}
					if opts.Security != nil {
						newSecurity := []swaggerSecurityRequirementObject{}
						if operationObject.Security != nil {
							newSecurity = *operationObject.Security
						}
						for _, secReq := range opts.Security {
							newSecReq := swaggerSecurityRequirementObject{}
							for secReqKey, secReqValue := range secReq.SecurityRequirement {
								if secReqValue == nil {
									continue
								}

								newSecReqValue := make([]string, len(secReqValue.Scope))
								copy(newSecReqValue, secReqValue.Scope)
								newSecReq[secReqKey] = newSecReqValue
							}

							if len(newSecReq) > 0 {
								newSecurity = append(newSecurity, newSecReq)
							}
						}
						operationObject.Security = &newSecurity
					}
					if opts.Responses != nil {
						for name, resp := range opts.Responses {
							// Merge response data into default response if available.
							respObj := operationObject.Responses[name]
							if resp.Description != "" {
								respObj.Description = resp.Description
							}
							if resp.Schema != nil {
								respObj.Schema = swaggerSchemaFromProtoSchema(resp.Schema, reg, customRefs, meth)
							}
							if resp.Examples != nil {
								respObj.Examples = swaggerExamplesFromProtoExamples(resp.Examples)
							}
							if resp.Headers != nil {
								hdrs, err := processHeaders(resp.Headers)
								if err != nil {
									return err
								}
								respObj.Headers = hdrs
							}
							if resp.Extensions != nil {
								exts, err := processExtensions(resp.Extensions)
								if err != nil {
									return err
								}
								respObj.extensions = exts
							}
							operationObject.Responses[name] = respObj
						}
					}

					if opts.Extensions != nil {
						exts, err := processExtensions(opts.Extensions)
						if err != nil {
							return err
						}
						operationObject.extensions = exts
					}

					if len(opts.Produces) > 0 {
						operationObject.Produces = make([]string, len(opts.Produces))
						copy(operationObject.Produces, opts.Produces)
					}

					// TODO(ivucica): add remaining fields of operation object
				}

				switch b.HTTPMethod {
				case "DELETE":
					pathItemObject.Delete = operationObject
					break
				case "GET":
					pathItemObject.Get = operationObject
					break
				case "POST":
					pathItemObject.Post = operationObject
					break
				case "PUT":
					pathItemObject.Put = operationObject
					break
				case "PATCH":
					pathItemObject.Patch = operationObject
					break
				}
				paths[templateToSwaggerPath(b.PathTmpl.Template, reg, meth.RequestType.Fields, msgs)] = pathItemObject
			}
		}
	}

	// Success! return nil on the error object
	return nil
}

// This function is called with a param which contains the entire definition of a method.
func applyTemplate(p param) (*swaggerObject, error) {
	// Create the basic template object. This is the object that everything is
	// defined off of.
	s := swaggerObject{
		// Swagger 2.0 is the version of this document
		Swagger:     "2.0",
		Consumes:    []string{"application/json"},
		Produces:    []string{"application/json"},
		Paths:       make(swaggerPathsObject),
		Definitions: make(swaggerDefinitionsObject),
		Info: swaggerInfoObject{
			Title:   *p.File.Name,
			Version: "version not set",
		},
	}

	// Loops through all the services and their exposed GET/POST/PUT/DELETE definitions
	// and create entries for all of them.
	// Also adds custom user specified references to second map.
	requestResponseRefs, customRefs := refMap{}, refMap{}
	if err := renderServices(p.Services, s.Paths, p.reg, requestResponseRefs, customRefs, p.Messages); err != nil {
		panic(err)
	}

	messages := messageMap{}
	streamingMessages := messageMap{}
	enums := enumMap{}

	if !p.reg.GetDisableDefaultErrors() {
		// Add the error type to the message map
		runtimeError, swgRef, err := lookupMsgAndSwaggerName(".grpc.gateway.runtime", "Error", p.reg)
		if err == nil {
			messages[swgRef] = runtimeError
		} else {
			// just in case there is an error looking up runtimeError
			glog.Error(err)
		}
	}

	// Find all the service's messages and enumerations that are defined (recursively)
	// and write request, response and other custom (but referenced) types out as definition objects.
	findServicesMessagesAndEnumerations(p.Services, p.reg, messages, streamingMessages, enums, requestResponseRefs)
	renderMessagesAsDefinition(messages, s.Definitions, p.reg, customRefs)
	renderEnumerationsAsDefinition(enums, s.Definitions, p.reg)

	// File itself might have some comments and metadata.
	packageProtoPath := protoPathIndex(reflect.TypeOf((*pbdescriptor.FileDescriptorProto)(nil)), "Package")
	packageComments := protoComments(p.reg, p.File, nil, "Package", packageProtoPath)
	if err := updateSwaggerDataFromComments(p.reg, &s, p, packageComments, true); err != nil {
		panic(err)
	}

	// There may be additional options in the swagger option in the proto.
	spb, err := extractSwaggerOptionFromFileDescriptor(p.FileDescriptorProto)
	if err != nil {
		panic(err)
	}
	if spb != nil {
		if spb.Swagger != "" {
			s.Swagger = spb.Swagger
		}
		if spb.Info != nil {
			if spb.Info.Title != "" {
				s.Info.Title = spb.Info.Title
			}
			if spb.Info.Description != "" {
				s.Info.Description = spb.Info.Description
			}
			if spb.Info.TermsOfService != "" {
				s.Info.TermsOfService = spb.Info.TermsOfService
			}
			if spb.Info.Version != "" {
				s.Info.Version = spb.Info.Version
			}
			if spb.Info.Contact != nil {
				if s.Info.Contact == nil {
					s.Info.Contact = &swaggerContactObject{}
				}
				if spb.Info.Contact.Name != "" {
					s.Info.Contact.Name = spb.Info.Contact.Name
				}
				if spb.Info.Contact.Url != "" {
					s.Info.Contact.URL = spb.Info.Contact.Url
				}
				if spb.Info.Contact.Email != "" {
					s.Info.Contact.Email = spb.Info.Contact.Email
				}
			}
			if spb.Info.License != nil {
				if s.Info.License == nil {
					s.Info.License = &swaggerLicenseObject{}
				}
				if spb.Info.License.Name != "" {
					s.Info.License.Name = spb.Info.License.Name
				}
				if spb.Info.License.Url != "" {
					s.Info.License.URL = spb.Info.License.Url
				}
			}
			if spb.Info.Extensions != nil {
				exts, err := processExtensions(spb.Info.Extensions)
				if err != nil {
					return nil, err
				}
				s.Info.extensions = exts
			}
		}
		if spb.Host != "" {
			s.Host = spb.Host
		}
		if spb.BasePath != "" {
			s.BasePath = spb.BasePath
		}
		if len(spb.Schemes) > 0 {
			s.Schemes = make([]string, len(spb.Schemes))
			for i, scheme := range spb.Schemes {
				s.Schemes[i] = strings.ToLower(scheme.String())
			}
		}
		if len(spb.Consumes) > 0 {
			s.Consumes = make([]string, len(spb.Consumes))
			copy(s.Consumes, spb.Consumes)
		}
		if len(spb.Produces) > 0 {
			s.Produces = make([]string, len(spb.Produces))
			copy(s.Produces, spb.Produces)
		}
		if spb.SecurityDefinitions != nil && spb.SecurityDefinitions.Security != nil {
			if s.SecurityDefinitions == nil {
				s.SecurityDefinitions = swaggerSecurityDefinitionsObject{}
			}
			for secDefKey, secDefValue := range spb.SecurityDefinitions.Security {
				var newSecDefValue swaggerSecuritySchemeObject
				if oldSecDefValue, ok := s.SecurityDefinitions[secDefKey]; !ok {
					newSecDefValue = swaggerSecuritySchemeObject{}
				} else {
					newSecDefValue = oldSecDefValue
				}
				if secDefValue.Type != swagger_options.SecurityScheme_TYPE_INVALID {
					switch secDefValue.Type {
					case swagger_options.SecurityScheme_TYPE_BASIC:
						newSecDefValue.Type = "basic"
					case swagger_options.SecurityScheme_TYPE_API_KEY:
						newSecDefValue.Type = "apiKey"
					case swagger_options.SecurityScheme_TYPE_OAUTH2:
						newSecDefValue.Type = "oauth2"
					}
				}
				if secDefValue.Description != "" {
					newSecDefValue.Description = secDefValue.Description
				}
				if secDefValue.Name != "" {
					newSecDefValue.Name = secDefValue.Name
				}
				if secDefValue.In != swagger_options.SecurityScheme_IN_INVALID {
					switch secDefValue.In {
					case swagger_options.SecurityScheme_IN_QUERY:
						newSecDefValue.In = "query"
					case swagger_options.SecurityScheme_IN_HEADER:
						newSecDefValue.In = "header"
					}
				}
				if secDefValue.Flow != swagger_options.SecurityScheme_FLOW_INVALID {
					switch secDefValue.Flow {
					case swagger_options.SecurityScheme_FLOW_IMPLICIT:
						newSecDefValue.Flow = "implicit"
					case swagger_options.SecurityScheme_FLOW_PASSWORD:
						newSecDefValue.Flow = "password"
					case swagger_options.SecurityScheme_FLOW_APPLICATION:
						newSecDefValue.Flow = "application"
					case swagger_options.SecurityScheme_FLOW_ACCESS_CODE:
						newSecDefValue.Flow = "accessCode"
					}
				}
				if secDefValue.AuthorizationUrl != "" {
					newSecDefValue.AuthorizationURL = secDefValue.AuthorizationUrl
				}
				if secDefValue.TokenUrl != "" {
					newSecDefValue.TokenURL = secDefValue.TokenUrl
				}
				if secDefValue.Scopes != nil {
					if newSecDefValue.Scopes == nil {
						newSecDefValue.Scopes = swaggerScopesObject{}
					}
					for scopeKey, scopeDesc := range secDefValue.Scopes.Scope {
						newSecDefValue.Scopes[scopeKey] = scopeDesc
					}
				}
				if secDefValue.Extensions != nil {
					exts, err := processExtensions(secDefValue.Extensions)
					if err != nil {
						return nil, err
					}
					newSecDefValue.extensions = exts
				}
				s.SecurityDefinitions[secDefKey] = newSecDefValue
			}
		}
		if spb.Security != nil {
			newSecurity := []swaggerSecurityRequirementObject{}
			if s.Security == nil {
				newSecurity = []swaggerSecurityRequirementObject{}
			} else {
				newSecurity = s.Security
			}
			for _, secReq := range spb.Security {
				newSecReq := swaggerSecurityRequirementObject{}
				for secReqKey, secReqValue := range secReq.SecurityRequirement {
					newSecReqValue := make([]string, len(secReqValue.Scope))
					copy(newSecReqValue, secReqValue.Scope)
					newSecReq[secReqKey] = newSecReqValue
				}
				newSecurity = append(newSecurity, newSecReq)
			}
			s.Security = newSecurity
		}
		s.ExternalDocs = protoExternalDocumentationToSwaggerExternalDocumentation(spb.ExternalDocs, p.reg, spb)
		// Populate all Paths with Responses set at top level,
		// preferring Responses already set over those at the top level.
		if spb.Responses != nil {
			for _, verbs := range s.Paths {
				var maps []swaggerResponsesObject
				if verbs.Delete != nil {
					maps = append(maps, verbs.Delete.Responses)
				}
				if verbs.Get != nil {
					maps = append(maps, verbs.Get.Responses)
				}
				if verbs.Post != nil {
					maps = append(maps, verbs.Post.Responses)
				}
				if verbs.Put != nil {
					maps = append(maps, verbs.Put.Responses)
				}
				if verbs.Patch != nil {
					maps = append(maps, verbs.Patch.Responses)
				}

				for k, v := range spb.Responses {
					for _, respMap := range maps {
						if _, ok := respMap[k]; ok {
							// Don't overwrite already existing Responses
							continue
						}
						respMap[k] = swaggerResponseObject{
							Description: v.Description,
							Schema:      swaggerSchemaFromProtoSchema(v.Schema, p.reg, customRefs, nil),
							Examples:    swaggerExamplesFromProtoExamples(v.Examples),
						}
					}
				}
			}
		}

		if spb.Extensions != nil {
			exts, err := processExtensions(spb.Extensions)
			if err != nil {
				return nil, err
			}
			s.extensions = exts
		}

		// Additional fields on the OpenAPI v2 spec's "Swagger" object
		// should be added here, once supported in the proto.
	}

	// Finally add any references added by users that aren't
	// otherwise rendered.
	addCustomRefs(s.Definitions, p.reg, customRefs)

	return &s, nil
}

func processExtensions(inputExts map[string]*structpb.Value) ([]extension, error) {
	exts := []extension{}
	for k, v := range inputExts {
		if !strings.HasPrefix(k, "x-") {
			return nil, fmt.Errorf("Extension keys need to start with \"x-\": %q", k)
		}
		ext, err := (&jsonpb.Marshaler{Indent: "  "}).MarshalToString(v)
		if err != nil {
			return nil, err
		}
		exts = append(exts, extension{key: k, value: json.RawMessage(ext)})
	}
	sort.Slice(exts, func(i, j int) bool { return exts[i].key < exts[j].key })
	return exts, nil
}

func validateHeaderTypeAndFormat(headerType string, format string) error {
	// The type of the object. The value MUST be one of "string", "number", "integer", "boolean", or "array"
	// See: https://github.com/OAI/OpenAPI-Specification/blob/3.0.0/versions/2.0.md#headerObject
	// Note: currently not implementing array as we are only implementing this in the operation response context
	switch headerType {
	// the format property is an open string-valued property, and can have any value to support documentation needs
	// primary check for format is to ensure that the number/integer formats are extensions of the specified type
	// See: https://github.com/OAI/OpenAPI-Specification/blob/3.0.0/versions/2.0.md#dataTypeFormat
	case "string":
		return nil
	case "number":
		switch format {
		case "uint",
			"uint8",
			"uint16",
			"uint32",
			"uint64",
			"int",
			"int8",
			"int16",
			"int32",
			"int64",
			"float",
			"float32",
			"float64",
			"complex64",
			"complex128",
			"double",
			"byte",
			"rune",
			"uintptr",
			"":
			return nil
		default:
			return fmt.Errorf("the provided format %q is not a valid extension of the type %q", format, headerType)
		}
	case "integer":
		switch format {
		case "uint",
			"uint8",
			"uint16",
			"uint32",
			"uint64",
			"int",
			"int8",
			"int16",
			"int32",
			"int64",
			"":
			return nil
		default:
			return fmt.Errorf("the provided format %q is not a valid extension of the type %q", format, headerType)
		}
	case "boolean":
		return nil
	}
	return fmt.Errorf("the provided header type %q is not supported", headerType)
}

func validateDefaultValueTypeAndFormat(headerType string, defaultValue string, format string) error {
	switch headerType {
	case "string":
		if !isQuotedString(defaultValue) {
			return fmt.Errorf("the provided default value %q does not match provider type %q, or is not properly quoted with escaped quotations", defaultValue, headerType)
		}
		switch format {
		case "date-time":
			unquoteTime := strings.Trim(defaultValue, `"`)
			_, err := time.Parse(time.RFC3339, unquoteTime)
			if err != nil {
				return fmt.Errorf("the provided default value %q is not a valid RFC3339 date-time string", defaultValue)
			}
		case "date":
			const (
				layoutRFC3339Date = "2006-01-02"
			)
			unquoteDate := strings.Trim(defaultValue, `"`)
			_, err := time.Parse(layoutRFC3339Date, unquoteDate)
			if err != nil {
				return fmt.Errorf("the provided default value %q is not a valid RFC3339 date-time string", defaultValue)
			}
		}
	case "number":
		err := isJSONNumber(defaultValue, headerType)
		if err != nil {
			return err
		}
	case "integer":
		switch format {
		case "int32":
			_, err := strconv.ParseInt(defaultValue, 0, 32)
			if err != nil {
				return fmt.Errorf("the provided default value %q does not match provided format %q", defaultValue, format)
			}
		case "uint32":
			_, err := strconv.ParseUint(defaultValue, 0, 32)
			if err != nil {
				return fmt.Errorf("the provided default value %q does not match provided format %q", defaultValue, format)
			}
		case "int64":
			_, err := strconv.ParseInt(defaultValue, 0, 64)
			if err != nil {
				return fmt.Errorf("the provided default value %q does not match provided format %q", defaultValue, format)
			}
		case "uint64":
			_, err := strconv.ParseUint(defaultValue, 0, 64)
			if err != nil {
				return fmt.Errorf("the provided default value %q does not match provided format %q", defaultValue, format)
			}
		default:
			_, err := strconv.ParseInt(defaultValue, 0, 64)
			if err != nil {
				return fmt.Errorf("the provided default value %q does not match provided type %q", defaultValue, headerType)
			}
		}
	case "boolean":
		if !isBool(defaultValue) {
			return fmt.Errorf("the provided default value %q does not match provider type %q", defaultValue, headerType)
		}
	}
	return nil
}

func isQuotedString(s string) bool {
	return len(s) >= 2 && s[0] == '"' && s[len(s)-1] == '"'
}

func isJSONNumber(s string, t string) error {
	val, err := strconv.ParseFloat(s, 64)
	if err != nil {
		return fmt.Errorf("the provided default value %q does not match provider type %q", s, t)
	}
	// Floating point values that cannot be represented as sequences of digits (such as Infinity and NaN) are not permitted.
	// See: https://tools.ietf.org/html/rfc4627#section-2.4
	if math.IsInf(val, 0) || math.IsNaN(val) {
		return fmt.Errorf("the provided number %q is not a valid JSON number", s)
	}

	return nil
}

func isBool(s string) bool {
	// Unable to use strconv.ParseBool because it returns truthy values https://golang.org/pkg/strconv/#example_ParseBool
	// per https://swagger.io/specification/v2/#data-types
	// type: boolean represents two values: true and false. Note that truthy and falsy values such as "true", "", 0 or null are not considered boolean values.
	return s == "true" || s == "false"
}

func processHeaders(inputHdrs map[string]*swagger_options.Header) (swaggerHeadersObject, error) {
	hdrs := map[string]swaggerHeaderObject{}
	for k, v := range inputHdrs {
		header := textproto.CanonicalMIMEHeaderKey(k)
		ret := swaggerHeaderObject{
			Description: v.Description,
			Format:      v.Format,
			Pattern:     v.Pattern,
		}
		err := validateHeaderTypeAndFormat(v.Type, v.Format)
		if err != nil {
			return nil, err
		}
		ret.Type = v.Type
		if v.Default != "" {
			err := validateDefaultValueTypeAndFormat(v.Type, v.Default, v.Format)
			if err != nil {
				return nil, err
			}
			ret.Default = json.RawMessage(v.Default)
		}
		hdrs[header] = ret
	}
	return hdrs, nil
}

// updateSwaggerDataFromComments updates a Swagger object based on a comment
// from the proto file.
//
// First paragraph of a comment is used for summary. Remaining paragraphs of
// a comment are used for description. If 'Summary' field is not present on
// the passed swaggerObject, the summary and description are joined by \n\n.
//
// If there is a field named 'Info', its 'Summary' and 'Description' fields
// will be updated instead.
//
// If there is no 'Summary', the same behavior will be attempted on 'Title',
// but only if the last character is not a period.
func updateSwaggerDataFromComments(reg *descriptor.Registry, swaggerObject interface{}, data interface{}, comment string, isPackageObject bool) error {
	if len(comment) == 0 {
		return nil
	}

	// Checks whether the "use_go_templates" flag is set to true
	if reg.GetUseGoTemplate() {
		comment = goTemplateComments(comment, data, reg)
	}

	// Figure out what to apply changes to.
	swaggerObjectValue := reflect.ValueOf(swaggerObject)
	infoObjectValue := swaggerObjectValue.Elem().FieldByName("Info")
	if !infoObjectValue.CanSet() {
		// No such field? Apply summary and description directly to
		// passed object.
		infoObjectValue = swaggerObjectValue.Elem()
	}

	// Figure out which properties to update.
	summaryValue := infoObjectValue.FieldByName("Summary")
	descriptionValue := infoObjectValue.FieldByName("Description")
	readOnlyValue := infoObjectValue.FieldByName("ReadOnly")

	if readOnlyValue.Kind() == reflect.Bool && readOnlyValue.CanSet() && strings.Contains(comment, "Output only.") {
		readOnlyValue.Set(reflect.ValueOf(true))
	}

	usingTitle := false
	if !summaryValue.CanSet() {
		summaryValue = infoObjectValue.FieldByName("Title")
		usingTitle = true
	}

	paragraphs := strings.Split(comment, "\n\n")

	// If there is a summary (or summary-equivalent) and it's empty, use the first
	// paragraph as summary, and the rest as description.
	if summaryValue.CanSet() {
		summary := strings.TrimSpace(paragraphs[0])
		description := strings.TrimSpace(strings.Join(paragraphs[1:], "\n\n"))
		if !usingTitle || (len(summary) > 0 && summary[len(summary)-1] != '.') {
			// overrides the schema value only if it's empty
			// keep the comment precedence when updating the package definition
			if summaryValue.Len() == 0 || isPackageObject {
				summaryValue.Set(reflect.ValueOf(summary))
			}
			if len(description) > 0 {
				if !descriptionValue.CanSet() {
					return fmt.Errorf("Encountered object type with a summary, but no description")
				}
				// overrides the schema value only if it's empty
				// keep the comment precedence when updating the package definition
				if descriptionValue.Len() == 0 || isPackageObject {
					descriptionValue.Set(reflect.ValueOf(description))
				}
			}
			return nil
		}
	}

	// There was no summary field on the swaggerObject. Try to apply the
	// whole comment into description if the swagger object description is empty.
	if descriptionValue.CanSet() {
		if descriptionValue.Len() == 0 || isPackageObject {
			descriptionValue.Set(reflect.ValueOf(strings.Join(paragraphs, "\n\n")))
		}
		return nil
	}

	return fmt.Errorf("no description nor summary property")
}

func fieldProtoComments(reg *descriptor.Registry, msg *descriptor.Message, field *descriptor.Field) string {
	protoPath := protoPathIndex(reflect.TypeOf((*pbdescriptor.DescriptorProto)(nil)), "Field")
	for i, f := range msg.Fields {
		if f == field {
			return protoComments(reg, msg.File, msg.Outers, "MessageType", int32(msg.Index), protoPath, int32(i))
		}
	}
	return ""
}

func enumValueProtoComments(reg *descriptor.Registry, enum *descriptor.Enum) string {
	protoPath := protoPathIndex(reflect.TypeOf((*pbdescriptor.EnumDescriptorProto)(nil)), "Value")
	var comments []string
	for idx, value := range enum.GetValue() {
		name := value.GetName()
		if reg.GetEnumsAsInts() {
			name = strconv.Itoa(int(value.GetNumber()))
		}
		str := protoComments(reg, enum.File, enum.Outers, "EnumType", int32(enum.Index), protoPath, int32(idx))
		if str != "" {
			comments = append(comments, name+": "+str)
		}
	}
	if len(comments) > 0 {
		return "- " + strings.Join(comments, "\n - ")
	}
	return ""
}

func protoComments(reg *descriptor.Registry, file *descriptor.File, outers []string, typeName string, typeIndex int32, fieldPaths ...int32) string {
	if file.SourceCodeInfo == nil {
		fmt.Fprintln(os.Stderr, "descriptor.File should not contain nil SourceCodeInfo")
		return ""
	}

	outerPaths := make([]int32, len(outers))
	for i := range outers {
		location := ""
		if file.Package != nil {
			location = file.GetPackage()
		}

		msg, err := reg.LookupMsg(location, strings.Join(outers[:i+1], "."))
		if err != nil {
			panic(err)
		}
		outerPaths[i] = int32(msg.Index)
	}

	for _, loc := range file.SourceCodeInfo.Location {
		if !isProtoPathMatches(loc.Path, outerPaths, typeName, typeIndex, fieldPaths) {
			continue
		}
		comments := ""
		if loc.LeadingComments != nil {
			comments = strings.TrimRight(*loc.LeadingComments, "\n")
			comments = strings.TrimSpace(comments)
			// TODO(ivucica): this is a hack to fix "// " being interpreted as "//".
			// perhaps we should:
			// - split by \n
			// - determine if every (but first and last) line begins with " "
			// - trim every line only if that is the case
			// - join by \n
			comments = strings.Replace(comments, "\n ", "\n", -1)
		}
		return comments
	}
	return ""
}

func goTemplateComments(comment string, data interface{}, reg *descriptor.Registry) string {
	var temp bytes.Buffer
	tpl, err := template.New("").Funcs(template.FuncMap{
		// Allows importing documentation from a file
		"import": func(name string) string {
			file, err := ioutil.ReadFile(name)
			if err != nil {
				return err.Error()
			}
			// Runs template over imported file
			return goTemplateComments(string(file), data, reg)
		},
		// Grabs title and description from a field
		"fieldcomments": func(msg *descriptor.Message, field *descriptor.Field) string {
			return strings.Replace(fieldProtoComments(reg, msg, field), "\n", "<br>", -1)
		},
	}).Parse(comment)
	if err != nil {
		// If there is an error parsing the templating insert the error as string in the comment
		// to make it easier to debug the template error
		return err.Error()
	}
	err = tpl.Execute(&temp, data)
	if err != nil {
		// If there is an error executing the templating insert the error as string in the comment
		// to make it easier to debug the error
		return err.Error()
	}
	return temp.String()
}

var messageProtoPath = protoPathIndex(reflect.TypeOf((*pbdescriptor.FileDescriptorProto)(nil)), "MessageType")
var nestedProtoPath = protoPathIndex(reflect.TypeOf((*pbdescriptor.DescriptorProto)(nil)), "NestedType")
var packageProtoPath = protoPathIndex(reflect.TypeOf((*pbdescriptor.FileDescriptorProto)(nil)), "Package")
var serviceProtoPath = protoPathIndex(reflect.TypeOf((*pbdescriptor.FileDescriptorProto)(nil)), "Service")
var methodProtoPath = protoPathIndex(reflect.TypeOf((*pbdescriptor.ServiceDescriptorProto)(nil)), "Method")

func isProtoPathMatches(paths []int32, outerPaths []int32, typeName string, typeIndex int32, fieldPaths []int32) bool {
	if typeName == "Package" && typeIndex == packageProtoPath {
		// path for package comments is just [2], and all the other processing
		// is too complex for it.
		if len(paths) == 0 || typeIndex != paths[0] {
			return false
		}
		return true
	}

	if len(paths) != len(outerPaths)*2+2+len(fieldPaths) {
		return false
	}

	if typeName == "Method" {
		if paths[0] != serviceProtoPath || paths[2] != methodProtoPath {
			return false
		}
		paths = paths[2:]
	} else {
		typeNameDescriptor := reflect.TypeOf((*pbdescriptor.FileDescriptorProto)(nil))

		if len(outerPaths) > 0 {
			if paths[0] != messageProtoPath || paths[1] != outerPaths[0] {
				return false
			}
			paths = paths[2:]
			outerPaths = outerPaths[1:]

			for i, v := range outerPaths {
				if paths[i*2] != nestedProtoPath || paths[i*2+1] != v {
					return false
				}
			}
			paths = paths[len(outerPaths)*2:]

			if typeName == "MessageType" {
				typeName = "NestedType"
			}
			typeNameDescriptor = reflect.TypeOf((*pbdescriptor.DescriptorProto)(nil))
		}

		if paths[0] != protoPathIndex(typeNameDescriptor, typeName) || paths[1] != typeIndex {
			return false
		}
		paths = paths[2:]
	}

	for i, v := range fieldPaths {
		if paths[i] != v {
			return false
		}
	}
	return true
}

// protoPathIndex returns a path component for google.protobuf.descriptor.SourceCode_Location.
//
// Specifically, it returns an id as generated from descriptor proto which
// can be used to determine what type the id following it in the path is.
// For example, if we are trying to locate comments related to a field named
// `Address` in a message named `Person`, the path will be:
//
//	 [4, a, 2, b]
//
// While `a` gets determined by the order in which the messages appear in
// the proto file, and `b` is the field index specified in the proto
// file itself, the path actually needs to specify that `a` refers to a
// message and not, say, a service; and  that `b` refers to a field and not
// an option.
//
// protoPathIndex figures out the values 4 and 2 in the above example. Because
// messages are top level objects, the value of 4 comes from field id for
// `MessageType` inside `google.protobuf.descriptor.FileDescriptor` message.
// This field has a message type `google.protobuf.descriptor.DescriptorProto`.
// And inside message `DescriptorProto`, there is a field named `Field` with id
// 2.
//
// Some code generators seem to be hardcoding these values; this method instead
// interprets them from `descriptor.proto`-derived Go source as necessary.
func protoPathIndex(descriptorType reflect.Type, what string) int32 {
	field, ok := descriptorType.Elem().FieldByName(what)
	if !ok {
		panic(fmt.Errorf("could not find protobuf descriptor type id for %s", what))
	}
	pbtag := field.Tag.Get("protobuf")
	if pbtag == "" {
		panic(fmt.Errorf("no Go tag 'protobuf' on protobuf descriptor for %s", what))
	}
	path, err := strconv.Atoi(strings.Split(pbtag, ",")[1])
	if err != nil {
		panic(fmt.Errorf("protobuf descriptor id for %s cannot be converted to a number: %s", what, err.Error()))
	}

	return int32(path)
}

// extractOperationOptionFromMethodDescriptor extracts the message of type
// swagger_options.Operation from a given proto method's descriptor.
func extractOperationOptionFromMethodDescriptor(meth *pbdescriptor.MethodDescriptorProto) (*swagger_options.Operation, error) {
	if meth.Options == nil {
		return nil, nil
	}
	if !proto.HasExtension(meth.Options, swagger_options.E_Openapiv2Operation) {
		return nil, nil
	}
	ext, err := proto.GetExtension(meth.Options, swagger_options.E_Openapiv2Operation)
	if err != nil {
		return nil, err
	}
	opts, ok := ext.(*swagger_options.Operation)
	if !ok {
		return nil, fmt.Errorf("extension is %T; want an Operation", ext)
	}
	return opts, nil
}

// extractSchemaOptionFromMessageDescriptor extracts the message of type
// swagger_options.Schema from a given proto message's descriptor.
func extractSchemaOptionFromMessageDescriptor(msg *pbdescriptor.DescriptorProto) (*swagger_options.Schema, error) {
	if msg.Options == nil {
		return nil, nil
	}
	if !proto.HasExtension(msg.Options, swagger_options.E_Openapiv2Schema) {
		return nil, nil
	}
	ext, err := proto.GetExtension(msg.Options, swagger_options.E_Openapiv2Schema)
	if err != nil {
		return nil, err
	}
	opts, ok := ext.(*swagger_options.Schema)
	if !ok {
		return nil, fmt.Errorf("extension is %T; want a Schema", ext)
	}
	return opts, nil
}

// extractSwaggerOptionFromFileDescriptor extracts the message of type
// swagger_options.Swagger from a given proto method's descriptor.
func extractSwaggerOptionFromFileDescriptor(file *pbdescriptor.FileDescriptorProto) (*swagger_options.Swagger, error) {
	if file.Options == nil {
		return nil, nil
	}
	if !proto.HasExtension(file.Options, swagger_options.E_Openapiv2Swagger) {
		return nil, nil
	}
	ext, err := proto.GetExtension(file.Options, swagger_options.E_Openapiv2Swagger)
	if err != nil {
		return nil, err
	}
	opts, ok := ext.(*swagger_options.Swagger)
	if !ok {
		return nil, fmt.Errorf("extension is %T; want a Swagger object", ext)
	}
	return opts, nil
}

func extractJSONSchemaFromFieldDescriptor(fd *pbdescriptor.FieldDescriptorProto) (*swagger_options.JSONSchema, error) {
	if fd.Options == nil {
		return nil, nil
	}
	if !proto.HasExtension(fd.Options, swagger_options.E_Openapiv2Field) {
		return nil, nil
	}
	ext, err := proto.GetExtension(fd.Options, swagger_options.E_Openapiv2Field)
	if err != nil {
		return nil, err
	}
	opts, ok := ext.(*swagger_options.JSONSchema)
	if !ok {
		return nil, fmt.Errorf("extension is %T; want a JSONSchema object", ext)
	}
	return opts, nil
}

func protoJSONSchemaToSwaggerSchemaCore(j *swagger_options.JSONSchema, reg *descriptor.Registry, refs refMap) schemaCore {
	ret := schemaCore{}

	if j.GetRef() != "" {
		swaggerName, ok := fullyQualifiedNameToSwaggerName(j.GetRef(), reg)
		if ok {
			ret.Ref = "#/definitions/" + swaggerName
			if refs != nil {
				refs[j.GetRef()] = struct{}{}
			}
		} else {
			ret.Ref += j.GetRef()
		}
	} else {
		f, t := protoJSONSchemaTypeToFormat(j.GetType())
		ret.Format = f
		ret.Type = t
	}

	return ret
}

func updateSwaggerObjectFromJSONSchema(s *swaggerSchemaObject, j *swagger_options.JSONSchema, reg *descriptor.Registry, data interface{}) {
	s.Title = j.GetTitle()
	s.Description = j.GetDescription()
	if reg.GetUseGoTemplate() {
		s.Title = goTemplateComments(s.Title, data, reg)
		s.Description = goTemplateComments(s.Description, data, reg)
	}

	s.ReadOnly = j.GetReadOnly()
	s.MultipleOf = j.GetMultipleOf()
	s.Maximum = j.GetMaximum()
	s.ExclusiveMaximum = j.GetExclusiveMaximum()
	s.Minimum = j.GetMinimum()
	s.ExclusiveMinimum = j.GetExclusiveMinimum()
	s.MaxLength = j.GetMaxLength()
	s.MinLength = j.GetMinLength()
	s.Pattern = j.GetPattern()
	s.Default = j.GetDefault()
	s.MaxItems = j.GetMaxItems()
	s.MinItems = j.GetMinItems()
	s.UniqueItems = j.GetUniqueItems()
	s.MaxProperties = j.GetMaxProperties()
	s.MinProperties = j.GetMinProperties()
	s.Required = j.GetRequired()
	s.Enum = j.GetEnum()
	if overrideType := j.GetType(); len(overrideType) > 0 {
		s.Type = strings.ToLower(overrideType[0].String())
	}
	if j != nil && j.GetExample() != "" {
		s.Example = json.RawMessage(j.GetExample())
	}
	if j != nil && j.GetFormat() != "" {
		s.Format = j.GetFormat()
	}
}

func swaggerSchemaFromProtoSchema(s *swagger_options.Schema, reg *descriptor.Registry, refs refMap, data interface{}) swaggerSchemaObject {
	ret := swaggerSchemaObject{
		ExternalDocs: protoExternalDocumentationToSwaggerExternalDocumentation(s.GetExternalDocs(), reg, data),
	}

	ret.schemaCore = protoJSONSchemaToSwaggerSchemaCore(s.GetJsonSchema(), reg, refs)
	updateSwaggerObjectFromJSONSchema(&ret, s.GetJsonSchema(), reg, data)

	if s != nil && s.Example != nil {
		ret.Example = json.RawMessage(s.Example.Value)
	}
	if s != nil && s.ExampleString != "" {
		ret.Example = json.RawMessage(s.ExampleString)
	}

	return ret
}

func swaggerExamplesFromProtoExamples(in map[string]string) map[string]interface{} {
	if len(in) == 0 {
		return nil
	}
	out := make(map[string]interface{})
	for mimeType, exampleStr := range in {
		switch mimeType {
		case "application/json":
			// JSON example objects are rendered raw.
			out[mimeType] = json.RawMessage(exampleStr)
		default:
			// All other mimetype examples are rendered as strings.
			out[mimeType] = exampleStr
		}
	}
	return out
}

func protoJSONSchemaTypeToFormat(in []swagger_options.JSONSchema_JSONSchemaSimpleTypes) (string, string) {
	if len(in) == 0 {
		return "", ""
	}

	// Can't support more than 1 type, just return the first element.
	// This is due to an inconsistency in the design of the openapiv2 proto
	// and that used in schemaCore. schemaCore uses the v3 definition of types,
	// which only allows a single string, while the openapiv2 proto uses the OpenAPI v2
	// definition, which defers to the JSON schema definition, which allows a string or an array.
	// Sources:
	// https://swagger.io/specification/#itemsObject
	// https://tools.ietf.org/html/draft-fge-json-schema-validation-00#section-5.5.2
	switch in[0] {
	case swagger_options.JSONSchema_UNKNOWN, swagger_options.JSONSchema_NULL:
		return "", ""
	case swagger_options.JSONSchema_OBJECT:
		return "object", ""
	case swagger_options.JSONSchema_ARRAY:
		return "array", ""
	case swagger_options.JSONSchema_BOOLEAN:
		// NOTE: in swagger specification, format should be empty on boolean type
		return "boolean", ""
	case swagger_options.JSONSchema_INTEGER:
		return "integer", "int32"
	case swagger_options.JSONSchema_NUMBER:
		return "number", "double"
	case swagger_options.JSONSchema_STRING:
		// NOTE: in swagger specification, format should be empty on string type
		return "string", ""
	default:
		// Maybe panic?
		return "", ""
	}
}

func protoExternalDocumentationToSwaggerExternalDocumentation(in *swagger_options.ExternalDocumentation, reg *descriptor.Registry, data interface{}) *swaggerExternalDocumentationObject {
	if in == nil {
		return nil
	}

	if reg.GetUseGoTemplate() {
		in.Description = goTemplateComments(in.Description, data, reg)
	}

	return &swaggerExternalDocumentationObject{
		Description: in.Description,
		URL:         in.Url,
	}
}

func addCustomRefs(d swaggerDefinitionsObject, reg *descriptor.Registry, refs refMap) {
	if len(refs) == 0 {
		return
	}
	msgMap := make(messageMap)
	enumMap := make(enumMap)
	for ref := range refs {
		swgName, swgOk := fullyQualifiedNameToSwaggerName(ref, reg)
		if !swgOk {
			glog.Errorf("can't resolve swagger name from CustomRef '%v'", ref)
			continue
		}
		if _, ok := d[swgName]; ok {
			// Skip already existing definitions
			delete(refs, ref)
			continue
		}
		msg, err := reg.LookupMsg("", ref)
		if err == nil {
			msgMap[swgName] = msg
			continue
		}
		enum, err := reg.LookupEnum("", ref)
		if err == nil {
			enumMap[swgName] = enum
			continue
		}

		// ?? Should be either enum or msg
	}
	renderMessagesAsDefinition(msgMap, d, reg, refs)
	renderEnumerationsAsDefinition(enumMap, d, reg)

	// Run again in case any new refs were added
	addCustomRefs(d, reg, refs)
}

func lowerCamelCase(fieldName string, fields []*descriptor.Field, msgs []*descriptor.Message) string {
	for _, oneField := range fields {
		if oneField.GetName() == fieldName {
			return oneField.GetJsonName()
		}
	}
	messageNameToFieldsToJSONName := make(map[string]map[string]string, 0)
	fieldNameToType := make(map[string]string, 0)
	for _, msg := range msgs {
		fieldNameToJSONName := make(map[string]string, 0)
		for _, oneField := range msg.GetField() {
			fieldNameToJSONName[oneField.GetName()] = oneField.GetJsonName()
			fieldNameToType[oneField.GetName()] = oneField.GetTypeName()
		}
		messageNameToFieldsToJSONName[msg.GetName()] = fieldNameToJSONName
	}
	if strings.Contains(fieldName, ".") {
		fieldNames := strings.Split(fieldName, ".")
		fieldNamesWithCamelCase := make([]string, 0)
		for i := 0; i < len(fieldNames)-1; i++ {
			fieldNamesWithCamelCase = append(fieldNamesWithCamelCase, doCamelCase(string(fieldNames[i])))
		}
		prefix := strings.Join(fieldNamesWithCamelCase, ".")
		reservedJSONName := getReservedJSONName(fieldName, messageNameToFieldsToJSONName, fieldNameToType)
		if reservedJSONName != "" {
			return prefix + "." + reservedJSONName
		}
	}
	return doCamelCase(fieldName)
}

func doCamelCase(input string) string {
	parameterString := casing.Camel(input)
	builder := &strings.Builder{}
	builder.WriteString(strings.ToLower(string(parameterString[0])))
	builder.WriteString(parameterString[1:])
	return builder.String()
}

func getReservedJSONName(fieldName string, messageNameToFieldsToJSONName map[string]map[string]string, fieldNameToType map[string]string) string {
	if len(strings.Split(fieldName, ".")) == 2 {
		fieldNames := strings.Split(fieldName, ".")
		firstVariable := fieldNames[0]
		firstType := fieldNameToType[firstVariable]
		firstTypeShortNames := strings.Split(firstType, ".")
		firstTypeShortName := firstTypeShortNames[len(firstTypeShortNames)-1]
		return messageNameToFieldsToJSONName[firstTypeShortName][fieldNames[1]]
	}
	fieldNames := strings.Split(fieldName, ".")
	return getReservedJSONName(strings.Join(fieldNames[1:], "."), messageNameToFieldsToJSONName, fieldNameToType)
}
