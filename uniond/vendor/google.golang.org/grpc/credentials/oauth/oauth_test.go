/*
 *
 * Copyright 2021 gRPC authors.
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

package oauth

import (
	"strings"
	"testing"
)

func checkErrorMsg(err error, msg string) bool {
	if err == nil && msg == "" {
		return true
	} else if err != nil {
		return strings.Contains(err.Error(), msg)
	}
	return false
}

func TestRemoveServiceNameFromJWTURI(t *testing.T) {
	tests := []struct {
		name         string
		uri          string
		wantedURI    string
		wantedErrMsg string
	}{
		{
			name:         "invalid URI",
			uri:          "ht tp://foo.com",
			wantedErrMsg: "first path segment in URL cannot contain colon",
		},
		{
			name:      "valid URI",
			uri:       "https://foo.com/go/",
			wantedURI: "https://foo.com/",
		},
	}
	for _, tt := range tests {
		t.Run(tt.name, func(t *testing.T) {
			if got, err := removeServiceNameFromJWTURI(tt.uri); got != tt.wantedURI || !checkErrorMsg(err, tt.wantedErrMsg) {
				t.Errorf("RemoveServiceNameFromJWTURI() = %s, %v, want %s, %v", got, err, tt.wantedURI, tt.wantedErrMsg)
			}
		})
	}
}
