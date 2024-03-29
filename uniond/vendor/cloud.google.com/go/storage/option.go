// Copyright 2023 Google LLC
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//      http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package storage

import (
	"google.golang.org/api/option"
	"google.golang.org/api/option/internaloption"
)

// storageConfig contains the Storage client option configuration that can be
// set through storageClientOptions.
type storageConfig struct {
	useJSONforReads bool
	readAPIWasSet   bool
}

// newStorageConfig generates a new storageConfig with all the given
// storageClientOptions applied.
func newStorageConfig(opts ...option.ClientOption) storageConfig {
	var conf storageConfig
	for _, opt := range opts {
		if storageOpt, ok := opt.(storageClientOption); ok {
			storageOpt.ApplyStorageOpt(&conf)
		}
	}
	return conf
}

// A storageClientOption is an option for a Google Storage client.
type storageClientOption interface {
	option.ClientOption
	ApplyStorageOpt(*storageConfig)
}

// WithJSONReads is an option that may be passed to a Storage Client on creation.
// It sets the client to use the JSON API for object reads. Currently, the
// default API used for reads is XML.
// Setting this option is required to use the GenerationNotMatch condition.
//
// Note that when this option is set, reads will return a zero date for
// [ReaderObjectAttrs].LastModified and may return a different value for
// [ReaderObjectAttrs].CacheControl.
func WithJSONReads() option.ClientOption {
	return &withReadAPI{useJSON: true}
}

// WithXMLReads is an option that may be passed to a Storage Client on creation.
// It sets the client to use the XML API for object reads.
//
// This is the current default.
func WithXMLReads() option.ClientOption {
	return &withReadAPI{useJSON: false}
}

type withReadAPI struct {
	internaloption.EmbeddableAdapter
	useJSON bool
}

func (w *withReadAPI) ApplyStorageOpt(c *storageConfig) {
	c.useJSONforReads = w.useJSON
	c.readAPIWasSet = true
}
