//go:build go1.10 && codegen
// +build go1.10,codegen

package api

import (
	"encoding/json"
	"testing"

	"github.com/aws/aws-sdk-go/private/util"
)

func buildAPI() *API {
	a := &API{}

	stringShape := &Shape{
		API:       a,
		ShapeName: "string",
		Type:      "string",
	}
	stringShapeRef := &ShapeRef{
		API:       a,
		ShapeName: "string",
		Shape:     stringShape,
	}

	intShape := &Shape{
		API:       a,
		ShapeName: "int",
		Type:      "int",
	}
	intShapeRef := &ShapeRef{
		API:       a,
		ShapeName: "int",
		Shape:     intShape,
	}

	nestedComplexShape := &Shape{
		API:       a,
		ShapeName: "NestedComplexShape",
		MemberRefs: map[string]*ShapeRef{
			"NestedField": stringShapeRef,
		},
		Type: "structure",
	}

	nestedComplexShapeRef := &ShapeRef{
		API:       a,
		ShapeName: "NestedComplexShape",
		Shape:     nestedComplexShape,
	}

	nestedListShape := &Shape{
		API:       a,
		ShapeName: "NestedListShape",
		MemberRef: *nestedComplexShapeRef,
		Type:      "list",
	}

	nestedListShapeRef := &ShapeRef{
		API:       a,
		ShapeName: "NestedListShape",
		Shape:     nestedListShape,
	}

	complexShape := &Shape{
		API:       a,
		ShapeName: "ComplexShape",
		MemberRefs: map[string]*ShapeRef{
			"Field": stringShapeRef,
			"List":  nestedListShapeRef,
		},
		Type: "structure",
	}

	complexShapeRef := &ShapeRef{
		API:       a,
		ShapeName: "ComplexShape",
		Shape:     complexShape,
	}

	listShape := &Shape{
		API:       a,
		ShapeName: "ListShape",
		MemberRef: *complexShapeRef,
		Type:      "list",
	}

	listShapeRef := &ShapeRef{
		API:       a,
		ShapeName: "ListShape",
		Shape:     listShape,
	}

	listsShape := &Shape{
		API:       a,
		ShapeName: "ListsShape",
		MemberRef: *listShapeRef,
		Type:      "list",
	}

	listsShapeRef := &ShapeRef{
		API:       a,
		ShapeName: "ListsShape",
		Shape:     listsShape,
	}

	input := &Shape{
		API:       a,
		ShapeName: "FooInput",
		MemberRefs: map[string]*ShapeRef{
			"BarShape":     stringShapeRef,
			"ComplexField": complexShapeRef,
			"ListField":    listShapeRef,
			"ListsField":   listsShapeRef,
		},
		Type: "structure",
	}
	output := &Shape{
		API:       a,
		ShapeName: "FooOutput",
		MemberRefs: map[string]*ShapeRef{
			"BazShape":     intShapeRef,
			"ComplexField": complexShapeRef,
			"ListField":    listShapeRef,
			"ListsField":   listsShapeRef,
		},
		Type: "structure",
	}

	inputRef := ShapeRef{
		API:       a,
		ShapeName: "FooInput",
		Shape:     input,
	}
	outputRef := ShapeRef{
		API:       a,
		ShapeName: "FooOutput",
		Shape:     output,
	}

	operations := map[string]*Operation{
		"Foo": {
			API:          a,
			Name:         "Foo",
			ExportedName: "Foo",
			InputRef:     inputRef,
			OutputRef:    outputRef,
		},
	}

	a.Operations = operations
	a.Shapes = map[string]*Shape{
		"FooInput":           input,
		"FooOutput":          output,
		"string":             stringShape,
		"int":                intShape,
		"NestedComplexShape": nestedComplexShape,
		"NestedListShape":    nestedListShape,
		"ComplexShape":       complexShape,
		"ListShape":          listShape,
		"ListsShape":         listsShape,
	}
	a.Metadata = Metadata{
		ServiceAbbreviation: "FooService",
	}

	a.BaseImportPath = "github.com/aws/aws-sdk-go/service/"

	a.Setup()
	return a
}

func TestExampleGeneration(t *testing.T) {
	example := `{
  "version": "1.0",
  "examples": {
    "Foo": [
      {
        "input": {
          "BarShape": "Hello world",
          "ComplexField": {
            "Field": "bar",
            "List": [
              {
                "NestedField": "qux"
              }
            ]
          },
          "ListField": [
            {
              "Field": "baz"
            }
          ],
          "ListsField": [
            [
              {
                "Field": "baz"
              }
            ]
          ],
          "FieldDoesNotExistInModel": true
        },
        "output": {
          "BazShape": 1
        },
        "comments": {
          "input": {
          },
          "output": {
          }
        },
        "description": "Foo bar baz qux",
        "title": "I pity the foo"
      }
    ],
    "NotInTheModel": [
      {
        "input": {},
        "output": {},
        "comments": {
          "input": {
          },
          "output": {
          }
        },
        "description": "Look the other way",
        "title": "I am not modeled"
      }
    ]
  }
}
	`
	a := buildAPI()
	def := &ExamplesDefinition{}
	err := json.Unmarshal([]byte(example), def)
	if err != nil {
		t.Error(err)
	}
	def.API = a

	def.setup()
	expected := `import (
	"bytes"
	"fmt"
	"time"

	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/session"
	"github.com/aws/aws-sdk-go/service/fooservice"
)

var _ time.Duration
var _ bytes.Buffer

func ExampleFooService_Foo() {
	sess := session.Must(session.NewSession())

	svc := fooservice.New(sess)

	params := &fooservice.FooInput{
		BarShape: aws.String("string"),
		ComplexField: &fooservice.ComplexShape{
			Field: aws.String("string"),
			List: []*fooservice.NestedComplexShape{
				&fooservice.NestedComplexShape{ // Required
					NestedField: aws.String("string"),
				},
				// More values...
			},
		},
		ListField: []*fooservice.ComplexShape{
			&fooservice.ComplexShape{ // Required
				Field: aws.String("string"),
				List: []*fooservice.NestedComplexShape{
					&fooservice.NestedComplexShape{ // Required
						NestedField: aws.String("string"),
					},
					// More values...
				},
			},
			// More values...
		},
		ListsField: [][]*fooservice.ComplexShape{
			[]*fooservice.ComplexShape{ // Required
				&fooservice.ComplexShape{ // Required
					Field: aws.String("string"),
					List: []*fooservice.NestedComplexShape{
						&fooservice.NestedComplexShape{ // Required
							NestedField: aws.String("string"),
						},
						// More values...
					},
				},
				// More values...
			},
			// More values...
		},
	}
	resp, err := svc.Foo(params)

	if err != nil {
		// Print the error, cast err to awserr.Error to get the Code and
		// Message from an error.
		fmt.Println(err.Error())
		return
	}

	// Pretty-print the response data.
	fmt.Println(resp)
}`
	if e, a := util.GoFmt(expected), util.GoFmt(a.ExampleGoCode()); e != a {
		t.Errorf("Expect:\n%s\nActual:\n%s\n", e, a)
	}
}

func TestBuildShape(t *testing.T) {
	a := buildAPI()
	cases := []struct {
		defs     map[string]interface{}
		expected string
	}{
		{
			defs: map[string]interface{}{
				"barShape": "Hello World",
			},
			expected: "BarShape: aws.String(\"Hello World\"),\n",
		},
		{
			defs: map[string]interface{}{
				"BarShape": "Hello World",
			},
			expected: "BarShape: aws.String(\"Hello World\"),\n",
		},
	}

	for _, c := range cases {
		ref := a.Operations["Foo"].InputRef
		shapeStr := defaultExamplesBuilder{}.BuildShape(&ref, c.defs, false)
		if c.expected != shapeStr {
			t.Errorf("Expected:\n%s\nReceived:\n%s", c.expected, shapeStr)
		}
	}
}
