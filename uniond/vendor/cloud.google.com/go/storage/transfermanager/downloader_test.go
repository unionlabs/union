// Copyright 2024 Google LLC
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

package transfermanager

import (
	"context"
	"strings"
	"testing"
)

func TestWaitAndClose(t *testing.T) {
	d, err := NewDownloader(nil)
	if err != nil {
		t.Fatalf("NewDownloader: %v", err)
	}

	if _, err := d.WaitAndClose(); err != nil {
		t.Fatalf("WaitAndClose: %v", err)
	}

	expectedErr := "transfermanager: WaitAndClose called before DownloadObject"
	err = d.DownloadObject(context.Background(), &DownloadObjectInput{})
	if err == nil {
		t.Fatalf("d.DownloadObject err was nil, should be %q", expectedErr)
	}
	if !strings.Contains(err.Error(), expectedErr) {
		t.Errorf("expected err %q, got: %v", expectedErr, err.Error())
	}
}
