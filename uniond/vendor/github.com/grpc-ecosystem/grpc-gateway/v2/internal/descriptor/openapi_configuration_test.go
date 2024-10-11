package descriptor

import (
	"strings"
	"testing"

	"github.com/grpc-ecosystem/grpc-gateway/v2/protoc-gen-openapiv2/options"
)

func TestLoadOpenAPIConfigFromYAMLRejectInvalidYAML(t *testing.T) {
	config, err := loadOpenAPIConfigFromYAML([]byte(`
openapiOptions:
file:
- file: test.proto
  - option:
      schemes:
        - HTTP
        - HTTPS
        - WSS
      securityDefinitions:
        security:
          ApiKeyAuth:
            type: TYPE_API_KEY
            in: IN_HEADER
            name: "X-API-Key"
`), "invalidyaml")
	if err == nil {
		t.Fatal(err)
	}

	if !strings.Contains(err.Error(), "line 3") {
		t.Errorf("Expected yaml error to be detected in line 3. Got other error: %v", err)
	}

	if config != nil {
		t.Fatal("Config returned")
	}
}

func TestLoadOpenAPIConfigFromYAML(t *testing.T) {
	config, err := loadOpenAPIConfigFromYAML([]byte(`
openapiOptions:
  file:
  - file: test.proto
    option:
      schemes:
      - HTTP
      - HTTPS
      - WSS
      securityDefinitions:
        security:
          ApiKeyAuth:
            type: TYPE_API_KEY
            in: IN_HEADER
            name: "X-API-Key"
`), "openapi_options")
	if err != nil {
		t.Fatal(err)
	}

	if config.OpenapiOptions == nil {
		t.Fatal("OpenAPIOptions is empty")
	}

	opts := config.OpenapiOptions
	if numFileOpts := len(opts.File); numFileOpts != 1 {
		t.Fatalf("expected 1 file option but got %d", numFileOpts)
	}

	fileOpt := opts.File[0]

	if fileOpt.File != "test.proto" {
		t.Fatalf("file option has unexpected binding %s", fileOpt.File)
	}

	swaggerOpt := fileOpt.Option

	if swaggerOpt == nil {
		t.Fatal("expected option to be set")
	}

	if numSchemes := len(swaggerOpt.Schemes); numSchemes != 3 {
		t.Fatalf("expected 3 schemes but got %d", numSchemes)
	}
	if swaggerOpt.Schemes[0] != options.Scheme_HTTP {
		t.Fatalf("expected first scheme to be HTTP but got %s", swaggerOpt.Schemes[0])
	}
	if swaggerOpt.Schemes[1] != options.Scheme_HTTPS {
		t.Fatalf("expected second scheme to be HTTPS but got %s", swaggerOpt.Schemes[1])
	}
	if swaggerOpt.Schemes[2] != options.Scheme_WSS {
		t.Fatalf("expected third scheme to be WSS but got %s", swaggerOpt.Schemes[2])
	}

	if swaggerOpt.SecurityDefinitions == nil {
		t.Fatal("expected securityDefinitions to be set")
	}
	if numSecOpts := len(swaggerOpt.SecurityDefinitions.Security); numSecOpts != 1 {
		t.Fatalf("expected 1 security option but got %d", numSecOpts)
	}
	secOpt, ok := swaggerOpt.SecurityDefinitions.Security["ApiKeyAuth"]
	if !ok {
		t.Fatal("no SecurityScheme for key \"ApiKeyAuth\"")
	}
	if secOpt.Type != options.SecurityScheme_TYPE_API_KEY {
		t.Fatalf("expected scheme type to be TYPE_API_KEY but got %s", secOpt.Type)
	}
	if secOpt.In != options.SecurityScheme_IN_HEADER {
		t.Fatalf("expected scheme  in to be IN_HEADER but got %s", secOpt.In)
	}
	if secOpt.Name != "X-API-Key" {
		t.Fatalf("expected name to be X-API-Key but got %s", secOpt.Name)
	}
}

func TestLoadOpenAPIConfigFromYAMLUnknownKeys(t *testing.T) {
	_, err := loadOpenAPIConfigFromYAML([]byte(`
closedapiOptions:
  get: it?
openapiOptions:
  file:
  - file: test.proto
    option:
      schemes:
      - HTTP
`), "openapi_options")
	if err == nil {
		t.Errorf("Expected invalid key error")
	}

}
