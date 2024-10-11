// Copyright (c) 2021 PlanetScale Inc. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package generator

import (
	"google.golang.org/protobuf/compiler/protogen"
	"google.golang.org/protobuf/reflect/protoreflect"
)

type featureHelpers struct {
	path    protogen.GoImportPath
	feature int
}

type Extensions struct {
	Poolable map[protogen.GoIdent]bool
}

type Generator struct {
	seen     map[featureHelpers]bool
	ext      *Extensions
	features []Feature
	local    map[string]bool
}

func NewGenerator(allFiles []*protogen.File, featureNames []string, ext *Extensions) (*Generator, error) {
	features, err := findFeatures(featureNames)

	if err != nil {
		return nil, err
	}

	local := make(map[string]bool)
	for _, f := range allFiles {
		if f.Generate {
			local[string(f.Desc.Package())] = true
		}
	}

	return &Generator{
		seen:     make(map[featureHelpers]bool),
		ext:      ext,
		features: features,
		local:    local,
	}, nil
}

func (gen *Generator) GenerateFile(plugin *protogen.Plugin, gf *protogen.GeneratedFile, file *protogen.File) bool {
	if file.Desc.Syntax() != protoreflect.Proto3 {
		return false
	}

	p := &GeneratedFile{
		GeneratedFile: gf,
		Ext:           gen.ext,
		LocalPackages: gen.local,
	}

	// DEPRECATED: this was used for our fork/copy of protoc-gen-go
	// GenerateProtocGenGo(plugin, p, file)

	var generated bool
	for fidx, feat := range gen.features {
		featGenerator := feat(p, plugin)
		if featGenerator.GenerateFile(file, plugin) {
			generated = true

			helpersForPlugin := featureHelpers{
				path:    file.GoImportPath,
				feature: fidx,
			}
			if !gen.seen[helpersForPlugin] {
				featGenerator.GenerateHelpers()
				gen.seen[helpersForPlugin] = true
			}
		}
	}

	return generated
}
