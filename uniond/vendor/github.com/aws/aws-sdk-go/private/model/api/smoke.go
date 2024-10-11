//go:build codegen
// +build codegen

package api

import (
	"bytes"
	"encoding/json"
	"fmt"
	"os"
	"sort"
	"text/template"
)

// SmokeTestSuite defines the test suite for smoke tests.
type SmokeTestSuite struct {
	Version       int             `json:"version"`
	DefaultRegion string          `json:"defaultRegion"`
	TestCases     []SmokeTestCase `json:"testCases"`
}

// SmokeTestCase provides the definition for a integration smoke test case.
type SmokeTestCase struct {
	OpName    string                 `json:"operationName"`
	Input     map[string]interface{} `json:"input"`
	ExpectErr bool                   `json:"errorExpectedFromService"`
}

var smokeTestsCustomizations = map[string]func(*SmokeTestSuite) error{
	"sts":          stsSmokeTestCustomization,
	"waf":          wafSmokeTestCustomization,
	"wafregional":  wafRegionalSmokeTestCustomization,
	"iotdataplane": iotDataPlaneSmokeTestCustomization,
}

func iotDataPlaneSmokeTestCustomization(suite *SmokeTestSuite) error {
	suite.TestCases = []SmokeTestCase{}
	return nil
}

func wafSmokeTestCustomization(suite *SmokeTestSuite) error {
	return filterWAFCreateSqlInjectionMatchSet(suite)
}

func wafRegionalSmokeTestCustomization(suite *SmokeTestSuite) error {
	return filterWAFCreateSqlInjectionMatchSet(suite)
}

func filterWAFCreateSqlInjectionMatchSet(suite *SmokeTestSuite) error {
	const createSqlInjectionMatchSetOp = "CreateSqlInjectionMatchSet"

	var testCases []SmokeTestCase
	for _, testCase := range suite.TestCases {
		if testCase.OpName == createSqlInjectionMatchSetOp {
			continue
		}
		testCases = append(testCases, testCase)
	}

	suite.TestCases = testCases

	return nil
}

func stsSmokeTestCustomization(suite *SmokeTestSuite) error {
	const getSessionTokenOp = "GetSessionToken"
	const getCallerIdentityOp = "GetCallerIdentity"

	opTestMap := make(map[string][]SmokeTestCase)
	for _, testCase := range suite.TestCases {
		opTestMap[testCase.OpName] = append(opTestMap[testCase.OpName], testCase)
	}

	if _, ok := opTestMap[getSessionTokenOp]; ok {
		delete(opTestMap, getSessionTokenOp)
	}

	if _, ok := opTestMap[getCallerIdentityOp]; !ok {
		opTestMap[getCallerIdentityOp] = append(opTestMap[getCallerIdentityOp], SmokeTestCase{
			OpName:    getCallerIdentityOp,
			Input:     map[string]interface{}{},
			ExpectErr: false,
		})
	}

	var testCases []SmokeTestCase

	var keys []string
	for name := range opTestMap {
		keys = append(keys, name)
	}
	sort.Strings(keys)
	for _, name := range keys {
		testCases = append(testCases, opTestMap[name]...)
	}

	suite.TestCases = testCases

	return nil
}

// BuildInputShape returns the Go code as a string for initializing the test
// case's input shape.
func (c SmokeTestCase) BuildInputShape(ref *ShapeRef) string {
	b := NewShapeValueBuilder()
	return fmt.Sprintf("&%s{\n%s\n}",
		b.GoType(ref, true),
		b.BuildShape(ref, c.Input, false),
	)
}

// AttachSmokeTests attaches the smoke test cases to the API model.
func (a *API) AttachSmokeTests(filename string) error {
	f, err := os.Open(filename)
	if err != nil {
		return fmt.Errorf("failed to open smoke tests %s, err: %v", filename, err)
	}
	defer f.Close()

	if err := json.NewDecoder(f).Decode(&a.SmokeTests); err != nil {
		return fmt.Errorf("failed to decode smoke tests %s, err: %v", filename, err)
	}

	if v := a.SmokeTests.Version; v != 1 {
		return fmt.Errorf("invalid smoke test version, %d", v)
	}

	if fn, ok := smokeTestsCustomizations[a.PackageName()]; ok {
		if err := fn(&a.SmokeTests); err != nil {
			return err
		}
	}

	return nil
}

// APISmokeTestsGoCode returns the Go Code string for the smoke tests.
func (a *API) APISmokeTestsGoCode() string {
	w := bytes.NewBuffer(nil)

	a.resetImports()
	a.AddImport("context")
	a.AddImport("testing")
	a.AddImport("time")
	a.AddSDKImport("aws")
	a.AddSDKImport("aws/request")
	a.AddSDKImport("aws/awserr")
	a.AddSDKImport("aws/request")
	a.AddSDKImport("awstesting/integration")
	a.AddImport(a.ImportPath())

	smokeTests := struct {
		API *API
		SmokeTestSuite
	}{
		API:            a,
		SmokeTestSuite: a.SmokeTests,
	}

	if err := smokeTestTmpl.Execute(w, smokeTests); err != nil {
		panic(fmt.Sprintf("failed to create smoke tests, %v", err))
	}

	ignoreImports := `
	var _ aws.Config
	var _ awserr.Error
	var _ request.Request
	`

	return a.importsGoCode() + ignoreImports + w.String()
}

var smokeTestTmpl = template.Must(template.New(`smokeTestTmpl`).Parse(`
{{- range $i, $testCase := $.TestCases }}
	{{- $op := index $.API.Operations $testCase.OpName }}
	{{- if $op }}
	func TestInteg_{{ printf "%02d" $i }}_{{ $op.ExportedName }}(t *testing.T) {
		ctx, cancelFn := context.WithTimeout(context.Background(), 5 *time.Second)
		defer cancelFn()
	
		sess := integration.SessionWithDefaultRegion("{{ $.DefaultRegion }}")
		svc := {{ $.API.PackageName }}.New(sess)
		params := {{ $testCase.BuildInputShape $op.InputRef }}
		_, err := svc.{{ $op.ExportedName }}WithContext(ctx, params, func(r *request.Request) {
			r.Handlers.Validate.RemoveByName("core.ValidateParametersHandler")
		})
		{{- if $testCase.ExpectErr }}
			if err == nil {
				t.Fatalf("expect request to fail")
			}
			aerr, ok := err.(awserr.RequestFailure)
			if !ok {
				t.Fatalf("expect awserr, was %T", err)
			}
			if len(aerr.Code()) == 0 {
				t.Errorf("expect non-empty error code")
			}
			if len(aerr.Message()) == 0 {
				t.Errorf("expect non-empty error message")
			}
			if v := aerr.Code(); v == request.ErrCodeSerialization {
				t.Errorf("expect API error code got serialization failure")
			}
		{{- else }}
			if err != nil {
				t.Errorf("expect no error, got %v", err)
			}
		{{- end }}
	}
	{{- end }}
{{- end }}
`))
