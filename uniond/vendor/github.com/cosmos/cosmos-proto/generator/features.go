// Copyright (c) 2021 PlanetScale Inc. All rights reserved.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package generator

import (
	"fmt"
	"sort"

	"google.golang.org/protobuf/compiler/protogen"
)

var defaultFeatures = make(map[string]Feature)

func findFeatures(featureNames []string) ([]Feature, error) {
	required := make(map[string]Feature)
	for _, name := range featureNames {
		if name == "all" {
			required = defaultFeatures
			break
		}

		feat, ok := defaultFeatures[name]
		if !ok {
			return nil, fmt.Errorf("unknown feature: %q", name)
		}
		required[name] = feat
	}

	type namefeat struct {
		name string
		feat Feature
	}
	var sorted []namefeat
	for name, feat := range required {
		sorted = append(sorted, namefeat{name, feat})
	}
	sort.Slice(sorted, func(i, j int) bool {
		return sorted[i].name < sorted[j].name
	})

	var features []Feature
	for _, sp := range sorted {
		features = append(features, sp.feat)
	}
	return features, nil
}

func RegisterFeature(name string, feat Feature) {
	defaultFeatures[name] = feat
}

type Feature func(gen *GeneratedFile, plugin *protogen.Plugin) FeatureGenerator

type FeatureGenerator interface {
	GenerateFile(file *protogen.File, plugin *protogen.Plugin) bool
	GenerateHelpers()
}
