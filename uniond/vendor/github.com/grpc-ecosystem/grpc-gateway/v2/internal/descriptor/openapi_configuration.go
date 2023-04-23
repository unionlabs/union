package descriptor

import (
	"encoding/json"
	"fmt"
	"os"

	"github.com/grpc-ecosystem/grpc-gateway/v2/internal/descriptor/openapiconfig"
	"google.golang.org/protobuf/encoding/protojson"
	"gopkg.in/yaml.v3"
)

func loadOpenAPIConfigFromYAML(yamlFileContents []byte, yamlSourceLogName string) (*openapiconfig.OpenAPIConfig, error) {
	var yamlContents interface{}
	if err := yaml.Unmarshal(yamlFileContents, &yamlContents); err != nil {
		return nil, fmt.Errorf("failed to parse gRPC API Configuration from YAML in %q: %w", yamlSourceLogName, err)
	}

	jsonContents, err := json.Marshal(yamlContents)
	if err != nil {
		return nil, err
	}

	// Reject unknown fields because OpenAPIConfig is only used here
	unmarshaler := protojson.UnmarshalOptions{
		DiscardUnknown: false,
	}

	openapiConfiguration := openapiconfig.OpenAPIConfig{}
	if err := unmarshaler.Unmarshal(jsonContents, &openapiConfiguration); err != nil {
		return nil, fmt.Errorf("failed to parse OpenAPI Configuration from YAML in %q: %w", yamlSourceLogName, err)
	}

	return &openapiConfiguration, nil
}

func registerOpenAPIOptions(registry *Registry, openAPIConfig *openapiconfig.OpenAPIConfig, yamlSourceLogName string) error {
	if openAPIConfig.OpenapiOptions == nil {
		// Nothing to do
		return nil
	}

	if err := registry.RegisterOpenAPIOptions(openAPIConfig.OpenapiOptions); err != nil {
		return fmt.Errorf("failed to register option in %s: %w", yamlSourceLogName, err)
	}
	return nil
}

// LoadOpenAPIConfigFromYAML loads an  OpenAPI Configuration from the given YAML file
// and registers the OpenAPI options the given registry.
// This must be done after loading the proto file.
func (r *Registry) LoadOpenAPIConfigFromYAML(yamlFile string) error {
	yamlFileContents, err := os.ReadFile(yamlFile)
	if err != nil {
		return fmt.Errorf("failed to read OpenAPI Configuration description from %q: %w", yamlFile, err)
	}

	config, err := loadOpenAPIConfigFromYAML(yamlFileContents, yamlFile)
	if err != nil {
		return err
	}

	return registerOpenAPIOptions(r, config, yamlFile)
}
