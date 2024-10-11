// Copyright 2019 Google LLC
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
	"context"
	"encoding/base64"
	"encoding/json"
	"fmt"
	"io/ioutil"
	"net/url"
	"os"
	"strconv"
	"strings"
	"testing"
	"time"

	storage_v1_tests "cloud.google.com/go/storage/internal/test/conformance"
	"github.com/google/go-cmp/cmp"
	"google.golang.org/api/option"
	"google.golang.org/protobuf/encoding/protojson"
)

func TestPostPolicyV4Conformance(t *testing.T) {
	oldUTCNow := utcNow
	defer func() {
		utcNow = oldUTCNow
	}()

	googleAccessID, privateKey, testFiles := parseFiles(t)

	for _, testFile := range testFiles {
		for _, tc := range testFile.PostPolicyV4Tests {
			t.Run(tc.Description, func(t *testing.T) {
				pin := tc.PolicyInput
				utcNow = func() time.Time {
					return time.Unix(pin.GetTimestamp().GetSeconds(), 0).UTC()
				}

				var style URLStyle
				switch pin.UrlStyle {
				case storage_v1_tests.UrlStyle_PATH_STYLE:
					style = PathStyle()
				case storage_v1_tests.UrlStyle_VIRTUAL_HOSTED_STYLE:
					style = VirtualHostedStyle()
				case storage_v1_tests.UrlStyle_BUCKET_BOUND_HOSTNAME:
					style = BucketBoundHostname(pin.BucketBoundHostname)
				}

				var conditions []PostPolicyV4Condition
				// Build the various conditions.
				pinConds := pin.Conditions
				if pinConds != nil {
					if clr := pinConds.ContentLengthRange; len(clr) > 0 {
						conditions = append(conditions, ConditionContentLengthRange(uint64(clr[0]), uint64(clr[1])))
					}
					if sw := pinConds.StartsWith; len(sw) > 0 {
						conditions = append(conditions, ConditionStartsWith(sw[0], sw[1]))
					}
				}

				metadata := make(map[string]string, len(pin.Fields))
				for key, value := range pin.Fields {
					if strings.HasPrefix(key, "x-goog-meta") {
						metadata[key] = value
					}
				}

				got, err := GenerateSignedPostPolicyV4(pin.Bucket, pin.Object, &PostPolicyV4Options{
					GoogleAccessID: googleAccessID,
					PrivateKey:     []byte(privateKey),
					Expires:        utcNow().Add(time.Duration(pin.Expiration) * time.Second),
					Style:          style,
					Insecure:       pin.Scheme == "http",
					Conditions:     conditions,
					Fields: &PolicyV4Fields{
						ACL:                    pin.Fields["acl"],
						CacheControl:           pin.Fields["cache-control"],
						ContentEncoding:        pin.Fields["content-encoding"],
						ContentDisposition:     pin.Fields["content-disposition"],
						ContentType:            pin.Fields["content-type"],
						Metadata:               metadata,
						RedirectToURLOnSuccess: strings.TrimSpace(pin.Fields["success_action_redirect"]),
						StatusCodeOnSuccess:    mustInt(t, pin.Fields["success_action_status"]),
					},
				})
				if err != nil {
					t.Fatal(err)
				}
				want := tc.PolicyOutput

				switch wantURL, err := url.Parse(want.Url); {
				case err != nil:
					t.Errorf("Failed to parse want.Url: %v", err)

				default:
					// Sort the headers.
					wantURL.RawQuery = wantURL.Query().Encode()
					if diff := cmp.Diff(got.URL, wantURL.String()); diff != "" {
						t.Errorf("URL mismatch: got - want +\n%s", diff)
					}
				}

				gotPolicy := b64Decode(t, got.Fields["policy"], "got")
				wantPolicy := want.ExpectedDecodedPolicy
				if diff := cmp.Diff(gotPolicy, wantPolicy); diff != "" {
					t.Fatalf("Policy mismatch: got - want +\n%s", diff)
				}
			})
		}
	}
}

func b64Decode(t *testing.T, b64Str, name string) string {
	dec, err := base64.StdEncoding.DecodeString(b64Str)
	if err != nil {
		t.Fatalf("%q: Base64 decoding failed: %v", name, err)
	}
	return string(dec)
}

func mustInt(t *testing.T, s string) int {
	if s == "" {
		return 0
	}
	i, err := strconv.ParseInt(s, 10, 64)
	if err != nil {
		t.Fatal(err)
	}
	return int(i)
}

func TestSigningV4Conformance(t *testing.T) {
	oldUTCNow := utcNow
	defer func() {
		utcNow = oldUTCNow
	}()

	googleAccessID, privateKey, testFiles := parseFiles(t)

	for _, testFile := range testFiles {
		for _, tc := range testFile.SigningV4Tests {
			t.Run(tc.Description, func(t *testing.T) {
				utcNow = func() time.Time {
					return time.Unix(tc.Timestamp.Seconds, 0).UTC()
				}

				qp := url.Values{}
				if tc.QueryParameters != nil {
					for k, v := range tc.QueryParameters {
						qp.Add(k, v)
					}
				}

				var style URLStyle
				switch tc.UrlStyle {
				case storage_v1_tests.UrlStyle_PATH_STYLE:
					style = PathStyle()
				case storage_v1_tests.UrlStyle_VIRTUAL_HOSTED_STYLE:
					style = VirtualHostedStyle()
				case storage_v1_tests.UrlStyle_BUCKET_BOUND_HOSTNAME:
					style = BucketBoundHostname(tc.BucketBoundHostname)
				}

				t.Setenv("STORAGE_EMULATOR_HOST", tc.EmulatorHostname)

				opts := []option.ClientOption{option.WithoutAuthentication()}
				if tc.ClientEndpoint != "" {
					opts = append(opts, option.WithEndpoint(tc.ClientEndpoint))
				}
				if tc.UniverseDomain != "" {
					opts = append(opts, option.WithUniverseDomain(tc.UniverseDomain))
				}

				c, err := NewClient(context.Background(), opts...)
				if err != nil {
					t.Fatalf("NewClient: %v", err)
				}

				gotURL, err := c.Bucket(tc.Bucket).SignedURL(tc.Object, &SignedURLOptions{
					GoogleAccessID:  googleAccessID,
					PrivateKey:      []byte(privateKey),
					Method:          tc.Method,
					Expires:         utcNow().Add(time.Duration(tc.Expiration) * time.Second),
					Scheme:          SigningSchemeV4,
					Headers:         headersAsSlice(tc.Headers),
					QueryParameters: qp,
					Style:           style,
					Insecure:        tc.Scheme == "http",
					Hostname:        tc.Hostname,
				})
				if err != nil {
					t.Fatal(err)
				}
				wantURL, err := url.Parse(tc.ExpectedUrl)
				if err != nil {
					t.Fatal(err)
				}
				// Sort the headers.
				wantURL.RawQuery = wantURL.Query().Encode()

				if gotURL != wantURL.String() {
					t.Fatalf("\nwant:\t%s\ngot:\t%s", wantURL.String(), gotURL)
				}
			})
		}
	}
}

func headersAsSlice(m map[string]string) []string {
	var s []string
	for k, v := range m {
		s = append(s, fmt.Sprintf("%s:%s", k, v))
	}
	return s
}

func parseFiles(t *testing.T) (googleAccessID, privateKey string, testFiles []*storage_v1_tests.TestFile) {
	dir := "internal/test/conformance"

	inBytes, err := ioutil.ReadFile(dir + "/service-account")
	if err != nil {
		t.Fatal(err)
	}
	serviceAccount := map[string]string{}
	if err := json.Unmarshal(inBytes, &serviceAccount); err != nil {
		t.Fatal(err)
	}
	googleAccessID = serviceAccount["client_email"]
	privateKey = serviceAccount["private_key"]

	files, err := os.ReadDir(dir)
	if err != nil {
		t.Fatal(err)
	}

	for _, f := range files {
		if !strings.HasSuffix(f.Name(), ".json") {
			continue
		}

		inBytes, err := os.ReadFile(dir + "/" + f.Name())
		if err != nil {
			t.Fatalf("%s: %v", f.Name(), err)
		}

		testFile := new(storage_v1_tests.TestFile)
		if err := protojson.Unmarshal(inBytes, testFile); err != nil {
			t.Fatalf("unmarshalling %s: %v", f.Name(), err)
		}
		testFiles = append(testFiles, testFile)
	}
	return
}
