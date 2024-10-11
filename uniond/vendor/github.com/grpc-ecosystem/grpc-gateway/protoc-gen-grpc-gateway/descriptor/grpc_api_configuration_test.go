package descriptor

import (
	"strings"
	"testing"
)

func TestLoadGrpcAPIServiceFromYAMLEmpty(t *testing.T) {
	service, err := loadGrpcAPIServiceFromYAML([]byte(``), "empty")
	if err != nil {
		t.Fatal(err)
	}

	if service == nil {
		t.Fatal("No service returned")
	}

	if service.HTTP != nil {
		t.Fatal("HTTP not empty")
	}
}

func TestLoadGrpcAPIServiceFromYAMLInvalidType(t *testing.T) {
	// Ideally this would fail but for now this test documents that it doesn't
	service, err := loadGrpcAPIServiceFromYAML([]byte(`type: not.the.right.type`), "invalidtype")
	if err != nil {
		t.Fatal(err)
	}

	if service == nil {
		t.Fatal("No service returned")
	}
}

func TestLoadGrpcAPIServiceFromYAMLSingleRule(t *testing.T) {
	service, err := loadGrpcAPIServiceFromYAML([]byte(`
type: google.api.Service
config_version: 3

http:
 rules:
 - selector: grpctest.YourService.Echo
   post: /v1/myecho
   body: "*"
`), "example")
	if err != nil {
		t.Fatal(err)
	}

	if service.HTTP == nil {
		t.Fatal("HTTP is empty")
	}

	if len(service.HTTP.GetRules()) != 1 {
		t.Fatalf("Have %v rules instead of one. Got: %v", len(service.HTTP.GetRules()), service.HTTP.GetRules())
	}

	rule := service.HTTP.GetRules()[0]
	if rule.GetSelector() != "grpctest.YourService.Echo" {
		t.Errorf("Rule has unexpected selector '%v'", rule.GetSelector())
	}
	if rule.GetPost() != "/v1/myecho" {
		t.Errorf("Rule has unexpected post '%v'", rule.GetPost())
	}
	if rule.GetBody() != "*" {
		t.Errorf("Rule has unexpected body '%v'", rule.GetBody())
	}
}

func TestLoadGrpcAPIServiceFromYAMLRejectInvalidYAML(t *testing.T) {
	service, err := loadGrpcAPIServiceFromYAML([]byte(`
type: google.api.Service
config_version: 3

http:
 rules:
 - selector: grpctest.YourService.Echo
   - post: thislinebreakstheselectorblockabovewiththeleadingdash
   body: "*"
`), "invalidyaml")
	if err == nil {
		t.Fatal(err)
	}

	if !strings.Contains(err.Error(), "line 7") {
		t.Errorf("Expected yaml error to be detected in line 7. Got other error: %v", err)
	}

	if service != nil {
		t.Fatal("Service returned")
	}
}

func TestLoadGrpcAPIServiceFromYAMLMultipleWithAdditionalBindings(t *testing.T) {
	service, err := loadGrpcAPIServiceFromYAML([]byte(`
type: google.api.Service
config_version: 3

http:
 rules:
 - selector: first.selector
   post: /my/post/path
   body: "*"
   additional_bindings:
   - post: /additional/post/path
   - put: /additional/put/{value}/path
   - delete: "{value}"
   - patch: "/additional/patch/{value}"
 - selector: some.other.service
   delete: foo
`), "example")
	if err != nil {
		t.Fatalf("Failed to load service description from YAML: %v", err)
	}

	if service == nil {
		t.Fatal("No service returned")
	}

	if service.HTTP == nil {
		t.Fatal("HTTP is empty")
	}

	if len(service.HTTP.GetRules()) != 2 {
		t.Fatalf("%v service(s) returned when two were expected. Got: %v", len(service.HTTP.GetRules()), service.HTTP)
	}

	first := service.HTTP.GetRules()[0]
	if first.GetSelector() != "first.selector" {
		t.Errorf("first.selector has unexpected selector '%v'", first.GetSelector())
	}
	if first.GetBody() != "*" {
		t.Errorf("first.selector has unexpected body '%v'", first.GetBody())
	}
	if first.GetPost() != "/my/post/path" {
		t.Errorf("first.selector has unexpected post '%v'", first.GetPost())
	}
	if len(first.GetAdditionalBindings()) != 4 {
		t.Fatalf("first.selector has unexpected number of bindings %v instead of four. Got: %v", len(first.GetAdditionalBindings()), first.GetAdditionalBindings())
	}
	if first.GetAdditionalBindings()[0].GetPost() != "/additional/post/path" {
		t.Errorf("first.selector additional binding 0 has unexpected post '%v'", first.GetAdditionalBindings()[0].GetPost())
	}
	if first.GetAdditionalBindings()[1].GetPut() != "/additional/put/{value}/path" {
		t.Errorf("first.selector additional binding 1 has unexpected put '%v'", first.GetAdditionalBindings()[0].GetPost())
	}
	if first.GetAdditionalBindings()[2].GetDelete() != "{value}" {
		t.Errorf("first.selector additional binding 2 has unexpected delete '%v'", first.GetAdditionalBindings()[0].GetPost())
	}
	if first.GetAdditionalBindings()[3].GetPatch() != "/additional/patch/{value}" {
		t.Errorf("first.selector additional binding 3 has unexpected patch '%v'", first.GetAdditionalBindings()[0].GetPost())
	}

	second := service.HTTP.GetRules()[1]
	if second.GetSelector() != "some.other.service" {
		t.Errorf("some.other.service has unexpected selector '%v'", second.GetSelector())
	}
	if second.GetDelete() != "foo" {
		t.Errorf("some.other.service has unexpected delete '%v'", second.GetDelete())
	}
	if len(second.GetAdditionalBindings()) != 0 {
		t.Errorf("some.other.service has %v additional bindings when it should not have any. Got: %v", len(second.GetAdditionalBindings()), second.GetAdditionalBindings())
	}
}
