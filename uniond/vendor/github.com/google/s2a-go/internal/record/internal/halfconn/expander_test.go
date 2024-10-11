/*
 *
 * Copyright 2021 Google LLC
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package halfconn

import (
	"bytes"
	"crypto/sha256"
	"testing"

	"github.com/google/s2a-go/internal/record/internal/aeadcrypter/testutil"
)

func TestExpand(t *testing.T) {
	// The following test vectors were taken from
	// https://tools.ietf.org/html/rfc5869. Note that `prk` and `okm`
	// mentioned in the RFC have been renamed to `secret` and `out`.
	for _, tc := range []struct {
		desc              string
		secret, info, out []byte
		length            int
	}{
		{
			desc:   "sha256 basic",
			secret: testutil.Dehex("077709362c2e32df0ddc3f0dc47bba6390b6c73bb50f9c3122ec844ad7c2b3e5"),
			info:   testutil.Dehex("f0f1f2f3f4f5f6f7f8f9"),
			out:    testutil.Dehex("3cb25f25faacd57a90434f64d0362f2a2d2d0a90cf1a5a4c5db02d56ecc4c5bf34007208d5b887185865"),
			length: 42,
		},
		{
			desc:   "sha256 longer input/output",
			secret: testutil.Dehex("06a6b88c5853361a06104c9ceb35b45cef760014904671014a193f40c15fc244"),
			info:   testutil.Dehex("b0b1b2b3b4b5b6b7b8b9babbbcbdbebfc0c1c2c3c4c5c6c7c8c9cacbcccdcecfd0d1d2d3d4d5d6d7d8d9dadbdcdddedfe0e1e2e3e4e5e6e7e8e9eaebecedeeeff0f1f2f3f4f5f6f7f8f9fafbfcfdfeff"),
			out:    testutil.Dehex("b11e398dc80327a1c8e7f78c596a49344f012eda2d4efad8a050cc4c19afa97c59045a99cac7827271cb41c65e590e09da3275600c2f09b8367793a9aca3db71cc30c58179ec3e87c14c01d5c1f3434f1d87"),
			length: 82,
		},
		{
			desc:   "sha256 zero length info",
			secret: testutil.Dehex("19ef24a32c717b167f33a91d6f648bdf96596776afdb6377ac434c1c293ccb04"),
			out:    testutil.Dehex("8da4e775a563c18f715f802a063c5a31b8a11f5c5ee1879ec3454e5f3c738d2d9d201395faa4b61a96c8"),
			length: 42,
		},
	} {
		t.Run(tc.desc, func(t *testing.T) {
			expander := newDefaultHKDFExpander(sha256.New)
			got, err := expander.expand(tc.secret, tc.info, tc.length)
			if err != nil {
				t.Errorf("expand failed with error: %v", err)
			}
			if !bytes.Equal(got, tc.out) {
				t.Errorf("expand(sha256.New, %v, %v, %v) = %v, want %v.", tc.secret, tc.info, tc.length, got, tc.out)
			}
		})
	}
}
