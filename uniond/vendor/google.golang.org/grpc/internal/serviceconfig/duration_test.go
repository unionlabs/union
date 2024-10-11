/*
 *
 * Copyright 2023 gRPC authors.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 *
 */

package serviceconfig

import (
	"fmt"
	"math"
	"math/rand"
	"strings"
	"testing"
	"time"
)

// Tests both marshalling and unmarshalling of Durations.
func TestDuration_MarshalUnmarshal(t *testing.T) {
	testCases := []struct {
		json         string
		td           time.Duration
		unmarshalErr error
		noMarshal    bool
	}{
		// Basic values.
		{json: `"1s"`, td: time.Second},
		{json: `"-100.700s"`, td: -100*time.Second - 700*time.Millisecond},
		{json: `".050s"`, td: 50 * time.Millisecond, noMarshal: true},
		{json: `"-.001s"`, td: -1 * time.Millisecond, noMarshal: true},
		{json: `"-0.200s"`, td: -200 * time.Millisecond},
		// Positive near / out of bounds.
		{json: `"9223372036s"`, td: 9223372036 * time.Second},
		{json: `"9223372037s"`, td: math.MaxInt64, noMarshal: true},
		{json: `"9223372036.854775807s"`, td: math.MaxInt64},
		{json: `"9223372036.854775808s"`, td: math.MaxInt64, noMarshal: true},
		{json: `"315576000000s"`, td: math.MaxInt64, noMarshal: true},
		{json: `"315576000001s"`, unmarshalErr: fmt.Errorf("out of range")},
		// Negative near / out of bounds.
		{json: `"-9223372036s"`, td: -9223372036 * time.Second},
		{json: `"-9223372037s"`, td: math.MinInt64, noMarshal: true},
		{json: `"-9223372036.854775808s"`, td: math.MinInt64},
		{json: `"-9223372036.854775809s"`, td: math.MinInt64, noMarshal: true},
		{json: `"-315576000000s"`, td: math.MinInt64, noMarshal: true},
		{json: `"-315576000001s"`, unmarshalErr: fmt.Errorf("out of range")},
		// Parse errors.
		{json: `123s`, unmarshalErr: fmt.Errorf("invalid character")},
		{json: `"5m"`, unmarshalErr: fmt.Errorf("malformed duration")},
		{json: `"5.3.2s"`, unmarshalErr: fmt.Errorf("malformed duration")},
		{json: `"x.3s"`, unmarshalErr: fmt.Errorf("malformed duration")},
		{json: `"3.xs"`, unmarshalErr: fmt.Errorf("malformed duration")},
		{json: `"3.1234567890s"`, unmarshalErr: fmt.Errorf("malformed duration")},
		{json: `".s"`, unmarshalErr: fmt.Errorf("malformed duration")},
		{json: `"s"`, unmarshalErr: fmt.Errorf("malformed duration")},
	}
	for _, tc := range testCases {
		// Seed `got` with a random value to ensure we properly reset it in all
		// non-error cases.
		got := Duration(rand.Uint64())
		err := got.UnmarshalJSON([]byte(tc.json))
		if (err == nil && time.Duration(got) != tc.td) ||
			(err != nil) != (tc.unmarshalErr != nil) || !strings.Contains(fmt.Sprint(err), fmt.Sprint(tc.unmarshalErr)) {
			t.Errorf("UnmarshalJSON of %v = %v, %v; want %v, %v", tc.json, time.Duration(got), err, tc.td, tc.unmarshalErr)
		}

		if tc.unmarshalErr == nil && !tc.noMarshal {
			d := Duration(tc.td)
			got, err := d.MarshalJSON()
			if string(got) != tc.json || err != nil {
				t.Errorf("MarshalJSON of %v = %v, %v; want %v, nil", d, string(got), err, tc.json)
			}
		}
	}
}
