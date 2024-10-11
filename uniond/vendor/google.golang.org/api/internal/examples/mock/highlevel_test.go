// Copyright 2021 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package mock

import "testing"

// mockService fulfills the TranslateService interface.
type mockService struct{}

func (*mockService) TranslateText(text, language string) (string, error) {
	return "Hello World", nil
}
func TestTranslateTextHighLevel(t *testing.T) {
	svc := &mockService{}
	text, err := TranslateTextHighLevel(svc, "Hola Mundo", "en-US")
	if err != nil {
		t.Fatal(err)
	}
	if text != "Hello World" {
		t.Fatalf("got %q, want Hello World", text)
	}
}
