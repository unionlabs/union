// Copyright (c) 2018, The GoGo Authors.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package main

import (
	"strings"

	"github.com/gogo/protobuf/vanity"
	"github.com/gogo/protobuf/vanity/command"
)

func main() {
	req := command.Read()
	files := req.GetProtoFile()
	files = vanity.FilterFiles(files, vanity.NotGoogleProtobufDescriptorProto)

	vanity.ForEachFile(files, vanity.TurnOnMarshalerAll)
	vanity.ForEachFile(files, vanity.TurnOnSizerAll)
	vanity.ForEachFile(files, vanity.TurnOnUnmarshalerAll)

	for _, file := range files {
		if strings.HasPrefix(file.GetName(), "google/rpc") {
			vanity.TurnOffGoEnumPrefixAll(file)
		}
	}

	vanity.ForEachFile(files, vanity.TurnOffGoEnumStringerAll)
	vanity.ForEachFile(files, vanity.TurnOnEnumStringerAll)

	vanity.ForEachFile(files, vanity.TurnOnEqualAll)
	vanity.ForEachFile(files, vanity.TurnOnGoStringAll)
	vanity.ForEachFile(files, vanity.TurnOffGoStringerAll)

	vanity.ForEachFile(files, vanity.TurnOnMessageNameAll)

	for _, file := range files {
		if strings.HasSuffix(file.GetName(), "annotations.proto") {
			continue
		}
		if strings.HasSuffix(file.GetName(), "datetime.proto") {
			continue
		}
		if strings.HasSuffix(file.GetName(), "http.proto") {
			continue
		}
		if strings.HasPrefix(file.GetName(), "google/api/expr/v1alpha1") {
			continue
		}
		if strings.HasSuffix(file.GetName(), "operations.proto") {
			continue
		}
		if strings.HasSuffix(file.GetName(), "viewport.proto") {
			continue
		}
		vanity.TurnOnCompareAll(file)
	}

	for _, file := range files {
		vanity.TurnOnStringerAll(file)
		vanity.TurnOnPopulateAll(file)
	}

	resp := command.Generate(req)
	command.Write(resp)
}
