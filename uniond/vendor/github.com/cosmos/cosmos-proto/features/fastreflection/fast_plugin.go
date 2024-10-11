package fastreflection

import (
	"github.com/cosmos/cosmos-proto/generator"
	"google.golang.org/protobuf/compiler/protogen"
)

func init() {
	generator.RegisterFeature("fast", func(gen *generator.GeneratedFile, _ *protogen.Plugin) generator.FeatureGenerator {
		return fastReflectionFeature{
			GeneratedFile: gen,
			Stable:        false,
			once:          false,
		}
	})
}

type fastReflectionFeature struct {
	*generator.GeneratedFile
	Stable, once bool
}

type fastGenerator struct {
	*generator.GeneratedFile
	file    *protogen.File
	message *protogen.Message

	Stable   bool
	typeName string
	err      error
}

func newGenerator(f *protogen.File, g *generator.GeneratedFile, message *protogen.Message) *fastGenerator {
	return &fastGenerator{
		GeneratedFile: g,
		file:          f,
		message:       message,
		Stable:        true,
		typeName:      fastReflectionTypeName(message),
		err:           nil,
	}
}

func (g fastReflectionFeature) GenerateFile(file *protogen.File, _ *protogen.Plugin) bool {
	for _, msg := range file.Messages {
		GenProtoMessage(file, g.GeneratedFile, msg)
	}
	return true // only do this once
}

func (g fastReflectionFeature) GenerateHelpers() {
	// no helpers needed here yet
}
