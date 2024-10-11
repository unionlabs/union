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

package credentials_test

import (
	"log"
	"os"

	"cloud.google.com/go/auth/credentials"
	"cloud.google.com/go/auth/httptransport"
)

func ExampleDetectDefault() {
	creds, err := credentials.DetectDefault(&credentials.DetectOptions{
		Scopes: []string{"https://www.googleapis.com/auth/devstorage.full_control"},
	})
	if err != nil {
		log.Fatal(err)
	}
	client, err := httptransport.NewClient(&httptransport.Options{
		Credentials: creds,
	})
	if err != nil {
		log.Fatal(err)
	}
	client.Get("...")
}

func ExampleDetectDefault_withFilepath() {
	// Your credentials should be obtained from the Google
	// Developer Console (https://console.developers.google.com).
	// Navigate to your project, then see the "Credentials" page
	// under "APIs & Auth".
	// To create a service account client, click "Create new Client ID",
	// select "Service Account", and click "Create Client ID". A JSON
	// key file will then be downloaded to your computer.
	filepath := "/path/to/your-project-key.json"
	creds, err := credentials.DetectDefault(&credentials.DetectOptions{
		Scopes:          []string{"https://www.googleapis.com/auth/bigquery"},
		CredentialsFile: filepath,
	})
	if err != nil {
		log.Fatal(err)
	}
	client, err := httptransport.NewClient(&httptransport.Options{
		Credentials: creds,
	})
	if err != nil {
		log.Fatal(err)
	}
	client.Get("...")
}

func ExampleDetectDefault_withJSON() {
	data, err := os.ReadFile("/path/to/key-file.json")
	if err != nil {
		log.Fatal(err)
	}
	creds, err := credentials.DetectDefault(&credentials.DetectOptions{
		Scopes:          []string{"https://www.googleapis.com/auth/bigquery"},
		CredentialsJSON: data,
	})
	if err != nil {
		log.Fatal(err)
	}
	client, err := httptransport.NewClient(&httptransport.Options{
		Credentials: creds,
	})
	if err != nil {
		log.Fatal(err)
	}
	client.Get("...")
}
