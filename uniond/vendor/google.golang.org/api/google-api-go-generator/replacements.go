// Copyright 2018 Google LLC
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package main

// deprecatedPkgs is a map that contains packages that should be
// deprecated in favor of another package.
type deprecatedPkgs map[string]string

func (d deprecatedPkgs) Get(name, version string) string {
	v, ok := d[name]
	if !ok {
		// try lookup of a specific name-version
		return d[name+":"+version]
	}
	return v
}

// replacementPackage is a map from an API package name to the
// import path of the package that replaces it. If an API appears
// in this map, its package doc comment will note that it is deprecated
// and point to the replacement.
// TODO(jba): consider automating this by looking at the structure of the gocloud repo.
var replacementPackage deprecatedPkgs = map[string]string{
	"bigquery":          "cloud.google.com/go/bigquery",
	"cloudkms":          "cloud.google.com/go/kms/apiv1",
	"cloudtasks":        "cloud.google.com/go/cloudtasks/apiv2beta2",
	"dataproc":          "cloud.google.com/go/dataproc/apiv1",
	"datastore":         "cloud.google.com/go/datastore",
	"dialogflow":        "cloud.google.com/go/dialogflow/apiv2",
	"dlp":               "cloud.google.com/go/dlp/apiv2",
	"firestore":         "cloud.google.com/go/firestore",
	"language":          "cloud.google.com/go/language/apiv1",
	"logging":           "cloud.google.com/go/logging",
	"monitoring:v3":     "cloud.google.com/go/monitoring/apiv3",
	"oslogin":           "cloud.google.com/go/oslogin/apiv1",
	"pubsub":            "cloud.google.com/go/pubsub",
	"redis":             "cloud.google.com/go/redis/apiv1",
	"spanner":           "cloud.google.com/go/spanner",
	"speech":            "cloud.google.com/go/speech/apiv1",
	"texttospeech":      "cloud.google.com/go/texttospeech/apiv1",
	"translate":         "cloud.google.com/go/translate",
	"videointelligence": "cloud.google.com/go/videointelligence/apiv1",
	"vision":            "cloud.google.com/go/vision/apiv1",
	"storage":           "cloud.google.com/go/storage",
}
