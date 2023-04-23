package codegenerator

import (
	"google.golang.org/protobuf/compiler/protogen"
	"google.golang.org/protobuf/types/pluginpb"
)

func supportedCodeGeneratorFeatures() uint64 {
	// Enable support for optional keyword in proto3.
	return uint64(pluginpb.CodeGeneratorResponse_FEATURE_PROTO3_OPTIONAL)
}

// SetSupportedFeaturesOnPluginGen sets supported proto3 features
// on protogen.Plugin.
func SetSupportedFeaturesOnPluginGen(gen *protogen.Plugin) {
	gen.SupportedFeatures = supportedCodeGeneratorFeatures()
}

// SetSupportedFeaturesOnCodeGeneratorResponse sets supported proto3 features
// on pluginpb.CodeGeneratorResponse.
func SetSupportedFeaturesOnCodeGeneratorResponse(resp *pluginpb.CodeGeneratorResponse) {
	sf := supportedCodeGeneratorFeatures()
	resp.SupportedFeatures = &sf
}
