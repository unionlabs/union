// Copyright 2018 Google LLC
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
	"strings"
	"testing"
)

func TestCopyMissingFields(t *testing.T) {
	// Verify that copying checks for missing fields.a
	t.Parallel()
	var tests = []struct {
		srcBucket, srcName, destBucket, destName string
		errMsg                                   string
	}{
		{
			"mybucket", "", "mybucket", "destname",
			"name is empty",
		},
		{
			"mybucket", "srcname", "mybucket", "",
			"name is empty",
		},
		{
			"", "srcfile", "mybucket", "destname",
			"name is empty",
		},
		{
			"mybucket", "srcfile", "", "destname",
			"name is empty",
		},
	}
	ctx := context.Background()
	client := mockClient(t, &mockTransport{})
	for i, test := range tests {
		src := client.Bucket(test.srcBucket).Object(test.srcName)
		dst := client.Bucket(test.destBucket).Object(test.destName)
		_, err := dst.CopierFrom(src).Run(ctx)
		if !strings.Contains(err.Error(), test.errMsg) {
			t.Errorf("CopyTo test #%v:\ngot err  %q\nwant err %q", i, err, test.errMsg)
		}
	}
}

func TestCopyBothEncryptionKeys(t *testing.T) {
	// Test that using both a customer-supplied key and a KMS key is an error.
	ctx := context.Background()
	client := mockClient(t, &mockTransport{})
	dest := client.Bucket("b").Object("d").Key(testEncryptionKey)
	c := dest.CopierFrom(client.Bucket("b").Object("s"))
	c.DestinationKMSKeyName = "key"
	if _, err := c.Run(ctx); err == nil {
		t.Error("got nil, want error")
	} else if !strings.Contains(err.Error(), "KMS") {
		t.Errorf(`got %q, want it to contain "KMS"`, err)
	}
}
