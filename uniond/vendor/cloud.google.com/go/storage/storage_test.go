// Copyright 2014 Google LLC
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
	"crypto/tls"
	"encoding/json"
	"fmt"
	"io"
	"io/ioutil"
	"log"
	"net"
	"net/http"
	"net/http/httptest"
	"net/url"
	"os"
	"reflect"
	"regexp"
	"sort"
	"strings"
	"testing"
	"time"

	"cloud.google.com/go/iam"
	"cloud.google.com/go/internal/testutil"
	"cloud.google.com/go/storage/internal/apiv2/storagepb"
	"github.com/google/go-cmp/cmp"
	"github.com/googleapis/gax-go/v2"
	"google.golang.org/api/iterator"
	"google.golang.org/api/option"
	raw "google.golang.org/api/storage/v1"
	"google.golang.org/protobuf/proto"
	"google.golang.org/protobuf/types/known/timestamppb"
)

func TestV2HeaderSanitization(t *testing.T) {
	t.Parallel()
	var tests = []struct {
		desc string
		in   []string
		want []string
	}{
		{
			desc: "already sanitized headers should not be modified",
			in:   []string{"x-goog-header1:true", "x-goog-header2:0"},
			want: []string{"x-goog-header1:true", "x-goog-header2:0"},
		},
		{
			desc: "sanitized headers should be sorted",
			in:   []string{"x-goog-header2:0", "x-goog-header1:true"},
			want: []string{"x-goog-header1:true", "x-goog-header2:0"},
		},
		{
			desc: "non-canonical headers should be removed",
			in:   []string{"x-goog-header1:true", "x-goog-no-value", "non-canonical-header:not-of-use"},
			want: []string{"x-goog-header1:true"},
		},
		{
			desc: "excluded canonical headers should be removed",
			in:   []string{"x-goog-header1:true", "x-goog-encryption-key:my_key", "x-goog-encryption-key-sha256:my_sha256"},
			want: []string{"x-goog-header1:true"},
		},
		{
			desc: "dirty headers should be formatted correctly",
			in:   []string{" x-goog-header1 : \textra-spaces ", "X-Goog-Header2:CamelCaseValue"},
			want: []string{"x-goog-header1:extra-spaces", "x-goog-header2:CamelCaseValue"},
		},
		{
			desc: "duplicate headers should be merged",
			in:   []string{"x-goog-header1:value1", "X-Goog-Header1:value2"},
			want: []string{"x-goog-header1:value1,value2"},
		},
	}
	for _, test := range tests {
		got := v2SanitizeHeaders(test.in)
		if !testutil.Equal(got, test.want) {
			t.Errorf("%s: got %v, want %v", test.desc, got, test.want)
		}
	}
}

func TestV4HeaderSanitization(t *testing.T) {
	t.Parallel()
	var tests = []struct {
		desc string
		in   []string
		want []string
	}{
		{
			desc: "already sanitized headers should not be modified",
			in:   []string{"x-goog-header1:true", "x-goog-header2:0"},
			want: []string{"x-goog-header1:true", "x-goog-header2:0"},
		},
		{
			desc: "dirty headers should be formatted correctly",
			in:   []string{" x-goog-header1 : \textra-spaces ", "X-Goog-Header2:CamelCaseValue"},
			want: []string{"x-goog-header1:extra-spaces", "x-goog-header2:CamelCaseValue"},
		},
		{
			desc: "duplicate headers should be merged",
			in:   []string{"x-goog-header1:value1", "X-Goog-Header1:value2"},
			want: []string{"x-goog-header1:value1,value2"},
		},
		{
			desc: "multiple spaces in value are stripped down to one",
			in:   []string{"foo:bar        gaz"},
			want: []string{"foo:bar gaz"},
		},
		{
			desc: "headers with colons in value are preserved",
			in:   []string{"x-goog-meta-start-time: 2023-02-10T02:00:00Z"},
			want: []string{"x-goog-meta-start-time:2023-02-10T02:00:00Z"},
		},
		{
			desc: "headers that end in a colon in value are preserved",
			in:   []string{"x-goog-meta-start-time: 2023-02-10T02:"},
			want: []string{"x-goog-meta-start-time:2023-02-10T02:"},
		},
	}
	for _, test := range tests {
		got := v4SanitizeHeaders(test.in)
		sort.Strings(got)
		sort.Strings(test.want)
		if !testutil.Equal(got, test.want) {
			t.Errorf("%s: got %v, want %v", test.desc, got, test.want)
		}
	}
}

func TestSignedURLV2(t *testing.T) {
	expires, _ := time.Parse(time.RFC3339, "2002-10-02T10:00:00-05:00")

	tests := []struct {
		desc       string
		objectName string
		opts       *SignedURLOptions
		want       string
	}{
		{
			desc:       "SignedURLV2 works",
			objectName: "object-name",
			opts: &SignedURLOptions{
				GoogleAccessID: "xxx@clientid",
				PrivateKey:     dummyKey("rsa"),
				Method:         "GET",
				MD5:            "ICy5YqxZB1uWSwcVLSNLcA==",
				Expires:        expires,
				ContentType:    "application/json",
				Headers:        []string{"x-goog-header1:true", "x-goog-header2:false"},
			},
			want: "https://storage.googleapis.com/bucket-name/object-name?" +
				"Expires=1033570800&GoogleAccessId=xxx%40clientid&Signature=" +
				"RfsHlPtbB2JUYjzCgNr2Mi%2BjggdEuL1V7E6N9o6aaqwVLBDuTv3I0%2B9" +
				"x94E6rmmr%2FVgnmZigkIUxX%2Blfl7LgKf30uPGLt0mjKGH2p7r9ey1ONJ" +
				"%2BhVec23FnTRcSgopglvHPuCMWU2oNJE%2F1y8EwWE27baHrG1RhRHbLVF" +
				"bPpLZ9xTRFK20pluIkfHV00JGljB1imqQHXM%2B2XPWqBngLr%2FwqxLN7i" +
				"FcUiqR8xQEOHF%2F2e7fbkTHPNq4TazaLZ8X0eZ3eFdJ55A5QmNi8atlN4W" +
				"5q7Hvs0jcxElG3yqIbx439A995BkspLiAcA%2Fo4%2BxAwEMkGLICdbvakq" +
				"3eEprNCojw%3D%3D",
		},
		{
			desc:       "With a PEM Private Key",
			objectName: "object-name",
			opts: &SignedURLOptions{
				GoogleAccessID: "xxx@clientid",
				PrivateKey:     dummyKey("pem"),
				Method:         "GET",
				MD5:            "ICy5YqxZB1uWSwcVLSNLcA==",
				Expires:        expires,
				ContentType:    "application/json",
				Headers:        []string{"x-goog-header1:true", "x-goog-header2:false"},
			},
			want: "https://storage.googleapis.com/bucket-name/object-name?" +
				"Expires=1033570800&GoogleAccessId=xxx%40clientid&Signature=" +
				"TiyKD%2FgGb6Kh0kkb2iF%2FfF%2BnTx7L0J4YiZua8AcTmnidutePEGIU5" +
				"NULYlrGl6l52gz4zqFb3VFfIRTcPXMdXnnFdMCDhz2QuJBUpsU1Ai9zlyTQ" +
				"dkb6ShG03xz9%2BEXWAUQO4GBybJw%2FULASuv37xA00SwLdkqj8YdyS5II" +
				"1lro%3D",
		},
		{
			desc:       "With custom SignBytes",
			objectName: "object-name",
			opts: &SignedURLOptions{
				GoogleAccessID: "xxx@clientid",
				SignBytes: func(b []byte) ([]byte, error) {
					return []byte("signed"), nil
				},
				Method:      "GET",
				MD5:         "ICy5YqxZB1uWSwcVLSNLcA==",
				Expires:     expires,
				ContentType: "application/json",
				Headers:     []string{"x-goog-header1:true", "x-goog-header2:false"},
			},
			want: "https://storage.googleapis.com/bucket-name/object-name?" +
				"Expires=1033570800&GoogleAccessId=xxx%40clientid&Signature=" +
				"c2lnbmVk", // base64('signed') == 'c2lnbmVk'
		},
		{
			desc:       "With unsafe object name",
			objectName: "object name界",
			opts: &SignedURLOptions{
				GoogleAccessID: "xxx@clientid",
				PrivateKey:     dummyKey("pem"),
				Method:         "GET",
				MD5:            "ICy5YqxZB1uWSwcVLSNLcA==",
				Expires:        expires,
				ContentType:    "application/json",
				Headers:        []string{"x-goog-header1:true", "x-goog-header2:false"},
			},
			want: "https://storage.googleapis.com/bucket-name/object%20name%E7%95%8C?" +
				"Expires=1033570800&GoogleAccessId=xxx%40clientid&Signature=bxVH1%2Bl%2" +
				"BSxpnj3XuqKz6mOFk6M94Y%2B4w85J6FCmJan%2FNhGSpndP6fAw1uLHlOn%2F8xUaY%2F" +
				"SfZ5GzcQ%2BbxOL1WA37yIwZ7xgLYlO%2ByAi3GuqMUmHZiNCai28emODXQ8RtWHvgv6dE" +
				"SQ%2F0KpDMIWW7rYCaUa63UkUyeSQsKhrVqkIA%3D",
		},
	}

	for _, test := range tests {
		u, err := SignedURL("bucket-name", test.objectName, test.opts)
		if err != nil {
			t.Fatalf("[%s] %v", test.desc, err)
		}
		if u != test.want {
			t.Fatalf("[%s] Unexpected signed URL; found %v", test.desc, u)
		}
	}
}

func TestSignedURLV4(t *testing.T) {
	expires, _ := time.Parse(time.RFC3339, "2002-10-02T10:00:00-05:00")

	tests := []struct {
		desc       string
		objectName string
		now        time.Time
		opts       *SignedURLOptions
		// Note for future implementors: X-Goog-Signature generated by having
		// the client run through its algorithm with pre-defined input and copy
		// pasting the output. These tests are not great for testing whether
		// the right signature is calculated - instead we rely on the backend
		// and integration tests for that.
		want string
	}{
		{
			desc:       "SignURLV4 works",
			objectName: "object-name",
			now:        expires.Add(-24 * time.Hour),
			opts: &SignedURLOptions{
				GoogleAccessID: "xxx@clientid",
				PrivateKey:     dummyKey("rsa"),
				Method:         "POST",
				Expires:        expires,
				Scheme:         SigningSchemeV4,
				ContentType:    "application/json",
				MD5:            "ICy5YqxZB1uWSwcVLSNLcA==",
				Headers:        []string{"x-goog-header1:true", "x-goog-header2:false"},
			},
			want: "https://storage.googleapis.com/bucket-name/object-name" +
				"?X-Goog-Algorithm=GOOG4-RSA-SHA256" +
				"&X-Goog-Credential=xxx%40clientid%2F20021001%2Fauto%2Fstorage%2Fgoog4_request" +
				"&X-Goog-Date=20021001T100000Z&X-Goog-Expires=86400" +
				"&X-Goog-Signature=774b11d89663d0562b0909131b8495e70d24e31f3417d3f8fd1438a72b620b256111a7221fecab14a6ebb7dc7eed7984316a794789beb4ecdda67a77407f6de1a68113e8fa2b885e330036a995c08f0f2a7d2c212a3d0a2fd1b392d40305d3fe31ab94c547a7541278f4a956ebb6565ebe4cb27f26e30b334adb7b065adc0d27f9eaa42ee76d75d673fc4523d023d9a636de0b5329f5dffbf80024cf21fdc6236e89aa41976572bfe4807be9a9a01f644ed9f546dcf1e0394665be7610f58c36b3d63379f4d1b64f646f7427f1fc55bb89d7fdd59017d007156c99e26440e828581cddf83faf03e739e5987c062d503f2b73f24049c25edc60ecbbc09f6ce945" +
				"&X-Goog-SignedHeaders=content-md5%3Bcontent-type%3Bhost%3Bx-goog-header1%3Bx-goog-header2",
		},
		{
			desc:       "With PEM Private Key",
			objectName: "object-name",
			now:        expires.Add(-24 * time.Hour),
			opts: &SignedURLOptions{
				GoogleAccessID: "xxx@clientid",
				PrivateKey:     dummyKey("pem"),
				Method:         "GET",
				Expires:        expires,
				Scheme:         SigningSchemeV4,
			},
			want: "https://storage.googleapis.com/bucket-name/object-name" +
				"?X-Goog-Algorithm=GOOG4-RSA-SHA256" +
				"&X-Goog-Credential=xxx%40clientid%2F20021001%2Fauto%2Fstorage%2Fgoog4_request" +
				"&X-Goog-Date=20021001T100000Z&X-Goog-Expires=86400" +
				"&X-Goog-Signature=5592f4b8b2cae14025b619546d69bb463ca8f2caaab538a3cc6b5868c8c64b83a8b04b57d8a82c8696a192f62abddc8d99e0454b3fc33feac5bf87c353f0703aab6cfee60364aaeecec2edd37c1d6e6793d90812b5811b7936a014a3efad5d08477b4fbfaebf04fa61f1ca03f31bcdc46a161868cd2f4e98def6c82634a01454" +
				"&X-Goog-SignedHeaders=host",
		},
		{
			desc:       "Unsafe object name",
			objectName: "object name界",
			now:        expires.Add(-24 * time.Hour),
			opts: &SignedURLOptions{
				GoogleAccessID: "xxx@clientid",
				PrivateKey:     dummyKey("pem"),
				Method:         "GET",
				Expires:        expires,
				Scheme:         SigningSchemeV4,
			},
			want: "https://storage.googleapis.com/bucket-name/object%20name%E7%95%8C" +
				"?X-Goog-Algorithm=GOOG4-RSA-SHA256" +
				"&X-Goog-Credential=xxx%40clientid%2F20021001%2Fauto%2Fstorage%2Fgoog4_request" +
				"&X-Goog-Date=20021001T100000Z&X-Goog-Expires=86400" +
				"&X-Goog-Signature=90fd455fb47725b45c08d65ddf99078184710ad30f09bc2a190c5416ba1596e4c58420e2e48744b03de2d1b85dc8679dcb4c36af6e7a1b2547cd62becaad72aebbbaf7c1686f1aa0fedf8a9b01cef20a8b8630d824a6f8b81bb9eb75f342a7d8a28457a4efd2baac93e37089b84b1506b2af72712187f638e0eafbac650b071a" +
				"&X-Goog-SignedHeaders=host",
		},
		{
			desc:       "With custom SignBytes",
			objectName: "object-name",
			now:        expires.Add(-24 * time.Hour),
			opts: &SignedURLOptions{
				GoogleAccessID: "xxx@clientid",
				SignBytes: func(b []byte) ([]byte, error) {
					return []byte("signed"), nil
				},
				Method:  "GET",
				Expires: expires,
				Scheme:  SigningSchemeV4,
			},
			want: "https://storage.googleapis.com/bucket-name/object-name" +
				"?X-Goog-Algorithm=GOOG4-RSA-SHA256" +
				"&X-Goog-Credential=xxx%40clientid%2F20021001%2Fauto%2Fstorage%2Fgoog4_request" +
				"&X-Goog-Date=20021001T100000Z&X-Goog-Expires=86400" +
				"&X-Goog-Signature=7369676e6564" + // hex('signed') = '7369676e6564'
				"&X-Goog-SignedHeaders=host",
		},
	}
	oldUTCNow := utcNow
	defer func() {
		utcNow = oldUTCNow
	}()

	for _, test := range tests {
		t.Logf("Testcase: '%s'", test.desc)

		utcNow = func() time.Time {
			return test.now
		}
		got, err := SignedURL("bucket-name", test.objectName, test.opts)
		if err != nil {
			t.Fatal(err)
		}
		if got != test.want {
			t.Fatalf("\n\tgot:\t%v\n\twant:\t%v", got, test.want)
		}
	}
}

// TestSignedURL_EmulatorHost tests that SignedURl respects the host set in
// STORAGE_EMULATOR_HOST
func TestSignedURL_EmulatorHost(t *testing.T) {
	expires, _ := time.Parse(time.RFC3339, "2002-10-02T10:00:00-05:00")
	bucketName := "bucket-name"
	objectName := "obj-name"

	emulatorHost := os.Getenv("STORAGE_EMULATOR_HOST")
	defer os.Setenv("STORAGE_EMULATOR_HOST", emulatorHost)

	tests := []struct {
		desc         string
		emulatorHost string
		now          time.Time
		opts         *SignedURLOptions
		// Note for future implementors: X-Goog-Signature generated by having
		// the client run through its algorithm with pre-defined input and copy
		// pasting the output. These tests are not great for testing whether
		// the right signature is calculated - instead we rely on the backend
		// and integration tests for that.
		want string
	}{
		{
			desc:         "SignURLV4 creates link to resources in emulator",
			emulatorHost: "localhost:9000",
			now:          expires.Add(-24 * time.Hour),
			opts: &SignedURLOptions{
				GoogleAccessID: "xxx@clientid",
				PrivateKey:     dummyKey("rsa"),
				Method:         "POST",
				Expires:        expires,
				Scheme:         SigningSchemeV4,
				Insecure:       true,
			},
			want: "http://localhost:9000/" + bucketName + "/" + objectName +
				"?X-Goog-Algorithm=GOOG4-RSA-SHA256" +
				"&X-Goog-Credential=xxx%40clientid%2F20021001%2Fauto%2Fstorage%2Fgoog4_request" +
				"&X-Goog-Date=20021001T100000Z&X-Goog-Expires=86400" +
				"&X-Goog-Signature=249c53142e57adf594b4f523a8a1f9c15f29b071e9abc0cf6665dbc5f692fc96fac4ab98bbea4c2397384367bc970a2e1771f2c86624475f3273970ecde8ff6df39d647e5c3f3263bf67a743e211c1958a96775edf53ece1f69ed337f0ab7fdc081c6c2b84e57b0922280d27f1da1bff47e77e3822fb1756e4c5cece9d220e6d0824ab9528e97e54f0cb09b352193b0e895344d894de11b3f5f9a2ec7d8fd6d0a4c487afd1896385a3ab9e8c3fcb3862ec0cad6ec10af1b574078eb7c79b558bcd85449a67079a0ee6da97fcbad074f1bf9fdfbdca12945336a8bd0a3b70b4c7708918cb83d10c7c4ff1f8b73275e9d1ba5d3db91069dffdf81eb7badf4e3c80" +
				"&X-Goog-SignedHeaders=host",
		},
		{
			desc:         "using SigningSchemeV2",
			emulatorHost: "localhost:9000",
			now:          expires.Add(-24 * time.Hour),
			opts: &SignedURLOptions{
				GoogleAccessID: "xxx@clientid",
				PrivateKey:     dummyKey("rsa"),
				Method:         "POST",
				Expires:        expires,
				Scheme:         SigningSchemeV2,
			},
			want: "https://localhost:9000/" + bucketName + "/" + objectName +
				"?Expires=1033570800" +
				"&GoogleAccessId=xxx%40clientid" +
				"&Signature=oRi3y2tBTmoDto7FezNx4AjC0RXA6fpJjTBa0hINeVroZ%2ByOeRU8MRwJbKg1IkBbV0IjtlPaGwv5YoUH16UYdipBjCXOS%2B1qgRWyzl8AnzvU%2BfwSXSlCk9zPtHHoBkFT7G4cZQOdDTLRrSG%2FmRJ3K09KEHYg%2Fc6R5Dd92inD1tLE2tiFMyHFs5uQHRMsepY4wrWiIQ4u53tPvk%2Fwiq1%2B9yL6x3QGblhdWwjX0BTVBOxexyKTlwczJW0XlWX8wpcTFfzQnJZuujbhanf2g9MGzSmkv3ylyuQdHMJDYp4Bzq%2FmnkNUg0Vp6iEvh9tyVdRNkwXeg3D8qn%2BFSOxcF%2B9vJw%3D%3D",
		},
		{
			desc:         "using VirtualHostedStyle",
			emulatorHost: "localhost:8000",
			now:          expires.Add(-24 * time.Hour),
			opts: &SignedURLOptions{
				GoogleAccessID: "xxx@clientid",
				PrivateKey:     dummyKey("rsa"),
				Method:         "POST",
				Expires:        expires,
				Scheme:         SigningSchemeV4,
				Style:          VirtualHostedStyle(),
				Insecure:       true,
			},
			want: "http://" + bucketName + ".localhost:8000/" + objectName +
				"?X-Goog-Algorithm=GOOG4-RSA-SHA256" +
				"&X-Goog-Credential=xxx%40clientid%2F20021001%2Fauto%2Fstorage%2Fgoog4_request" +
				"&X-Goog-Date=20021001T100000Z&X-Goog-Expires=86400" +
				"&X-Goog-Signature=35e0b9d33901a2518956821175f88c2c4eb3f4461b725af74b37c36d23f8bbe927558ac57b0be40d345f20bca55ba0652d38b7a620f8da68d4f733706ad104da468c3a039459acf35f3022e388760cd49893c998c33fe3ccc8c022d7034ab98bdbdcac4b680bb24ae5ed586a42ee9495a873ffc484e297853a8a3892d0d6385c980cb7e3c5c8bdd4939b4c17105f10fe8b5b9744017bf59431ff176c1550ae1c64ddd6628096eb6895c97c5da4d850aca72c14b7f5018c15b34d4b00ec63ff2ccb688ddbef2d32648e247ffd0137498080f320f293eb811a94fb526227324bbbd01335446388797803e67d802f97b52565deba3d2387ecabf4f3094662236017" +
				"&X-Goog-SignedHeaders=host",
		},
		{
			desc:         "using BucketBoundHostname",
			emulatorHost: "localhost:8000",
			now:          expires.Add(-24 * time.Hour),
			opts: &SignedURLOptions{
				GoogleAccessID: "xxx@clientid",
				PrivateKey:     dummyKey("rsa"),
				Method:         "POST",
				Expires:        expires,
				Scheme:         SigningSchemeV4,
				Style:          BucketBoundHostname("myhost"),
			},
			want: "https://" + "myhost/" + objectName +
				"?X-Goog-Algorithm=GOOG4-RSA-SHA256" +
				"&X-Goog-Credential=xxx%40clientid%2F20021001%2Fauto%2Fstorage%2Fgoog4_request" +
				"&X-Goog-Date=20021001T100000Z&X-Goog-Expires=86400" +
				"&X-Goog-Signature=15fe19f6c61bcbdbd6473c32f2bec29caa8a5fa3b2ce32cfb5329a71edaa0d4e5ffe6469f32ed4c23ca2fbed3882fdf1ed107c6a98c2c4995dda6036c64bae51e6cb542c353618f483832aa1f3ef85342ddadd69c13ad4c69fd3f573ea5cf325a58056e3d5a37005217662af63b49fef8688de3c5c7a2f7b43651a030edd0813eb7f7713989a4c29a8add65133ce652895fea9de7dbc6248ee11b4d7c6c1e152df87700100e896e544ba8eeea96584078f56e699665140b750e90550b9b79633f4e7c8409efa807be5670d6e987eeee04a4180be9b9e30bb8557597beaf390a3805cc602c87a3e34800f8bc01449c3dd10ac2f2263e55e55b91e445052548d5e" +
				"&X-Goog-SignedHeaders=host",
		},
		{
			desc:         "emulator host specifies scheme",
			emulatorHost: "https://localhost:6000",
			now:          expires.Add(-24 * time.Hour),
			opts: &SignedURLOptions{
				GoogleAccessID: "xxx@clientid",
				PrivateKey:     dummyKey("rsa"),
				Method:         "POST",
				Expires:        expires,
				Scheme:         SigningSchemeV4,
				Insecure:       true,
			},
			want: "http://localhost:6000/" + bucketName + "/" + objectName +
				"?X-Goog-Algorithm=GOOG4-RSA-SHA256" +
				"&X-Goog-Credential=xxx%40clientid%2F20021001%2Fauto%2Fstorage%2Fgoog4_request" +
				"&X-Goog-Date=20021001T100000Z&X-Goog-Expires=86400" +
				"&X-Goog-Signature=249c53142e57adf594b4f523a8a1f9c15f29b071e9abc0cf6665dbc5f692fc96fac4ab98bbea4c2397384367bc970a2e1771f2c86624475f3273970ecde8ff6df39d647e5c3f3263bf67a743e211c1958a96775edf53ece1f69ed337f0ab7fdc081c6c2b84e57b0922280d27f1da1bff47e77e3822fb1756e4c5cece9d220e6d0824ab9528e97e54f0cb09b352193b0e895344d894de11b3f5f9a2ec7d8fd6d0a4c487afd1896385a3ab9e8c3fcb3862ec0cad6ec10af1b574078eb7c79b558bcd85449a67079a0ee6da97fcbad074f1bf9fdfbdca12945336a8bd0a3b70b4c7708918cb83d10c7c4ff1f8b73275e9d1ba5d3db91069dffdf81eb7badf4e3c80" +
				"&X-Goog-SignedHeaders=host",
		},
		{
			desc:         "emulator host specifies scheme using SigningSchemeV2",
			emulatorHost: "https://localhost:8000",
			now:          expires.Add(-24 * time.Hour),
			opts: &SignedURLOptions{
				GoogleAccessID: "xxx@clientid",
				PrivateKey:     dummyKey("rsa"),
				Method:         "POST",
				Expires:        expires,
				Scheme:         SigningSchemeV2,
			},
			want: "https://localhost:8000/" + bucketName + "/" + objectName +
				"?Expires=1033570800" +
				"&GoogleAccessId=xxx%40clientid" +
				"&Signature=oRi3y2tBTmoDto7FezNx4AjC0RXA6fpJjTBa0hINeVroZ%2ByOeRU8MRwJbKg1IkBbV0IjtlPaGwv5YoUH16UYdipBjCXOS%2B1qgRWyzl8AnzvU%2BfwSXSlCk9zPtHHoBkFT7G4cZQOdDTLRrSG%2FmRJ3K09KEHYg%2Fc6R5Dd92inD1tLE2tiFMyHFs5uQHRMsepY4wrWiIQ4u53tPvk%2Fwiq1%2B9yL6x3QGblhdWwjX0BTVBOxexyKTlwczJW0XlWX8wpcTFfzQnJZuujbhanf2g9MGzSmkv3ylyuQdHMJDYp4Bzq%2FmnkNUg0Vp6iEvh9tyVdRNkwXeg3D8qn%2BFSOxcF%2B9vJw%3D%3D",
		},
	}
	oldUTCNow := utcNow
	defer func() {
		utcNow = oldUTCNow
	}()

	for _, test := range tests {
		t.Run(test.desc, func(s *testing.T) {
			utcNow = func() time.Time {
				return test.now
			}

			os.Setenv("STORAGE_EMULATOR_HOST", test.emulatorHost)

			got, err := SignedURL(bucketName, objectName, test.opts)
			if err != nil {
				s.Fatal(err)
			}

			if got != test.want {
				s.Fatalf("\n\tgot:\t%v\n\twant:\t%v", got, test.want)
			}
		})
	}
}

func TestSignedURL_MissingOptions(t *testing.T) {
	now, _ := time.Parse(time.RFC3339, "2002-10-01T00:00:00-05:00")
	expires, _ := time.Parse(time.RFC3339, "2002-10-15T00:00:00-05:00")
	pk := dummyKey("rsa")

	var tests = []struct {
		opts   *SignedURLOptions
		errMsg string
	}{
		{
			&SignedURLOptions{},
			"missing required GoogleAccessID",
		},
		{
			&SignedURLOptions{GoogleAccessID: "access_id"},
			"exactly one of PrivateKey or SignedBytes must be set",
		},
		{
			&SignedURLOptions{
				GoogleAccessID: "access_id",
				SignBytes:      func(b []byte) ([]byte, error) { return b, nil },
				PrivateKey:     pk,
			},
			"exactly one of PrivateKey or SignedBytes must be set",
		},
		{
			&SignedURLOptions{
				GoogleAccessID: "access_id",
				PrivateKey:     pk,
			},
			errMethodNotValid.Error(),
		},
		{
			&SignedURLOptions{
				GoogleAccessID: "access_id",
				PrivateKey:     pk,
				Method:         "getMethod", // wrong method name
			},
			errMethodNotValid.Error(),
		},
		{
			&SignedURLOptions{
				GoogleAccessID: "access_id",
				PrivateKey:     pk,
				Method:         "get", // name will be uppercased
			},
			"missing required expires",
		},
		{
			&SignedURLOptions{
				GoogleAccessID: "access_id",
				SignBytes:      func(b []byte) ([]byte, error) { return b, nil },
			},
			errMethodNotValid.Error(),
		},
		{
			&SignedURLOptions{
				GoogleAccessID: "access_id",
				PrivateKey:     pk,
				Method:         "PUT",
			},
			"missing required expires",
		},
		{
			&SignedURLOptions{
				GoogleAccessID: "access_id",
				PrivateKey:     pk,
				Method:         "PUT",
				Expires:        expires,
				MD5:            "invalid",
			},
			"invalid MD5 checksum",
		},
		// SigningSchemeV4 tests
		{
			&SignedURLOptions{
				PrivateKey: pk,
				Method:     "GET",
				Expires:    expires,
				Scheme:     SigningSchemeV4,
			},
			"missing required GoogleAccessID",
		},
		{
			&SignedURLOptions{
				GoogleAccessID: "access_id",
				Method:         "GET",
				Expires:        expires,
				SignBytes:      func(b []byte) ([]byte, error) { return b, nil },
				PrivateKey:     pk,
				Scheme:         SigningSchemeV4,
			},
			"exactly one of PrivateKey or SignedBytes must be set",
		},
		{
			&SignedURLOptions{
				GoogleAccessID: "access_id",
				PrivateKey:     pk,
				Expires:        expires,
				Scheme:         SigningSchemeV4,
			},
			errMethodNotValid.Error(),
		},
		{
			&SignedURLOptions{
				GoogleAccessID: "access_id",
				PrivateKey:     pk,
				Method:         "PUT",
				Scheme:         SigningSchemeV4,
			},
			"missing required expires",
		},
		{
			&SignedURLOptions{
				GoogleAccessID: "access_id",
				PrivateKey:     pk,
				Method:         "PUT",
				Expires:        now.Add(time.Hour),
				MD5:            "invalid",
				Scheme:         SigningSchemeV4,
			},
			"invalid MD5 checksum",
		},
		{
			&SignedURLOptions{
				GoogleAccessID: "access_id",
				PrivateKey:     pk,
				Method:         "GET",
				Expires:        expires,
				Scheme:         SigningSchemeV4,
			},
			"expires must be within seven days from now",
		},
		{
			&SignedURLOptions{
				GoogleAccessID: "access_id",
				PrivateKey:     pk,
				Method:         "GET",
				Expires:        now.Add(time.Hour),
				Scheme:         SigningSchemeV2,
				Style:          VirtualHostedStyle(),
			},
			"are permitted with SigningSchemeV2",
		},
	}
	oldUTCNow := utcNow
	defer func() {
		utcNow = oldUTCNow
	}()
	utcNow = func() time.Time {
		return now
	}

	for _, test := range tests {
		_, err := SignedURL("bucket", "name", test.opts)
		if !strings.Contains(err.Error(), test.errMsg) {
			t.Errorf("expected err: %v, found: %v", test.errMsg, err)
		}
	}
}

func TestPathEncodeV4(t *testing.T) {
	tests := []struct {
		input string
		want  string
	}{
		{
			"path/with/slashes",
			"path/with/slashes",
		},
		{
			"path/with/speci@lchar$&",
			"path/with/speci%40lchar%24%26",
		},
		{
			"path/with/un_ersc_re/~tilde/sp  ace/",
			"path/with/un_ersc_re/~tilde/sp%20%20ace/",
		},
	}
	for _, test := range tests {
		if got := pathEncodeV4(test.input); got != test.want {
			t.Errorf("pathEncodeV4(%q) =  %q, want %q", test.input, got, test.want)
		}
	}
}

func dummyKey(kind string) []byte {
	slurp, err := ioutil.ReadFile(fmt.Sprintf("./internal/test/dummy_%s", kind))
	if err != nil {
		log.Fatal(err)
	}
	return slurp
}

func TestObjectNames(t *testing.T) {
	t.Parallel()
	// Naming requirements: https://cloud.google.com/storage/docs/bucket-naming
	const maxLegalLength = 1024

	type testT struct {
		name, want string
	}
	tests := []testT{
		// Embedded characters important in URLs.
		{"foo % bar", "foo%20%25%20bar"},
		{"foo ? bar", "foo%20%3F%20bar"},
		{"foo / bar", "foo%20/%20bar"},
		{"foo %?/ bar", "foo%20%25%3F/%20bar"},

		// Non-Roman scripts
		{"타코", "%ED%83%80%EC%BD%94"},
		{"世界", "%E4%B8%96%E7%95%8C"},

		// Longest legal name
		{strings.Repeat("a", maxLegalLength), strings.Repeat("a", maxLegalLength)},

		// Line terminators besides CR and LF: https://en.wikipedia.org/wiki/Newline#Unicode
		{"foo \u000b bar", "foo%20%0B%20bar"},
		{"foo \u000c bar", "foo%20%0C%20bar"},
		{"foo \u0085 bar", "foo%20%C2%85%20bar"},
		{"foo \u2028 bar", "foo%20%E2%80%A8%20bar"},
		{"foo \u2029 bar", "foo%20%E2%80%A9%20bar"},

		// Null byte.
		{"foo \u0000 bar", "foo%20%00%20bar"},

		// Non-control characters that are discouraged, but not forbidden, according to the documentation.
		{"foo # bar", "foo%20%23%20bar"},
		{"foo []*? bar", "foo%20%5B%5D%2A%3F%20bar"},

		// Angstrom symbol singleton and normalized forms: http://unicode.org/reports/tr15/
		{"foo \u212b bar", "foo%20%E2%84%AB%20bar"},
		{"foo \u0041\u030a bar", "foo%20A%CC%8A%20bar"},
		{"foo \u00c5 bar", "foo%20%C3%85%20bar"},

		// Hangul separating jamo: http://www.unicode.org/versions/Unicode7.0.0/ch18.pdf (Table 18-10)
		{"foo \u3131\u314f bar", "foo%20%E3%84%B1%E3%85%8F%20bar"},
		{"foo \u1100\u1161 bar", "foo%20%E1%84%80%E1%85%A1%20bar"},
		{"foo \uac00 bar", "foo%20%EA%B0%80%20bar"},
	}

	// C0 control characters not forbidden by the docs.
	var runes []rune
	for r := rune(0x01); r <= rune(0x1f); r++ {
		if r != '\u000a' && r != '\u000d' {
			runes = append(runes, r)
		}
	}
	tests = append(tests, testT{fmt.Sprintf("foo %s bar", string(runes)), "foo%20%01%02%03%04%05%06%07%08%09%0B%0C%0E%0F%10%11%12%13%14%15%16%17%18%19%1A%1B%1C%1D%1E%1F%20bar"})

	// C1 control characters, plus DEL.
	runes = nil
	for r := rune(0x7f); r <= rune(0x9f); r++ {
		runes = append(runes, r)
	}
	tests = append(tests, testT{fmt.Sprintf("foo %s bar", string(runes)), "foo%20%7F%C2%80%C2%81%C2%82%C2%83%C2%84%C2%85%C2%86%C2%87%C2%88%C2%89%C2%8A%C2%8B%C2%8C%C2%8D%C2%8E%C2%8F%C2%90%C2%91%C2%92%C2%93%C2%94%C2%95%C2%96%C2%97%C2%98%C2%99%C2%9A%C2%9B%C2%9C%C2%9D%C2%9E%C2%9F%20bar"})

	opts := &SignedURLOptions{
		GoogleAccessID: "xxx@clientid",
		PrivateKey:     dummyKey("rsa"),
		Method:         "GET",
		MD5:            "ICy5YqxZB1uWSwcVLSNLcA==",
		Expires:        time.Date(2002, time.October, 2, 10, 0, 0, 0, time.UTC),
		ContentType:    "application/json",
		Headers:        []string{"x-goog-header1", "x-goog-header2"},
	}

	for _, test := range tests {
		g, err := SignedURL("bucket-name", test.name, opts)
		if err != nil {
			t.Errorf("SignedURL(%q) err=%v, want nil", test.name, err)
		}
		if w := "/bucket-name/" + test.want; !strings.Contains(g, w) {
			t.Errorf("SignedURL(%q)=%q, want substring %q", test.name, g, w)
		}
	}
}

func TestCondition(t *testing.T) {
	t.Parallel()
	gotReq := make(chan *http.Request, 1)
	hc, close := newTestServer(func(w http.ResponseWriter, r *http.Request) {
		io.Copy(ioutil.Discard, r.Body)
		gotReq <- r
		w.WriteHeader(200)
	})
	defer close()
	ctx := context.Background()
	c, err := NewClient(ctx, option.WithHTTPClient(hc))
	if err != nil {
		t.Fatal(err)
	}

	obj := c.Bucket("buck").Object("obj")
	dst := c.Bucket("dstbuck").Object("dst")
	tests := []struct {
		fn   func() error
		want string
	}{
		{
			func() error {
				_, err := obj.Generation(1234).NewReader(ctx)
				return err
			},
			"GET /buck/obj?generation=1234",
		},
		{
			func() error {
				_, err := obj.If(Conditions{MetagenerationNotMatch: 1234}).Attrs(ctx)
				return err
			},
			"GET /storage/v1/b/buck/o/obj?alt=json&ifMetagenerationNotMatch=1234&prettyPrint=false&projection=full",
		},
		{
			func() error {
				_, err := obj.If(Conditions{MetagenerationMatch: 1234}).Update(ctx, ObjectAttrsToUpdate{})
				return err
			},
			"PATCH /storage/v1/b/buck/o/obj?alt=json&ifMetagenerationMatch=1234&prettyPrint=false&projection=full",
		},
		{
			func() error { return obj.Generation(1234).Delete(ctx) },
			"DELETE /storage/v1/b/buck/o/obj?alt=json&generation=1234&prettyPrint=false",
		},
		{
			func() error {
				w := obj.If(Conditions{GenerationMatch: 1234}).NewWriter(ctx)
				w.ContentType = "text/plain"
				return w.Close()
			},
			"POST /upload/storage/v1/b/buck/o?alt=json&ifGenerationMatch=1234&name=obj&prettyPrint=false&projection=full&uploadType=multipart",
		},
		{
			func() error {
				w := obj.If(Conditions{DoesNotExist: true}).NewWriter(ctx)
				w.ContentType = "text/plain"
				return w.Close()
			},
			"POST /upload/storage/v1/b/buck/o?alt=json&ifGenerationMatch=0&name=obj&prettyPrint=false&projection=full&uploadType=multipart",
		},
		{
			func() error {
				_, err := dst.If(Conditions{MetagenerationMatch: 5678}).CopierFrom(obj.If(Conditions{GenerationMatch: 1234})).Run(ctx)
				return err
			},
			"POST /storage/v1/b/buck/o/obj/rewriteTo/b/dstbuck/o/dst?alt=json&ifMetagenerationMatch=5678&ifSourceGenerationMatch=1234&prettyPrint=false&projection=full",
		},
	}

	for i, tt := range tests {
		if err := tt.fn(); err != nil && err != io.EOF {
			t.Error(err)
			continue
		}
		select {
		case r := <-gotReq:
			got := r.Method + " " + r.RequestURI
			if got != tt.want {
				t.Errorf("%d. RequestURI = %q; want %q", i, got, tt.want)
			}
		case <-time.After(5 * time.Second):
			t.Fatalf("%d. timeout", i)
		}
		if err != nil {
			t.Fatal(err)
		}
	}

	readerTests := []struct {
		fn   func() error
		want string
	}{
		{
			func() error {
				_, err := obj.If(Conditions{GenerationMatch: 1234}).NewReader(ctx)
				return err
			},
			"x-goog-if-generation-match: 1234, x-goog-if-metageneration-match: ",
		},
		{
			func() error {
				_, err := obj.If(Conditions{MetagenerationMatch: 5}).NewReader(ctx)
				return err
			},
			"x-goog-if-generation-match: , x-goog-if-metageneration-match: 5",
		},
		{
			func() error {
				_, err := obj.If(Conditions{GenerationMatch: 1234, MetagenerationMatch: 5}).NewReader(ctx)
				return err
			},
			"x-goog-if-generation-match: 1234, x-goog-if-metageneration-match: 5",
		},
	}

	for i, tt := range readerTests {
		if err := tt.fn(); err != nil && err != io.EOF {
			t.Error(err)
			continue
		}

		select {
		case r := <-gotReq:
			generationConds := r.Header.Get("x-goog-if-generation-match")
			metagenerationConds := r.Header.Get("x-goog-if-metageneration-match")
			got := fmt.Sprintf(
				"x-goog-if-generation-match: %s, x-goog-if-metageneration-match: %s",
				generationConds,
				metagenerationConds,
			)
			if got != tt.want {
				t.Errorf("%d. RequestHeaders = %q; want %q", i, got, tt.want)
			}
		case <-time.After(5 * time.Second):
			t.Fatalf("%d. timeout", i)
		}
		if err != nil {
			t.Fatal(err)
		}
	}

	// Test an error, too:
	err = obj.Generation(1234).NewWriter(ctx).Close()
	if err == nil || !strings.Contains(err.Error(), "storage: generation not supported") {
		t.Errorf("want error about unsupported generation; got %v", err)
	}
}

func TestConditionErrors(t *testing.T) {
	t.Parallel()
	for _, conds := range []Conditions{
		{GenerationMatch: 0},
		{DoesNotExist: false}, // same as above, actually
		{GenerationMatch: 1, GenerationNotMatch: 2},
		{GenerationNotMatch: 2, DoesNotExist: true},
		{MetagenerationMatch: 1, MetagenerationNotMatch: 2},
	} {
		if err := conds.validate(""); err == nil {
			t.Errorf("%+v: got nil, want error", conds)
		}
	}
}

func expectedAttempts(value int) *int {
	return &value
}

// Test that ObjectHandle.Retryer correctly configures the retry configuration
// in the ObjectHandle.
func TestObjectRetryer(t *testing.T) {
	testCases := []struct {
		name string
		call func(o *ObjectHandle) *ObjectHandle
		want *retryConfig
	}{
		{
			name: "all defaults",
			call: func(o *ObjectHandle) *ObjectHandle {
				return o.Retryer()
			},
			want: &retryConfig{},
		},
		{
			name: "set all options",
			call: func(o *ObjectHandle) *ObjectHandle {
				return o.Retryer(
					WithBackoff(gax.Backoff{
						Initial:    2 * time.Second,
						Max:        30 * time.Second,
						Multiplier: 3,
					}),
					WithMaxAttempts(5),
					WithPolicy(RetryAlways),
					WithErrorFunc(func(err error) bool { return false }))
			},
			want: &retryConfig{
				backoff: &gax.Backoff{
					Initial:    2 * time.Second,
					Max:        30 * time.Second,
					Multiplier: 3,
				},
				maxAttempts: expectedAttempts(5),
				policy:      RetryAlways,
				shouldRetry: func(err error) bool { return false },
			},
		},
		{
			name: "set some backoff options",
			call: func(o *ObjectHandle) *ObjectHandle {
				return o.Retryer(
					WithBackoff(gax.Backoff{
						Multiplier: 3,
					}))
			},
			want: &retryConfig{
				backoff: &gax.Backoff{
					Multiplier: 3,
				}},
		},
		{
			name: "set policy only",
			call: func(o *ObjectHandle) *ObjectHandle {
				return o.Retryer(WithPolicy(RetryNever))
			},
			want: &retryConfig{
				policy: RetryNever,
			},
		},
		{
			name: "set max retry attempts only",
			call: func(o *ObjectHandle) *ObjectHandle {
				return o.Retryer(WithMaxAttempts(11))
			},
			want: &retryConfig{
				maxAttempts: expectedAttempts(11),
			},
		},
		{
			name: "set ErrorFunc only",
			call: func(o *ObjectHandle) *ObjectHandle {
				return o.Retryer(
					WithErrorFunc(func(err error) bool { return false }))
			},
			want: &retryConfig{
				shouldRetry: func(err error) bool { return false },
			},
		},
	}
	for _, tc := range testCases {
		t.Run(tc.name, func(s *testing.T) {
			o := tc.call(&ObjectHandle{})
			if diff := cmp.Diff(
				o.retry,
				tc.want,
				cmp.AllowUnexported(retryConfig{}, gax.Backoff{}),
				// ErrorFunc cannot be compared directly, but we check if both are
				// either nil or non-nil.
				cmp.Comparer(func(a, b func(err error) bool) bool {
					return (a == nil && b == nil) || (a != nil && b != nil)
				}),
			); diff != "" {
				s.Fatalf("retry not configured correctly: %v", diff)
			}
		})
	}
}

// Test that Client.SetRetry correctly configures the retry configuration
// on the Client.
func TestClientSetRetry(t *testing.T) {
	testCases := []struct {
		name          string
		clientOptions []RetryOption
		want          *retryConfig
	}{
		{
			name:          "all defaults",
			clientOptions: []RetryOption{},
			want:          &retryConfig{},
		},
		{
			name: "set all options",
			clientOptions: []RetryOption{
				WithBackoff(gax.Backoff{
					Initial:    2 * time.Second,
					Max:        30 * time.Second,
					Multiplier: 3,
				}),
				WithMaxAttempts(5),
				WithPolicy(RetryAlways),
				WithErrorFunc(func(err error) bool { return false }),
			},
			want: &retryConfig{
				backoff: &gax.Backoff{
					Initial:    2 * time.Second,
					Max:        30 * time.Second,
					Multiplier: 3,
				},
				maxAttempts: expectedAttempts(5),
				policy:      RetryAlways,
				shouldRetry: func(err error) bool { return false },
			},
		},
		{
			name: "set some backoff options",
			clientOptions: []RetryOption{
				WithBackoff(gax.Backoff{
					Multiplier: 3,
				}),
			},
			want: &retryConfig{
				backoff: &gax.Backoff{
					Multiplier: 3,
				}},
		},
		{
			name: "set policy only",
			clientOptions: []RetryOption{
				WithPolicy(RetryNever),
			},
			want: &retryConfig{
				policy: RetryNever,
			},
		},
		{
			name: "set max retry attempts only",
			clientOptions: []RetryOption{
				WithMaxAttempts(7),
			},
			want: &retryConfig{
				maxAttempts: expectedAttempts(7),
			},
		},
		{
			name: "set ErrorFunc only",
			clientOptions: []RetryOption{
				WithErrorFunc(func(err error) bool { return false }),
			},
			want: &retryConfig{
				shouldRetry: func(err error) bool { return false },
			},
		},
	}
	for _, tc := range testCases {
		t.Run(tc.name, func(s *testing.T) {
			c, err := NewClient(context.Background(), option.WithoutAuthentication())
			if err != nil {
				t.Fatalf("NewClient: %v", err)
			}
			defer c.Close()
			c.SetRetry(tc.clientOptions...)

			if diff := cmp.Diff(
				c.retry,
				tc.want,
				cmp.AllowUnexported(retryConfig{}, gax.Backoff{}),
				// ErrorFunc cannot be compared directly, but we check if both are
				// either nil or non-nil.
				cmp.Comparer(func(a, b func(err error) bool) bool {
					return (a == nil && b == nil) || (a != nil && b != nil)
				}),
			); diff != "" {
				s.Fatalf("retry not configured correctly: %v", diff)
			}
		})
	}
}

// Test the interactions between Client, ObjectHandle and BucketHandle Retryers,
// and that they correctly configure the retry configuration for objects, ACLs, and HmacKeys
func TestRetryer(t *testing.T) {
	testCases := []struct {
		name          string
		clientOptions []RetryOption
		bucketOptions []RetryOption
		objectOptions []RetryOption
		want          *retryConfig
	}{
		{
			name: "no retries",
			want: nil,
		},
		{
			name: "object retryer configures retry",
			objectOptions: []RetryOption{
				WithPolicy(RetryAlways),
				WithMaxAttempts(5),
				WithErrorFunc(ShouldRetry),
			},
			want: &retryConfig{
				shouldRetry: ShouldRetry,
				maxAttempts: expectedAttempts(5),
				policy:      RetryAlways,
			},
		},
		{
			name: "bucket retryer configures retry",
			bucketOptions: []RetryOption{
				WithBackoff(gax.Backoff{
					Initial:    time.Minute,
					Max:        time.Hour,
					Multiplier: 6,
				}),
				WithPolicy(RetryAlways),
				WithMaxAttempts(11),
				WithErrorFunc(ShouldRetry),
			},
			want: &retryConfig{
				backoff: &gax.Backoff{
					Initial:    time.Minute,
					Max:        time.Hour,
					Multiplier: 6,
				},
				shouldRetry: ShouldRetry,
				maxAttempts: expectedAttempts(11),
				policy:      RetryAlways,
			},
		},
		{
			name: "client retryer configures retry",
			clientOptions: []RetryOption{
				WithBackoff(gax.Backoff{
					Initial:    time.Minute,
					Max:        time.Hour,
					Multiplier: 6,
				}),
				WithPolicy(RetryAlways),
				WithMaxAttempts(7),
				WithErrorFunc(ShouldRetry),
			},
			want: &retryConfig{
				backoff: &gax.Backoff{
					Initial:    time.Minute,
					Max:        time.Hour,
					Multiplier: 6,
				},
				shouldRetry: ShouldRetry,
				maxAttempts: expectedAttempts(7),
				policy:      RetryAlways,
			},
		},
		{
			name: "object retryer overrides bucket retryer",
			bucketOptions: []RetryOption{
				WithPolicy(RetryAlways),
			},
			objectOptions: []RetryOption{
				WithPolicy(RetryNever),
				WithMaxAttempts(5),
				WithErrorFunc(ShouldRetry),
			},
			want: &retryConfig{
				policy:      RetryNever,
				maxAttempts: expectedAttempts(5),
				shouldRetry: ShouldRetry,
			},
		},
		{
			name: "object retryer overrides client retryer",
			clientOptions: []RetryOption{
				WithPolicy(RetryAlways),
			},
			objectOptions: []RetryOption{
				WithPolicy(RetryNever),
				WithMaxAttempts(11),
				WithErrorFunc(ShouldRetry),
			},
			want: &retryConfig{
				policy:      RetryNever,
				maxAttempts: expectedAttempts(11),
				shouldRetry: ShouldRetry,
			},
		},
		{
			name: "bucket retryer overrides client retryer",
			clientOptions: []RetryOption{
				WithBackoff(gax.Backoff{
					Initial:    time.Minute,
					Max:        time.Hour,
					Multiplier: 6,
				}),
				WithPolicy(RetryAlways),
			},
			bucketOptions: []RetryOption{
				WithBackoff(gax.Backoff{
					Initial: time.Nanosecond,
					Max:     time.Microsecond,
				}),
				WithErrorFunc(ShouldRetry),
				WithMaxAttempts(5),
			},
			want: &retryConfig{
				policy:      RetryAlways,
				maxAttempts: expectedAttempts(5),
				shouldRetry: ShouldRetry,
				backoff: &gax.Backoff{
					Initial: time.Nanosecond,
					Max:     time.Microsecond,
				},
			},
		},
		{
			name: "object retryer overrides bucket retryer backoff options",
			bucketOptions: []RetryOption{
				WithBackoff(gax.Backoff{
					Initial:    time.Minute,
					Max:        time.Hour,
					Multiplier: 6,
				}),
			},
			objectOptions: []RetryOption{
				WithBackoff(gax.Backoff{
					Initial: time.Nanosecond,
					Max:     time.Microsecond,
				}),
			},
			want: &retryConfig{
				backoff: &gax.Backoff{
					Initial: time.Nanosecond,
					Max:     time.Microsecond,
				},
			},
		},
		{
			name: "object retryer does not override bucket retryer if option is not set",
			bucketOptions: []RetryOption{
				WithPolicy(RetryNever),
				WithErrorFunc(ShouldRetry),
				WithMaxAttempts(5),
			},
			objectOptions: []RetryOption{
				WithBackoff(gax.Backoff{
					Initial: time.Nanosecond,
					Max:     time.Second,
				}),
			},
			want: &retryConfig{
				policy:      RetryNever,
				maxAttempts: expectedAttempts(5),
				shouldRetry: ShouldRetry,
				backoff: &gax.Backoff{
					Initial: time.Nanosecond,
					Max:     time.Second,
				},
			},
		},
		{
			name: "object's backoff completely overwrites bucket's backoff",
			bucketOptions: []RetryOption{
				WithBackoff(gax.Backoff{
					Initial: time.Hour,
				}),
			},
			objectOptions: []RetryOption{
				WithBackoff(gax.Backoff{
					Multiplier: 4,
				}),
			},
			want: &retryConfig{
				backoff: &gax.Backoff{
					Multiplier: 4,
				},
			},
		},
	}
	for _, tc := range testCases {
		t.Run(tc.name, func(s *testing.T) {
			ctx := context.Background()
			c, err := NewClient(ctx, option.WithoutAuthentication())
			if err != nil {
				t.Fatalf("NewClient: %v", err)
			}
			defer c.Close()
			if len(tc.clientOptions) > 0 {
				c.SetRetry(tc.clientOptions...)
			}
			b := c.Bucket("buck")
			if len(tc.bucketOptions) > 0 {
				b = b.Retryer(tc.bucketOptions...)
			}
			o := b.Object("obj")
			if len(tc.objectOptions) > 0 {
				o = o.Retryer(tc.objectOptions...)
			}

			configHandleCases := []struct {
				r    *retryConfig
				name string
				want *retryConfig
			}{
				{
					name: "object.retry",
					r:    o.retry,
					want: tc.want,
				},
				{
					name: "object.ACL()",
					r:    o.ACL().retry,
					want: tc.want,
				},
				{
					name: "bucket.ACL()",
					r:    b.ACL().retry,
					want: b.retry,
				},
				{
					name: "bucket.DefaultObjectACL()",
					r:    b.DefaultObjectACL().retry,
					want: b.retry,
				},
				{
					name: "client.HMACKeyHandle()",
					r:    c.HMACKeyHandle("pID", "accessID").retry,
					want: c.retry,
				},
			}
			for _, ac := range configHandleCases {
				s.Run(ac.name, func(ss *testing.T) {
					if diff := cmp.Diff(
						ac.want,
						ac.r,
						cmp.AllowUnexported(retryConfig{}, gax.Backoff{}),
						// ErrorFunc cannot be compared directly, but we check if both are
						// either nil or non-nil.
						cmp.Comparer(func(a, b func(err error) bool) bool {
							return (a == nil && b == nil) || (a != nil && b != nil)
						}),
					); diff != "" {
						ss.Fatalf("retry not configured correctly: %v", diff)
					}
				})
			}
		})
	}
}

// Test object compose.
func TestObjectCompose(t *testing.T) {
	t.Parallel()
	gotURL := make(chan string, 1)
	gotBody := make(chan []byte, 1)
	hc, close := newTestServer(func(w http.ResponseWriter, r *http.Request) {
		body, _ := ioutil.ReadAll(r.Body)
		gotURL <- r.URL.String()
		gotBody <- body
		w.Write([]byte("{}"))
	})
	defer close()
	ctx := context.Background()
	c, err := NewClient(ctx, option.WithHTTPClient(hc))
	if err != nil {
		t.Fatal(err)
	}

	testCases := []struct {
		desc       string
		dst        *ObjectHandle
		srcs       []*ObjectHandle
		attrs      *ObjectAttrs
		sendCRC32C bool
		wantReq    raw.ComposeRequest
		wantURL    string
		wantErr    bool
	}{
		{
			desc: "basic case",
			dst:  c.Bucket("foo").Object("bar"),
			srcs: []*ObjectHandle{
				c.Bucket("foo").Object("baz"),
				c.Bucket("foo").Object("quux"),
			},
			wantURL: "/storage/v1/b/foo/o/bar/compose?alt=json&prettyPrint=false",
			wantReq: raw.ComposeRequest{
				Destination: &raw.Object{Bucket: "foo"},
				SourceObjects: []*raw.ComposeRequestSourceObjects{
					{Name: "baz"},
					{Name: "quux"},
				},
			},
		},
		{
			desc: "with object attrs",
			dst:  c.Bucket("foo").Object("bar"),
			srcs: []*ObjectHandle{
				c.Bucket("foo").Object("baz"),
				c.Bucket("foo").Object("quux"),
			},
			attrs: &ObjectAttrs{
				Name:        "not-bar",
				ContentType: "application/json",
			},
			wantURL: "/storage/v1/b/foo/o/bar/compose?alt=json&prettyPrint=false",
			wantReq: raw.ComposeRequest{
				Destination: &raw.Object{
					Bucket:      "foo",
					Name:        "not-bar",
					ContentType: "application/json",
				},
				SourceObjects: []*raw.ComposeRequestSourceObjects{
					{Name: "baz"},
					{Name: "quux"},
				},
			},
		},
		{
			desc: "with conditions",
			dst: c.Bucket("foo").Object("bar").If(Conditions{
				GenerationMatch:     12,
				MetagenerationMatch: 34,
			}),
			srcs: []*ObjectHandle{
				c.Bucket("foo").Object("baz").Generation(56),
				c.Bucket("foo").Object("quux").If(Conditions{GenerationMatch: 78}),
			},
			wantURL: "/storage/v1/b/foo/o/bar/compose?alt=json&ifGenerationMatch=12&ifMetagenerationMatch=34&prettyPrint=false",
			wantReq: raw.ComposeRequest{
				Destination: &raw.Object{Bucket: "foo"},
				SourceObjects: []*raw.ComposeRequestSourceObjects{
					{
						Name:       "baz",
						Generation: 56,
					},
					{
						Name: "quux",
						ObjectPreconditions: &raw.ComposeRequestSourceObjectsObjectPreconditions{
							IfGenerationMatch: 78,
						},
					},
				},
			},
		},
		{
			desc: "with crc32c",
			dst:  c.Bucket("foo").Object("bar"),
			srcs: []*ObjectHandle{
				c.Bucket("foo").Object("baz"),
				c.Bucket("foo").Object("quux"),
			},
			attrs: &ObjectAttrs{
				CRC32C: 42,
			},
			sendCRC32C: true,
			wantURL:    "/storage/v1/b/foo/o/bar/compose?alt=json&prettyPrint=false",
			wantReq: raw.ComposeRequest{
				Destination: &raw.Object{Bucket: "foo", Crc32c: "AAAAKg=="},
				SourceObjects: []*raw.ComposeRequestSourceObjects{
					{Name: "baz"},
					{Name: "quux"},
				},
			},
		},
		{
			desc:    "no sources",
			dst:     c.Bucket("foo").Object("bar"),
			wantErr: true,
		},
		{
			desc: "destination, no bucket",
			dst:  c.Bucket("").Object("bar"),
			srcs: []*ObjectHandle{
				c.Bucket("foo").Object("baz"),
			},
			wantErr: true,
		},
		{
			desc: "destination, no object",
			dst:  c.Bucket("foo").Object(""),
			srcs: []*ObjectHandle{
				c.Bucket("foo").Object("baz"),
			},
			wantErr: true,
		},
		{
			desc: "source, different bucket",
			dst:  c.Bucket("foo").Object("bar"),
			srcs: []*ObjectHandle{
				c.Bucket("otherbucket").Object("baz"),
			},
			wantErr: true,
		},
		{
			desc: "source, no object",
			dst:  c.Bucket("foo").Object("bar"),
			srcs: []*ObjectHandle{
				c.Bucket("foo").Object(""),
			},
			wantErr: true,
		},
		{
			desc: "destination, bad condition",
			dst:  c.Bucket("foo").Object("bar").Generation(12),
			srcs: []*ObjectHandle{
				c.Bucket("foo").Object("baz"),
			},
			wantErr: true,
		},
		{
			desc: "source, bad condition",
			dst:  c.Bucket("foo").Object("bar"),
			srcs: []*ObjectHandle{
				c.Bucket("foo").Object("baz").If(Conditions{MetagenerationMatch: 12}),
			},
			wantErr: true,
		},
	}

	for _, tt := range testCases {
		composer := tt.dst.ComposerFrom(tt.srcs...)
		if tt.attrs != nil {
			composer.ObjectAttrs = *tt.attrs
		}
		composer.SendCRC32C = tt.sendCRC32C
		_, err := composer.Run(ctx)
		if gotErr := err != nil; gotErr != tt.wantErr {
			t.Errorf("%s: got error %v; want err %t", tt.desc, err, tt.wantErr)
			continue
		}
		if tt.wantErr {
			continue
		}
		u, body := <-gotURL, <-gotBody
		if u != tt.wantURL {
			t.Errorf("%s: request URL\ngot  %q\nwant %q", tt.desc, u, tt.wantURL)
		}
		var req raw.ComposeRequest
		if err := json.Unmarshal(body, &req); err != nil {
			t.Errorf("%s: json.Unmarshal %v (body %s)", tt.desc, err, body)
		}
		if !testutil.Equal(req, tt.wantReq) {
			// Print to JSON.
			wantReq, _ := json.Marshal(tt.wantReq)
			t.Errorf("%s: request body\ngot  %s\nwant %s", tt.desc, body, wantReq)
		}
	}
}

// Test that ObjectIterator's Next and NextPage methods correctly terminate
// if there is nothing to iterate over.
func TestEmptyObjectIterator(t *testing.T) {
	t.Parallel()
	hClient, close := newTestServer(func(w http.ResponseWriter, r *http.Request) {
		io.Copy(ioutil.Discard, r.Body)
		fmt.Fprintf(w, "{}")
	})
	defer close()
	ctx := context.Background()
	client, err := NewClient(ctx, option.WithHTTPClient(hClient))
	if err != nil {
		t.Fatal(err)
	}
	it := client.Bucket("b").Objects(ctx, nil)
	_, err = it.Next()
	if err != iterator.Done {
		t.Errorf("got %v, want Done", err)
	}
}

// Test that BucketIterator's Next method correctly terminates if there is
// nothing to iterate over.
func TestEmptyBucketIterator(t *testing.T) {
	t.Parallel()
	hClient, close := newTestServer(func(w http.ResponseWriter, r *http.Request) {
		io.Copy(ioutil.Discard, r.Body)
		fmt.Fprintf(w, "{}")
	})
	defer close()
	ctx := context.Background()
	client, err := NewClient(ctx, option.WithHTTPClient(hClient))
	if err != nil {
		t.Fatal(err)
	}
	it := client.Buckets(ctx, "project")
	_, err = it.Next()
	if err != iterator.Done {
		t.Errorf("got %v, want Done", err)
	}

}

func TestCodecUint32(t *testing.T) {
	t.Parallel()
	for _, u := range []uint32{0, 1, 256, 0xFFFFFFFF} {
		s := encodeUint32(u)
		d, err := decodeUint32(s)
		if err != nil {
			t.Fatal(err)
		}
		if d != u {
			t.Errorf("got %d, want input %d", d, u)
		}
	}
}

func TestUserProject(t *testing.T) {
	// Verify that the userProject query param is sent.
	t.Parallel()
	ctx := context.Background()
	gotURL := make(chan *url.URL, 1)
	hClient, close := newTestServer(func(w http.ResponseWriter, r *http.Request) {
		io.Copy(ioutil.Discard, r.Body)
		gotURL <- r.URL
		if strings.Contains(r.URL.String(), "/rewriteTo/") {
			res := &raw.RewriteResponse{Done: true}
			bytes, err := res.MarshalJSON()
			if err != nil {
				t.Fatal(err)
			}
			w.Write(bytes)
		} else {
			fmt.Fprintf(w, "{}")
		}
	})
	defer close()
	client, err := NewClient(ctx, option.WithHTTPClient(hClient))
	if err != nil {
		t.Fatal(err)
	}

	re := regexp.MustCompile(`\buserProject=p\b`)
	b := client.Bucket("b").UserProject("p")
	o := b.Object("o")

	check := func(msg string, f func()) {
		f()
		select {
		case u := <-gotURL:
			if !re.MatchString(u.RawQuery) {
				t.Errorf("%s: query string %q does not contain userProject", msg, u.RawQuery)
			}
		case <-time.After(2 * time.Second):
			t.Errorf("%s: timed out", msg)
		}
	}

	check("buckets.delete", func() { b.Delete(ctx) })
	check("buckets.get", func() { b.Attrs(ctx) })
	check("buckets.patch", func() { b.Update(ctx, BucketAttrsToUpdate{}) })
	check("storage.objects.compose", func() { o.ComposerFrom(b.Object("x")).Run(ctx) })
	check("storage.objects.delete", func() { o.Delete(ctx) })
	check("storage.objects.get", func() { o.Attrs(ctx) })
	check("storage.objects.insert", func() { o.NewWriter(ctx).Close() })
	check("storage.objects.list", func() { b.Objects(ctx, nil).Next() })
	check("storage.objects.patch", func() { o.Update(ctx, ObjectAttrsToUpdate{}) })
	check("storage.objects.rewrite", func() { o.CopierFrom(b.Object("x")).Run(ctx) })
	check("storage.objectAccessControls.list", func() { o.ACL().List(ctx) })
	check("storage.objectAccessControls.update", func() { o.ACL().Set(ctx, "", "") })
	check("storage.objectAccessControls.delete", func() { o.ACL().Delete(ctx, "") })
	check("storage.bucketAccessControls.list", func() { b.ACL().List(ctx) })
	check("storage.bucketAccessControls.update", func() { b.ACL().Set(ctx, "", "") })
	check("storage.bucketAccessControls.delete", func() { b.ACL().Delete(ctx, "") })
	check("storage.defaultObjectAccessControls.list",
		func() { b.DefaultObjectACL().List(ctx) })
	check("storage.defaultObjectAccessControls.update",
		func() { b.DefaultObjectACL().Set(ctx, "", "") })
	check("storage.defaultObjectAccessControls.delete",
		func() { b.DefaultObjectACL().Delete(ctx, "") })
	check("buckets.getIamPolicy", func() { b.IAM().Policy(ctx) })
	check("buckets.setIamPolicy", func() {
		p := &iam.Policy{}
		p.Add("m", iam.Owner)
		b.IAM().SetPolicy(ctx, p)
	})
	check("buckets.testIamPermissions", func() { b.IAM().TestPermissions(ctx, nil) })
	check("storage.notifications.insert", func() {
		b.AddNotification(ctx, &Notification{TopicProjectID: "p", TopicID: "t"})
	})
	check("storage.notifications.delete", func() { b.DeleteNotification(ctx, "n") })
	check("storage.notifications.list", func() { b.Notifications(ctx) })
}

func newTestServer(handler func(w http.ResponseWriter, r *http.Request)) (*http.Client, func()) {
	ts := httptest.NewTLSServer(http.HandlerFunc(handler))
	tlsConf := &tls.Config{InsecureSkipVerify: true}
	tr := &http.Transport{
		TLSClientConfig: tlsConf,
		DialTLS: func(netw, addr string) (net.Conn, error) {
			return tls.Dial("tcp", ts.Listener.Addr().String(), tlsConf)
		},
	}
	return &http.Client{Transport: tr}, func() {
		tr.CloseIdleConnections()
		ts.Close()
	}
}

func TestRawObjectToObjectAttrs(t *testing.T) {
	t.Parallel()
	tests := []struct {
		in   *raw.Object
		want *ObjectAttrs
	}{
		{in: nil, want: nil},
		{
			in: &raw.Object{
				Bucket:                  "Test",
				ContentLanguage:         "en-us",
				ContentType:             "video/mpeg",
				CustomTime:              "2020-08-25T19:33:36Z",
				EventBasedHold:          false,
				Etag:                    "Zkyw9ACJZUvcYmlFaKGChzhmtnE/dt1zHSfweiWpwzdGsqXwuJZqiD0",
				Generation:              7,
				Md5Hash:                 "MTQ2ODNjYmE0NDRkYmNjNmRiMjk3NjQ1ZTY4M2Y1YzE=",
				Name:                    "foo.mp4",
				RetentionExpirationTime: "2019-03-31T19:33:36Z",
				Size:                    1 << 20,
				TimeCreated:             "2019-03-31T19:32:10Z",
				TimeDeleted:             "2019-03-31T19:33:39Z",
				TemporaryHold:           true,
				ComponentCount:          2,
			},
			want: &ObjectAttrs{
				Bucket:                  "Test",
				Created:                 time.Date(2019, 3, 31, 19, 32, 10, 0, time.UTC),
				ContentLanguage:         "en-us",
				ContentType:             "video/mpeg",
				CustomTime:              time.Date(2020, 8, 25, 19, 33, 36, 0, time.UTC),
				Deleted:                 time.Date(2019, 3, 31, 19, 33, 39, 0, time.UTC),
				EventBasedHold:          false,
				Etag:                    "Zkyw9ACJZUvcYmlFaKGChzhmtnE/dt1zHSfweiWpwzdGsqXwuJZqiD0",
				Generation:              7,
				MD5:                     []byte("14683cba444dbcc6db297645e683f5c1"),
				Name:                    "foo.mp4",
				RetentionExpirationTime: time.Date(2019, 3, 31, 19, 33, 36, 0, time.UTC),
				Size:                    1 << 20,
				TemporaryHold:           true,
				ComponentCount:          2,
			},
		},
	}

	for i, tt := range tests {
		got := newObject(tt.in)
		if diff := testutil.Diff(got, tt.want); diff != "" {
			t.Errorf("#%d: newObject mismatches:\ngot=-, want=+:\n%s", i, diff)
		}
	}
}

func TestObjectAttrsToRawObject(t *testing.T) {
	t.Parallel()
	bucketName := "the-bucket"
	in := &ObjectAttrs{
		Bucket:                  "Test",
		Created:                 time.Date(2019, 3, 31, 19, 32, 10, 0, time.UTC),
		ContentLanguage:         "en-us",
		ContentType:             "video/mpeg",
		Deleted:                 time.Date(2019, 3, 31, 19, 33, 39, 0, time.UTC),
		EventBasedHold:          false,
		Etag:                    "Zkyw9ACJZUvcYmlFaKGChzhmtnE/dt1zHSfweiWpwzdGsqXwuJZqiD0",
		Generation:              7,
		MD5:                     []byte("14683cba444dbcc6db297645e683f5c1"),
		Name:                    "foo.mp4",
		RetentionExpirationTime: time.Date(2019, 3, 31, 19, 33, 36, 0, time.UTC),
		Size:                    1 << 20,
		TemporaryHold:           true,
	}
	want := &raw.Object{
		Bucket:                  bucketName,
		ContentLanguage:         "en-us",
		ContentType:             "video/mpeg",
		EventBasedHold:          false,
		Name:                    "foo.mp4",
		RetentionExpirationTime: "2019-03-31T19:33:36Z",
		TemporaryHold:           true,
	}
	got := in.toRawObject(bucketName)
	if !testutil.Equal(got, want) {
		if diff := testutil.Diff(got, want); diff != "" {
			t.Errorf("toRawObject mismatches:\ngot=-, want=+:\n%s", diff)
		}
	}
}

func TestProtoObjectToObjectAttrs(t *testing.T) {
	t.Parallel()
	now := time.Now()
	tests := []struct {
		in   *storagepb.Object
		want *ObjectAttrs
	}{
		{in: nil, want: nil},
		{
			in: &storagepb.Object{
				Bucket:              "Test",
				ContentLanguage:     "en-us",
				ContentType:         "video/mpeg",
				CustomTime:          timestamppb.New(now),
				EventBasedHold:      proto.Bool(false),
				Generation:          7,
				Checksums:           &storagepb.ObjectChecksums{Md5Hash: []byte("14683cba444dbcc6db297645e683f5c1")},
				Name:                "foo.mp4",
				RetentionExpireTime: timestamppb.New(now),
				Size:                1 << 20,
				CreateTime:          timestamppb.New(now),
				DeleteTime:          timestamppb.New(now),
				TemporaryHold:       true,
				ComponentCount:      2,
			},
			want: &ObjectAttrs{
				Bucket:                  "Test",
				Created:                 now,
				ContentLanguage:         "en-us",
				ContentType:             "video/mpeg",
				CustomTime:              now,
				Deleted:                 now,
				EventBasedHold:          false,
				Generation:              7,
				MD5:                     []byte("14683cba444dbcc6db297645e683f5c1"),
				Name:                    "foo.mp4",
				RetentionExpirationTime: now,
				Size:                    1 << 20,
				TemporaryHold:           true,
				ComponentCount:          2,
			},
		},
	}

	for i, tt := range tests {
		got := newObjectFromProto(tt.in)
		if diff := testutil.Diff(got, tt.want); diff != "" {
			t.Errorf("#%d: newObject mismatches:\ngot=-, want=+:\n%s", i, diff)
		}
	}
}

func TestObjectAttrsToProtoObject(t *testing.T) {
	t.Parallel()
	now := time.Now()

	b := "bucket"
	want := &storagepb.Object{
		Bucket:              "projects/_/buckets/" + b,
		ContentLanguage:     "en-us",
		ContentType:         "video/mpeg",
		CustomTime:          timestamppb.New(now),
		EventBasedHold:      proto.Bool(false),
		Generation:          7,
		Name:                "foo.mp4",
		RetentionExpireTime: timestamppb.New(now),
		Size:                1 << 20,
		CreateTime:          timestamppb.New(now),
		DeleteTime:          timestamppb.New(now),
		TemporaryHold:       true,
	}
	in := &ObjectAttrs{
		Created:                 now,
		ContentLanguage:         "en-us",
		ContentType:             "video/mpeg",
		CustomTime:              now,
		Deleted:                 now,
		EventBasedHold:          false,
		Generation:              7,
		Name:                    "foo.mp4",
		RetentionExpirationTime: now,
		Size:                    1 << 20,
		TemporaryHold:           true,
	}

	got := in.toProtoObject(b)
	if diff := testutil.Diff(got, want); diff != "" {
		t.Errorf("toProtoObject mismatches:\ngot=-, want=+:\n%s", diff)
	}
}

func TestApplyCondsProto(t *testing.T) {
	for _, tst := range []struct {
		name     string
		in, want proto.Message
		err      error
		gen      int64
		conds    *Conditions
	}{
		{
			name: "generation",
			gen:  123,
			in:   &storagepb.ReadObjectRequest{},
			want: &storagepb.ReadObjectRequest{Generation: 123},
		},
		{
			name: "invalid_no_generation",
			gen:  123,
			in:   &storagepb.WriteObjectRequest{},
			err:  fmt.Errorf("generation not supported"),
		},
		{
			name:  "if_match",
			gen:   -1,
			in:    &storagepb.ReadObjectRequest{},
			want:  &storagepb.ReadObjectRequest{IfGenerationMatch: proto.Int64(123), IfMetagenerationMatch: proto.Int64(123)},
			conds: &Conditions{GenerationMatch: 123, MetagenerationMatch: 123},
		},
		{
			name:  "if_dne",
			gen:   -1,
			in:    &storagepb.ReadObjectRequest{},
			want:  &storagepb.ReadObjectRequest{IfGenerationMatch: proto.Int64(0)},
			conds: &Conditions{DoesNotExist: true},
		},
		{
			name:  "if_not_match",
			gen:   -1,
			in:    &storagepb.ReadObjectRequest{},
			want:  &storagepb.ReadObjectRequest{IfGenerationNotMatch: proto.Int64(123), IfMetagenerationNotMatch: proto.Int64(123)},
			conds: &Conditions{GenerationNotMatch: 123, MetagenerationNotMatch: 123},
		},
		{
			name:  "invalid_multiple_conditions",
			gen:   -1,
			in:    &storagepb.ReadObjectRequest{},
			conds: &Conditions{MetagenerationMatch: 123, MetagenerationNotMatch: 123},
			err:   fmt.Errorf("multiple conditions"),
		},
	} {
		if err := applyCondsProto(tst.name, tst.gen, tst.conds, tst.in); tst.err == nil && err != nil {
			t.Errorf("%s: error got %v, want nil", tst.name, err)
		} else if tst.err != nil && (err == nil || !strings.Contains(err.Error(), tst.err.Error())) {
			t.Errorf("%s: error got %v, want %v", tst.name, err, tst.err)
		} else if diff := cmp.Diff(tst.in, tst.want, cmp.Comparer(proto.Equal)); tst.err == nil && diff != "" {
			t.Errorf("%s: got(-),want(+):\n%s", tst.name, diff)
		}
	}
}

func TestAttrToFieldMapCoverage(t *testing.T) {
	t.Parallel()

	oa := reflect.TypeOf((*ObjectAttrs)(nil)).Elem()
	oaFields := make(map[string]bool)

	for i := 0; i < oa.NumField(); i++ {
		fieldName := oa.Field(i).Name
		oaFields[fieldName] = true
	}

	// Check that all fields of attrToFieldMap exist in ObjectAttrs.
	for k := range attrToFieldMap {
		if _, ok := oaFields[k]; !ok {
			t.Errorf("%v is not an ObjectAttrs field", k)
		}
	}

	// Check that all fields of ObjectAttrs exist in attrToFieldMap, with
	// known exceptions which aren't sent over the wire but are settable by
	// the user.
	for k := range oaFields {
		if _, ok := attrToFieldMap[k]; !ok {
			if k != "Prefix" && k != "PredefinedACL" {
				t.Errorf("ObjectAttrs.%v is not in attrToFieldMap", k)
			}
		}
	}
}

func TestEmulatorWithCredentialsFile(t *testing.T) {
	t.Setenv("STORAGE_EMULATOR_HOST", "localhost:1234")

	client, err := NewClient(context.Background(), option.WithCredentialsFile("/path/to/key.json"))
	if err != nil {
		t.Fatalf("failed creating a client with credentials file when running agains an emulator: %v", err)
		return
	}
	client.Close()
}

// Create a client using a combination of custom endpoint and
// STORAGE_EMULATOR_HOST env variable and verify that raw.BasePath (used
// for writes) and xmlHost and scheme (used for reads) are all set correctly.
func TestWithEndpoint(t *testing.T) {
	originalStorageEmulatorHost := os.Getenv("STORAGE_EMULATOR_HOST")
	testCases := []struct {
		desc                string
		CustomEndpoint      string
		StorageEmulatorHost string
		WantRawBasePath     string
		WantXMLHost         string
		WantScheme          string
	}{
		{
			desc:                "No endpoint or emulator host specified",
			CustomEndpoint:      "",
			StorageEmulatorHost: "",
			WantRawBasePath:     "https://storage.googleapis.com/storage/v1/",
			WantXMLHost:         "storage.googleapis.com",
			WantScheme:          "https",
		},
		{
			desc:                "With specified https endpoint, no specified emulator host",
			CustomEndpoint:      "https://fake.gcs.com:8080/storage/v1",
			StorageEmulatorHost: "",
			WantRawBasePath:     "https://fake.gcs.com:8080/storage/v1",
			WantXMLHost:         "fake.gcs.com:8080",
			WantScheme:          "https",
		},
		{
			desc:                "With specified http endpoint, no specified emulator host",
			CustomEndpoint:      "http://fake.gcs.com:8080/storage/v1",
			StorageEmulatorHost: "",
			WantRawBasePath:     "http://fake.gcs.com:8080/storage/v1",
			WantXMLHost:         "fake.gcs.com:8080",
			WantScheme:          "http",
		},
		{
			desc:                "Emulator host specified, no specified endpoint",
			CustomEndpoint:      "",
			StorageEmulatorHost: "http://emu.com",
			WantRawBasePath:     "http://emu.com/storage/v1/",
			WantXMLHost:         "emu.com",
			WantScheme:          "http",
		},
		{
			desc:                "Emulator host specified without scheme",
			CustomEndpoint:      "",
			StorageEmulatorHost: "emu.com",
			WantRawBasePath:     "http://emu.com/storage/v1/",
			WantXMLHost:         "emu.com",
			WantScheme:          "http",
		},
		{
			desc:                "Emulator host specified as host:port",
			CustomEndpoint:      "",
			StorageEmulatorHost: "localhost:9000",
			WantRawBasePath:     "http://localhost:9000/storage/v1/",
			WantXMLHost:         "localhost:9000",
			WantScheme:          "http",
		},
		{
			desc:                "Endpoint overrides emulator host when both are specified - https",
			CustomEndpoint:      "https://fake.gcs.com:8080/storage/v1",
			StorageEmulatorHost: "http://emu.com",
			WantRawBasePath:     "https://fake.gcs.com:8080/storage/v1",
			WantXMLHost:         "fake.gcs.com:8080",
			WantScheme:          "https",
		},
		{
			desc:                "Endpoint overrides emulator host when both are specified - http",
			CustomEndpoint:      "http://fake.gcs.com:8080/storage/v1",
			StorageEmulatorHost: "https://emu.com",
			WantRawBasePath:     "http://fake.gcs.com:8080/storage/v1",
			WantXMLHost:         "fake.gcs.com:8080",
			WantScheme:          "http",
		},
		{
			desc:                "Endpoint overrides emulator host when host is specified as scheme://host:port",
			CustomEndpoint:      "http://localhost:8080/storage/v1",
			StorageEmulatorHost: "https://localhost:9000",
			WantRawBasePath:     "http://localhost:8080/storage/v1",
			WantXMLHost:         "localhost:8080",
			WantScheme:          "http",
		},
		{
			desc:                "Endpoint overrides emulator host when host is specified as host:port",
			CustomEndpoint:      "http://localhost:8080/storage/v1",
			StorageEmulatorHost: "localhost:9000",
			WantRawBasePath:     "http://localhost:8080/storage/v1",
			WantXMLHost:         "localhost:8080",
			WantScheme:          "http",
		},
	}
	ctx := context.Background()
	for _, tc := range testCases {
		os.Setenv("STORAGE_EMULATOR_HOST", tc.StorageEmulatorHost)
		c, err := NewClient(ctx, option.WithEndpoint(tc.CustomEndpoint), option.WithoutAuthentication())
		if err != nil {
			t.Fatalf("error creating client: %v", err)
		}

		if c.raw.BasePath != tc.WantRawBasePath {
			t.Errorf("%s: raw.BasePath not set correctly\n\tgot %v, want %v", tc.desc, c.raw.BasePath, tc.WantRawBasePath)
		}
		if c.xmlHost != tc.WantXMLHost {
			t.Errorf("%s: xmlHost not set correctly\n\tgot %v, want %v", tc.desc, c.xmlHost, tc.WantXMLHost)
		}
		if c.scheme != tc.WantScheme {
			t.Errorf("%s: scheme not set correctly\n\tgot %v, want %v", tc.desc, c.scheme, tc.WantScheme)
		}
	}
	os.Setenv("STORAGE_EMULATOR_HOST", originalStorageEmulatorHost)
}

// Create a client using a combination of custom endpoint and STORAGE_EMULATOR_HOST
// env variable and verify that the client hits the correct endpoint for several
// different operations performe in sequence.
// Verifies also that raw.BasePath, xmlHost and scheme are not changed
// after running the operations.
func TestOperationsWithEndpoint(t *testing.T) {
	originalStorageEmulatorHost := os.Getenv("STORAGE_EMULATOR_HOST")
	defer os.Setenv("STORAGE_EMULATOR_HOST", originalStorageEmulatorHost)

	gotURL := make(chan string, 1)
	gotHost := make(chan string, 1)
	gotMethod := make(chan string, 1)

	timedOut := make(chan bool, 1)

	hClient, closeServer := newTestServer(func(w http.ResponseWriter, r *http.Request) {
		done := make(chan bool, 1)
		io.Copy(ioutil.Discard, r.Body)
		fmt.Fprintf(w, "{}")
		go func() {
			gotHost <- r.Host
			gotURL <- r.RequestURI
			gotMethod <- r.Method
			done <- true
		}()

		select {
		case <-timedOut:
		case <-done:
		}

	})
	defer closeServer()

	testCases := []struct {
		desc                string
		CustomEndpoint      string
		StorageEmulatorHost string
		wantScheme          string
		wantHost            string
	}{
		{
			desc:                "No endpoint or emulator host specified",
			CustomEndpoint:      "",
			StorageEmulatorHost: "",
			wantScheme:          "https",
			wantHost:            "storage.googleapis.com",
		},
		{
			desc:                "emulator host specified",
			CustomEndpoint:      "",
			StorageEmulatorHost: "https://" + "addr",
			wantScheme:          "https",
			wantHost:            "addr",
		},
		{
			desc:                "endpoint specified",
			CustomEndpoint:      "https://" + "end" + "/storage/v1/",
			StorageEmulatorHost: "",
			wantScheme:          "https",
			wantHost:            "end",
		},
		{
			desc:                "both emulator and endpoint specified",
			CustomEndpoint:      "https://" + "end" + "/storage/v1/",
			StorageEmulatorHost: "http://host",
			wantScheme:          "https",
			wantHost:            "end",
		},
	}

	for _, tc := range testCases {
		ctx := context.Background()
		t.Run(tc.desc, func(t *testing.T) {
			timeout := time.After(time.Second)
			done := make(chan bool, 1)
			go func() {
				os.Setenv("STORAGE_EMULATOR_HOST", tc.StorageEmulatorHost)

				c, err := NewClient(ctx, option.WithHTTPClient(hClient), option.WithEndpoint(tc.CustomEndpoint))
				if err != nil {
					t.Errorf("error creating client: %v", err)
					return
				}
				originalRawBasePath := c.raw.BasePath
				originalXMLHost := c.xmlHost
				originalScheme := c.scheme

				operations := []struct {
					desc       string
					runOp      func() error
					wantURL    string
					wantMethod string
				}{
					{
						desc: "Create a bucket",
						runOp: func() error {
							return c.Bucket("test-bucket").Create(ctx, "pid", nil)
						},
						wantURL:    "/storage/v1/b?alt=json&prettyPrint=false&project=pid",
						wantMethod: "POST",
					},
					{
						desc: "Upload an object",
						runOp: func() error {
							w := c.Bucket("test-bucket").Object("file").NewWriter(ctx)
							_, err = io.Copy(w, strings.NewReader("copyng into bucket"))
							if err != nil {
								return err
							}
							return w.Close()
						},
						wantURL:    "/upload/storage/v1/b/test-bucket/o?alt=json&name=file&prettyPrint=false&projection=full&uploadType=multipart",
						wantMethod: "POST",
					},
					{
						desc: "Download an object",
						runOp: func() error {
							rc, err := c.Bucket("test-bucket").Object("file").NewReader(ctx)
							if err != nil {
								return err
							}

							_, err = io.Copy(ioutil.Discard, rc)
							if err != nil {
								return err
							}
							return rc.Close()
						},
						wantURL:    "/test-bucket/file",
						wantMethod: "GET",
					},
					{
						desc: "Delete bucket",
						runOp: func() error {
							return c.Bucket("test-bucket").Delete(ctx)
						},
						wantURL:    "/storage/v1/b/test-bucket?alt=json&prettyPrint=false",
						wantMethod: "DELETE",
					},
				}

				// Check that the calls made to the server are as expected
				// given the operations performed
				for _, op := range operations {
					if err := op.runOp(); err != nil {
						t.Errorf("%s: %v", op.desc, err)
					}
					u, method := <-gotURL, <-gotMethod
					if u != op.wantURL {
						t.Errorf("%s: unexpected request URL\ngot  %q\nwant %q",
							op.desc, u, op.wantURL)
					}
					if method != op.wantMethod {
						t.Errorf("%s: unexpected request method\ngot  %q\nwant %q",
							op.desc, method, op.wantMethod)
					}

					if got := <-gotHost; got != tc.wantHost {
						t.Errorf("%s: unexpected request host\ngot  %q\nwant %q",
							op.desc, got, tc.wantHost)
					}
				}

				// Check that the client fields have not changed
				if c.raw.BasePath != originalRawBasePath {
					t.Errorf("raw.BasePath changed\n\tgot:\t\t%v\n\toriginal:\t%v",
						c.raw.BasePath, originalRawBasePath)
				}
				if c.xmlHost != originalXMLHost {
					t.Errorf("xmlHost changed\n\tgot:\t\t%v\n\toriginal:\t%v",
						c.xmlHost, originalXMLHost)
				}
				if c.scheme != originalScheme {
					t.Errorf("scheme changed\n\tgot:\t\t%v\n\toriginal:\t%v",
						c.scheme, originalScheme)
				}
				done <- true
			}()
			select {
			case <-timeout:
				t.Errorf("test timeout")
				timedOut <- true
			case <-done:
			}
		})

	}
}

func TestSignedURLOptionsClone(t *testing.T) {
	t.Parallel()

	opts := &SignedURLOptions{
		GoogleAccessID: "accessID",
		PrivateKey:     []byte{},
		SignBytes: func(b []byte) ([]byte, error) {
			return b, nil
		},
		Method:          "GET",
		Expires:         time.Now(),
		ContentType:     "text/plain",
		Headers:         []string{},
		QueryParameters: map[string][]string{},
		MD5:             "some-checksum",
		Style:           VirtualHostedStyle(),
		Insecure:        true,
		Scheme:          SigningSchemeV2,
		Hostname:        "localhost:8000",
	}

	// Check that all fields are set to a non-zero value, so we can check that
	// clone accurately clones all fields and catch newly added fields not cloned
	reflectOpts := reflect.ValueOf(*opts)
	for i := 0; i < reflectOpts.NumField(); i++ {
		zero, err := isZeroValue(reflectOpts.Field(i))
		if err != nil {
			t.Errorf("IsZero: %v", err)
		}
		if zero {
			t.Errorf("SignedURLOptions field %d not set", i)
		}
	}

	// Check that fields are properly cloned
	optsClone := opts.clone()

	// We need a special comparer for functions
	signBytesComp := func(a func([]byte) ([]byte, error), b func([]byte) ([]byte, error)) bool {
		return reflect.ValueOf(a) == reflect.ValueOf(b)
	}

	if diff := cmp.Diff(opts, optsClone, cmp.Comparer(signBytesComp), cmp.AllowUnexported(SignedURLOptions{})); diff != "" {
		t.Errorf("clone does not match (original: -, cloned: +):\n%s", diff)
	}
}

func TestParseProjectNumber(t *testing.T) {
	for _, tst := range []struct {
		input string
		want  uint64
	}{
		{"projects/123", 123},
		{"projects/123/foos/456", 123},
		{"projects/abc-123/foos/456", 0},
		{"projects/abc-123", 0},
		{"projects/abc", 0},
		{"projects/abc/foos", 0},
	} {
		if got := parseProjectNumber(tst.input); got != tst.want {
			t.Errorf("For %q: got %v, expected %v", tst.input, got, tst.want)
		}
	}
}

func TestObjectValidate(t *testing.T) {
	for _, c := range []struct {
		name        string
		bucket      string
		object      string
		wantSuccess bool
	}{
		{
			name:        "valid object",
			bucket:      "my-bucket",
			object:      "my-object",
			wantSuccess: true,
		},
		{
			name:        "empty bucket name",
			bucket:      "",
			object:      "my-object",
			wantSuccess: false,
		},
		{
			name:        "empty object name",
			bucket:      "my-bucket",
			object:      "",
			wantSuccess: false,
		},
		{
			name:        "invalid utf-8",
			bucket:      "my-bucket",
			object:      "\xc3\x28",
			wantSuccess: false,
		},
		{
			name:        "object name .",
			bucket:      "my-bucket",
			object:      ".",
			wantSuccess: false,
		},
	} {
		t.Run(c.name, func(r *testing.T) {
			b := &BucketHandle{name: c.bucket}
			err := b.Object(c.object).validate()
			if c.wantSuccess && err != nil {
				r.Errorf("want success, got error %v", err)
			}
			if !c.wantSuccess && err == nil {
				r.Errorf("want error, got nil")
			}
		})
	}
}

// isZeroValue reports whether v is the zero value for its type
// It errors if the argument is unknown
func isZeroValue(v reflect.Value) (bool, error) {
	switch v.Kind() {
	case reflect.Bool:
		return !v.Bool(), nil
	case reflect.Int, reflect.Int64:
		return v.Int() == 0, nil
	case reflect.Uint, reflect.Uint64:
		return v.Uint() == 0, nil
	case reflect.Array:
		for i := 0; i < v.Len(); i++ {
			zero, err := isZeroValue(v.Index(i))
			if !zero || err != nil {
				return false, err
			}
		}
		return true, nil
	case reflect.Func, reflect.Interface, reflect.Map, reflect.Slice, reflect.Ptr:
		return v.IsNil(), nil
	case reflect.String:
		return v.Len() == 0, nil
	case reflect.Struct:
		for i := 0; i < v.NumField(); i++ {
			zero, err := isZeroValue(v.Field(i))
			if !zero || err != nil {
				return false, err
			}
		}
		return true, nil
	default:
		return false, fmt.Errorf("unable to check kind %s", v.Kind())
	}
}
