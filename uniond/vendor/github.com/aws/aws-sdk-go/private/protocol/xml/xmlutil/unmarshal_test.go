//go:build go1.7
// +build go1.7

package xmlutil

import (
	"encoding/xml"
	"fmt"
	"io"
	"math"
	"reflect"
	"strconv"
	"strings"
	"testing"

	"github.com/aws/aws-sdk-go/aws"
	"github.com/aws/aws-sdk-go/aws/awsutil"
)

type mockBody struct {
	DoneErr error
	Body    io.Reader
}

func (m *mockBody) Read(p []byte) (int, error) {
	n, err := m.Body.Read(p)
	if (n == 0 || err == io.EOF) && m.DoneErr != nil {
		return n, m.DoneErr
	}

	return n, err
}

type mockOutput struct {
	_       struct{}          `type:"structure"`
	String  *string           `type:"string"`
	Integer *int64            `type:"integer"`
	Float   *float64          `type:"double"`
	Nested  *mockNestedStruct `type:"structure"`
	List    []*mockListElem   `locationName:"List" locationNameList:"Elem" type:"list"`
	Closed  *mockClosedTags   `type:"structure"`
}
type mockNestedStruct struct {
	_            struct{} `type:"structure"`
	NestedString *string  `type:"string"`
	NestedInt    *int64   `type:"integer"`
}
type mockClosedTags struct {
	_    struct{} `type:"structure" xmlPrefix:"xsi" xmlURI:"http://www.w3.org/2001/XMLSchema-instance"`
	Attr *string  `locationName:"xsi:attrval" type:"string" xmlAttribute:"true"`
}
type mockListElem struct {
	_          struct{}            `type:"structure" xmlPrefix:"xsi" xmlURI:"http://www.w3.org/2001/XMLSchema-instance"`
	String     *string             `type:"string"`
	NestedElem *mockNestedListElem `type:"structure"`
}
type mockNestedListElem struct {
	_ struct{} `type:"structure" xmlPrefix:"xsi" xmlURI:"http://www.w3.org/2001/XMLSchema-instance"`

	String *string `type:"string"`
	Type   *string `locationName:"xsi:type" type:"string" xmlAttribute:"true"`
}

func TestUnmarshal(t *testing.T) {

	cases := []struct {
		Body     string
		Expect   mockOutput
		ExpectFn func(t *testing.T, actual mockOutput)
	}{
		{
			Body: `<?xml version="1.0" encoding="UTF-8"?>
<MockResponse xmlns="http://xmlns.example.com">
	<String>string value</String>
	<Integer>123</Integer>
	<Closed xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:attrval="attr value"/>
	<Nested>
		<NestedString>nested string value</NestedString>
		<NestedInt>321</NestedInt>
	</Nested>
	<List>
		<Elem>
			<NestedElem xmlns:xsi="http://www.w3.org/2001/XMLSchema-instance" xsi:type="type">
				<String>nested elem string value</String>
			</NestedElem>
			<String>elem string value</String>
		</Elem>
	</List>
</MockResponse>`,
			Expect: mockOutput{
				String:  aws.String("string value"),
				Integer: aws.Int64(123),
				Closed: &mockClosedTags{
					Attr: aws.String("attr value"),
				},
				Nested: &mockNestedStruct{
					NestedString: aws.String("nested string value"),
					NestedInt:    aws.Int64(321),
				},
				List: []*mockListElem{
					{
						String: aws.String("elem string value"),
						NestedElem: &mockNestedListElem{
							String: aws.String("nested elem string value"),
							Type:   aws.String("type"),
						},
					},
				},
			},
		},
		{
			Body:   `<?xml version="1.0" encoding="UTF-8"?><MockResponse xmlns="http://xmlns.example.com"><Float>123456789.123</Float></MockResponse>`,
			Expect: mockOutput{Float: aws.Float64(123456789.123)},
		},
		{
			Body: `<?xml version="1.0" encoding="UTF-8"?><MockResponse xmlns="http://xmlns.example.com"><Float>Infinity</Float></MockResponse>`,
			ExpectFn: func(t *testing.T, actual mockOutput) {
				if a := aws.Float64Value(actual.Float); !math.IsInf(a, 1) {
					t.Errorf("expect infinity, got %v", a)
				}
			},
		},
		{
			Body: `<?xml version="1.0" encoding="UTF-8"?><MockResponse xmlns="http://xmlns.example.com"><Float>-Infinity</Float></MockResponse>`,
			ExpectFn: func(t *testing.T, actual mockOutput) {
				if a := aws.Float64Value(actual.Float); !math.IsInf(a, -1) {
					t.Errorf("expect -infinity, got %v", a)
				}
			},
		},
		{
			Body: `<?xml version="1.0" encoding="UTF-8"?><MockResponse xmlns="http://xmlns.example.com"><Float>NaN</Float></MockResponse>`,
			ExpectFn: func(t *testing.T, actual mockOutput) {
				if a := aws.Float64Value(actual.Float); !math.IsNaN(a) {
					t.Errorf("expect NaN, got %v", a)
				}
			},
		},
	}

	for i, tt := range cases {
		t.Run(strconv.Itoa(i), func(t *testing.T) {
			actual := mockOutput{}
			decoder := xml.NewDecoder(strings.NewReader(tt.Body))
			err := UnmarshalXML(&actual, decoder, "")
			if err != nil {
				t.Fatalf("expect no error, got %v", err)
			}

			if tt.ExpectFn != nil {
				tt.ExpectFn(t, actual)
				return
			}

			if !reflect.DeepEqual(tt.Expect, actual) {
				t.Errorf("expect unmarshal to match\nExpect: %s\nActual: %s",
					awsutil.Prettify(tt.Expect), awsutil.Prettify(actual))
			}
		})
	}
}

func TestUnmarshal_UnexpectedEOF(t *testing.T) {
	const partialXMLBody = `<?xml version="1.0" encoding="UTF-8"?>
	<First>first value</First>
	<Second>Second val`

	out := struct {
		First  *string `locationName:"First" type:"string"`
		Second *string `locationName:"Second" type:"string"`
	}{}

	expect := out
	expect.First = aws.String("first")
	expect.Second = aws.String("second")

	expectErr := fmt.Errorf("expected read error")

	body := &mockBody{
		DoneErr: expectErr,
		Body:    strings.NewReader(partialXMLBody),
	}

	decoder := xml.NewDecoder(body)
	err := UnmarshalXML(&out, decoder, "")

	if err == nil {
		t.Fatalf("expect error, got none")
	}
	if e, a := expectErr, err; e != a {
		t.Errorf("expect %v error in %v, but was not", e, a)
	}
}
