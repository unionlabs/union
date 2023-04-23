package genopenapi

import (
	"bytes"
	"encoding/json"
	"fmt"

	"github.com/grpc-ecosystem/grpc-gateway/v2/internal/descriptor"
	"gopkg.in/yaml.v3"
)

type param struct {
	*descriptor.File
	reg *descriptor.Registry
}

// http://swagger.io/specification/#infoObject
type openapiInfoObject struct {
	Title          string `json:"title" yaml:"title"`
	Description    string `json:"description,omitempty" yaml:"description,omitempty"`
	TermsOfService string `json:"termsOfService,omitempty" yaml:"termsOfService,omitempty"`
	Version        string `json:"version" yaml:"version"`

	Contact *openapiContactObject `json:"contact,omitempty" yaml:"contact,omitempty"`
	License *openapiLicenseObject `json:"license,omitempty" yaml:"license,omitempty"`

	extensions []extension `json:"-" yaml:"-"`
}

// https://swagger.io/specification/#tagObject
type openapiTagObject struct {
	Name         string                              `json:"name" yaml:"name"`
	Description  string                              `json:"description,omitempty" yaml:"description,omitempty"`
	ExternalDocs *openapiExternalDocumentationObject `json:"externalDocs,omitempty" yaml:"externalDocs,omitempty"`

	extensions []extension `json:"-" yaml:"-"`
}

// http://swagger.io/specification/#contactObject
type openapiContactObject struct {
	Name  string `json:"name,omitempty" yaml:"name,omitempty"`
	URL   string `json:"url,omitempty" yaml:"url,omitempty"`
	Email string `json:"email,omitempty" yaml:"email,omitempty"`
}

// http://swagger.io/specification/#licenseObject
type openapiLicenseObject struct {
	Name string `json:"name,omitempty" yaml:"name,omitempty"`
	URL  string `json:"url,omitempty" yaml:"url,omitempty"`
}

// http://swagger.io/specification/#externalDocumentationObject
type openapiExternalDocumentationObject struct {
	Description string `json:"description,omitempty" yaml:"description,omitempty"`
	URL         string `json:"url,omitempty" yaml:"url,omitempty"`
}

type extension struct {
	key   string          `json:"-" yaml:"-"`
	value json.RawMessage `json:"-" yaml:"-"`
}

// http://swagger.io/specification/#swaggerObject
type openapiSwaggerObject struct {
	Swagger             string                              `json:"swagger" yaml:"swagger"`
	Info                openapiInfoObject                   `json:"info" yaml:"info"`
	Tags                []openapiTagObject                  `json:"tags,omitempty" yaml:"tags,omitempty"`
	Host                string                              `json:"host,omitempty" yaml:"host,omitempty"`
	BasePath            string                              `json:"basePath,omitempty" yaml:"basePath,omitempty"`
	Schemes             []string                            `json:"schemes,omitempty" yaml:"schemes,omitempty"`
	Consumes            []string                            `json:"consumes" yaml:"consumes"`
	Produces            []string                            `json:"produces" yaml:"produces"`
	Paths               openapiPathsObject                  `json:"paths" yaml:"paths"`
	Definitions         openapiDefinitionsObject            `json:"definitions" yaml:"definitions"`
	SecurityDefinitions openapiSecurityDefinitionsObject    `json:"securityDefinitions,omitempty" yaml:"securityDefinitions,omitempty"`
	Security            []openapiSecurityRequirementObject  `json:"security,omitempty" yaml:"security,omitempty"`
	ExternalDocs        *openapiExternalDocumentationObject `json:"externalDocs,omitempty" yaml:"externalDocs,omitempty"`

	extensions []extension `json:"-" yaml:"-"`
}

// http://swagger.io/specification/#securityDefinitionsObject
type openapiSecurityDefinitionsObject map[string]openapiSecuritySchemeObject

// http://swagger.io/specification/#securitySchemeObject
type openapiSecuritySchemeObject struct {
	Type             string              `json:"type" yaml:"type"`
	Description      string              `json:"description,omitempty" yaml:"description,omitempty"`
	Name             string              `json:"name,omitempty" yaml:"name,omitempty"`
	In               string              `json:"in,omitempty" yaml:"in,omitempty"`
	Flow             string              `json:"flow,omitempty" yaml:"flow,omitempty"`
	AuthorizationURL string              `json:"authorizationUrl,omitempty" yaml:"authorizationUrl,omitempty"`
	TokenURL         string              `json:"tokenUrl,omitempty" yaml:"tokenUrl,omitempty"`
	Scopes           openapiScopesObject `json:"scopes,omitempty" yaml:"scopes,omitempty"`

	extensions []extension `json:"-" yaml:"-"`
}

// http://swagger.io/specification/#scopesObject
type openapiScopesObject map[string]string

// http://swagger.io/specification/#securityRequirementObject
type openapiSecurityRequirementObject map[string][]string

// http://swagger.io/specification/#pathsObject
type openapiPathsObject map[string]openapiPathItemObject

// http://swagger.io/specification/#pathItemObject
type openapiPathItemObject struct {
	Get     *openapiOperationObject `json:"get,omitempty" yaml:"get,omitempty"`
	Delete  *openapiOperationObject `json:"delete,omitempty" yaml:"delete,omitempty"`
	Post    *openapiOperationObject `json:"post,omitempty" yaml:"post,omitempty"`
	Put     *openapiOperationObject `json:"put,omitempty" yaml:"put,omitempty"`
	Patch   *openapiOperationObject `json:"patch,omitempty" yaml:"patch,omitempty"`
	Head    *openapiOperationObject `json:"head,omitempty" yaml:"head,omitempty"`
	Options *openapiOperationObject `json:"options,omitempty" yaml:"options,omitempty"`
	// While TRACE is supported in OpenAPI v3, it is not supported in OpenAPI v2
	// Trace   *openapiOperationObject `json:"trace,omitempty" yaml:"trace,omitempty"`
}

// http://swagger.io/specification/#operationObject
type openapiOperationObject struct {
	Summary     string                  `json:"summary,omitempty" yaml:"summary,omitempty"`
	Description string                  `json:"description,omitempty" yaml:"description,omitempty"`
	OperationID string                  `json:"operationId" yaml:"operationId"`
	Responses   openapiResponsesObject  `json:"responses" yaml:"responses"`
	Parameters  openapiParametersObject `json:"parameters,omitempty" yaml:"parameters,omitempty"`
	Tags        []string                `json:"tags,omitempty" yaml:"tags,omitempty"`
	Deprecated  bool                    `json:"deprecated,omitempty" yaml:"deprecated,omitempty"`
	Consumes    []string                `json:"consumes,omitempty" yaml:"consumes,omitempty"`
	Produces    []string                `json:"produces,omitempty" yaml:"produces,omitempty"`

	Security     *[]openapiSecurityRequirementObject `json:"security,omitempty" yaml:"security,omitempty"`
	ExternalDocs *openapiExternalDocumentationObject `json:"externalDocs,omitempty" yaml:"externalDocs,omitempty"`

	extensions []extension `json:"-" yaml:"-"`
}

type openapiParametersObject []openapiParameterObject

// http://swagger.io/specification/#parameterObject
type openapiParameterObject struct {
	Name             string              `json:"name" yaml:"name"`
	Description      string              `json:"description,omitempty" yaml:"description,omitempty"`
	In               string              `json:"in,omitempty" yaml:"in,omitempty"`
	Required         bool                `json:"required" yaml:"required"`
	Type             string              `json:"type,omitempty" yaml:"type,omitempty"`
	Format           string              `json:"format,omitempty" yaml:"format,omitempty"`
	Items            *openapiItemsObject `json:"items,omitempty" yaml:"items,omitempty"`
	Enum             []string            `json:"enum,omitempty" yaml:"enum,omitempty"`
	CollectionFormat string              `json:"collectionFormat,omitempty" yaml:"collectionFormat,omitempty"`
	Default          string              `json:"default,omitempty" yaml:"default,omitempty"`
	MinItems         *int                `json:"minItems,omitempty" yaml:"minItems,omitempty"`
	Pattern          string              `json:"pattern,omitempty" yaml:"pattern,omitempty"`

	// Or you can explicitly refer to another type. If this is defined all
	// other fields should be empty
	Schema *openapiSchemaObject `json:"schema,omitempty" yaml:"schema,omitempty"`

	extensions []extension
}

// core part of schema, which is common to itemsObject and schemaObject.
// http://swagger.io/specification/v2/#itemsObject
// The OAS3 spec (https://swagger.io/specification/#schemaObject) defines the
// `nullable` field as part of a Schema Object. This behavior has been
// "back-ported" to OAS2 as the Specification Extension `x-nullable`, and is
// supported by generation tools such as swagger-codegen and go-swagger.
// For protoc-gen-openapiv3, we'd want to add `nullable` instead.
type schemaCore struct {
	Type      string     `json:"type,omitempty" yaml:"type,omitempty"`
	Format    string     `json:"format,omitempty" yaml:"format,omitempty"`
	Ref       string     `json:"$ref,omitempty" yaml:"$ref,omitempty"`
	XNullable bool       `json:"x-nullable,omitempty" yaml:"x-nullable,omitempty"`
	Example   RawExample `json:"example,omitempty" yaml:"example,omitempty"`

	Items *openapiItemsObject `json:"items,omitempty" yaml:"items,omitempty"`

	// If the item is an enumeration include a list of all the *NAMES* of the
	// enum values.  I'm not sure how well this will work but assuming all enums
	// start from 0 index it will be great. I don't think that is a good assumption.
	Enum    []string `json:"enum,omitempty" yaml:"enum,omitempty"`
	Default string   `json:"default,omitempty" yaml:"default,omitempty"`
}

type allOfEntry struct {
	Ref string `json:"$ref,omitempty" yaml:"$ref,omitempty"`
}

type RawExample json.RawMessage

func (m RawExample) MarshalJSON() ([]byte, error) {
	return (json.RawMessage)(m).MarshalJSON()
}

func (m *RawExample) UnmarshalJSON(data []byte) error {
	return (*json.RawMessage)(m).UnmarshalJSON(data)
}

// MarshalYAML implements yaml.Marshaler interface.
//
// It converts RawExample to one of yaml-supported types and returns it.
//
// From yaml.Marshaler docs: The Marshaler interface may be implemented
// by types to customize their behavior when being marshaled into a YAML
// document. The returned value is marshaled in place of the original
// value implementing Marshaler.
func (e RawExample) MarshalYAML() (interface{}, error) {
	// From docs, json.Unmarshal will store one of next types to data:
	// - bool, for JSON booleans;
	// - float64, for JSON numbers;
	// - string, for JSON strings;
	// - []interface{}, for JSON arrays;
	// - map[string]interface{}, for JSON objects;
	// - nil for JSON null.
	var data interface{}
	if err := json.Unmarshal(e, &data); err != nil {
		return nil, err
	}

	return data, nil
}

func (s *schemaCore) setRefFromFQN(ref string, reg *descriptor.Registry) error {
	name, ok := fullyQualifiedNameToOpenAPIName(ref, reg)
	if !ok {
		return fmt.Errorf("setRefFromFQN: can't resolve OpenAPI name from %q", ref)
	}
	s.Ref = fmt.Sprintf("#/definitions/%s", name)
	return nil
}

type openapiItemsObject openapiSchemaObject

// http://swagger.io/specification/#responsesObject
type openapiResponsesObject map[string]openapiResponseObject

// http://swagger.io/specification/#responseObject
type openapiResponseObject struct {
	Description string                 `json:"description" yaml:"description"`
	Schema      openapiSchemaObject    `json:"schema" yaml:"schema"`
	Examples    map[string]interface{} `json:"examples,omitempty" yaml:"examples,omitempty"`
	Headers     openapiHeadersObject   `json:"headers,omitempty" yaml:"headers,omitempty"`

	extensions []extension `json:"-" yaml:"-"`
}

type openapiHeadersObject map[string]openapiHeaderObject

// http://swagger.io/specification/#headerObject
type openapiHeaderObject struct {
	Description string     `json:"description,omitempty" yaml:"description,omitempty"`
	Type        string     `json:"type,omitempty" yaml:"type,omitempty"`
	Format      string     `json:"format,omitempty" yaml:"format,omitempty"`
	Default     RawExample `json:"default,omitempty" yaml:"default,omitempty"`
	Pattern     string     `json:"pattern,omitempty" yaml:"pattern,omitempty"`
}

type keyVal struct {
	Key   string
	Value interface{}
}

type openapiSchemaObjectProperties []keyVal

func (p openapiSchemaObjectProperties) MarshalYAML() (interface{}, error) {
	n := yaml.Node{
		Kind:    yaml.MappingNode,
		Content: make([]*yaml.Node, len(p)*2),
	}
	for i, v := range p {
		keyNode := yaml.Node{}
		if err := keyNode.Encode(v.Key); err != nil {
			return nil, err
		}
		valueNode := yaml.Node{}
		if err := valueNode.Encode(v.Value); err != nil {
			return nil, err
		}
		n.Content[i*2+0] = &keyNode
		n.Content[i*2+1] = &valueNode
	}
	return n, nil
}

func (op openapiSchemaObjectProperties) MarshalJSON() ([]byte, error) {
	var buf bytes.Buffer
	buf.WriteString("{")
	for i, kv := range op {
		if i != 0 {
			buf.WriteString(",")
		}
		key, err := json.Marshal(kv.Key)
		if err != nil {
			return nil, err
		}
		buf.Write(key)
		buf.WriteString(":")
		val, err := json.Marshal(kv.Value)
		if err != nil {
			return nil, err
		}
		buf.Write(val)
	}

	buf.WriteString("}")
	return buf.Bytes(), nil
}

// http://swagger.io/specification/#schemaObject
type openapiSchemaObject struct {
	schemaCore `yaml:",inline"`
	// Properties can be recursively defined
	Properties           *openapiSchemaObjectProperties `json:"properties,omitempty" yaml:"properties,omitempty"`
	AdditionalProperties *openapiSchemaObject           `json:"additionalProperties,omitempty" yaml:"additionalProperties,omitempty"`

	Description string `json:"description,omitempty" yaml:"description,omitempty"`
	Title       string `json:"title,omitempty" yaml:"title,omitempty"`

	ExternalDocs *openapiExternalDocumentationObject `json:"externalDocs,omitempty" yaml:"externalDocs,omitempty"`

	ReadOnly         bool     `json:"readOnly,omitempty" yaml:"readOnly,omitempty"`
	MultipleOf       float64  `json:"multipleOf,omitempty" yaml:"multipleOf,omitempty"`
	Maximum          float64  `json:"maximum,omitempty" yaml:"maximum,omitempty"`
	ExclusiveMaximum bool     `json:"exclusiveMaximum,omitempty" yaml:"exclusiveMaximum,omitempty"`
	Minimum          float64  `json:"minimum,omitempty" yaml:"minimum,omitempty"`
	ExclusiveMinimum bool     `json:"exclusiveMinimum,omitempty" yaml:"exclusiveMinimum,omitempty"`
	MaxLength        uint64   `json:"maxLength,omitempty" yaml:"maxLength,omitempty"`
	MinLength        uint64   `json:"minLength,omitempty" yaml:"minLength,omitempty"`
	Pattern          string   `json:"pattern,omitempty" yaml:"pattern,omitempty"`
	MaxItems         uint64   `json:"maxItems,omitempty" yaml:"maxItems,omitempty"`
	MinItems         uint64   `json:"minItems,omitempty" yaml:"minItems,omitempty"`
	UniqueItems      bool     `json:"uniqueItems,omitempty" yaml:"uniqueItems,omitempty"`
	MaxProperties    uint64   `json:"maxProperties,omitempty" yaml:"maxProperties,omitempty"`
	MinProperties    uint64   `json:"minProperties,omitempty" yaml:"minProperties,omitempty"`
	Required         []string `json:"required,omitempty" yaml:"required,omitempty"`

	extensions []extension

	AllOf []allOfEntry `json:"allOf,omitempty" yaml:"allOf,omitempty"`
}

// http://swagger.io/specification/#definitionsObject
type openapiDefinitionsObject map[string]openapiSchemaObject

// Internal type mapping from FQMN to descriptor.Message. Used as a set by the
// findServiceMessages function.
type messageMap map[string]*descriptor.Message

// Internal type mapping from FQEN to descriptor.Enum. Used as a set by the
// findServiceMessages function.
type enumMap map[string]*descriptor.Enum

// Internal type to store used references.
type refMap map[string]struct{}
