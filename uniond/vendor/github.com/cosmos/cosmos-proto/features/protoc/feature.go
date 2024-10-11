package protoc

import (
	"github.com/cosmos/cosmos-proto/generator"
	"google.golang.org/protobuf/compiler/protogen"
)

func init() {
	generator.RegisterFeature("protoc", func(gen *generator.GeneratedFile, plugin *protogen.Plugin) generator.FeatureGenerator {
		return protocGenGoFeature{
			Plugin: plugin,
			GeneratedFile: gen,
			once:          false,
		}
	})
}

type protocGenGoFeature struct {
	*protogen.Plugin
	*generator.GeneratedFile
	once bool
}

func (pg protocGenGoFeature) GenerateFile(file *protogen.File, plugin *protogen.Plugin) bool {
	GenerateFile(plugin, file, pg.GeneratedFile)
	return pg.once
}

func (pg  protocGenGoFeature) GenerateHelpers() {} //noop