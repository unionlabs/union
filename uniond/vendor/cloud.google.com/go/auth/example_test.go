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

package auth_test

import (
	"log"

	"cloud.google.com/go/auth"
	"cloud.google.com/go/auth/httptransport"
)

func ExampleNew2LOTokenProvider() {
	// Your credentials should be obtained from the Google
	// Developer Console (https://console.developers.google.com).
	opts := &auth.Options2LO{
		Email: "xxx@developer.gserviceaccount.com",
		// The contents of your RSA private key or your PEM file
		// that contains a private key.
		// If you have a p12 file instead, you
		// can use `openssl` to export the private key into a pem file.
		//
		//    $ openssl pkcs12 -in key.p12 -passin pass:notasecret -out key.pem -nodes
		//
		// The field only supports PEM containers with no passphrase.
		// The openssl command will convert p12 keys to passphrase-less PEM containers.
		PrivateKey: []byte("-----BEGIN RSA PRIVATE KEY-----..."),
		Scopes: []string{
			"https://www.googleapis.com/auth/bigquery",
			"https://www.googleapis.com/auth/blogger",
		},
		TokenURL: "https://oauth2.googleapis.com/token",
		// If you would like to impersonate a user, you can
		// create a transport with a subject. The following GET
		// request will be made on the behalf of user@example.com.
		// Optional.
		Subject: "user@example.com",
	}

	tp, err := auth.New2LOTokenProvider(opts)
	if err != nil {
		log.Fatal(err)
	}
	client, err := httptransport.NewClient(&httptransport.Options{
		Credentials: auth.NewCredentials(&auth.CredentialsOptions{
			TokenProvider: tp,
		}),
	})
	if err != nil {
		log.Fatal(err)
	}
	client.Get("...")
	_ = tp
}
