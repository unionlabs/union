// Copyright 2021 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

package fake

import (
	"context"
	"encoding/json"
	"net/http"
	"net/http/httptest"
	"testing"

	"google.golang.org/api/option"
	"google.golang.org/api/translate/v3"
)

func TestTranslateText(t *testing.T) {
	ctx := context.Background()
	ts := httptest.NewServer(http.HandlerFunc(func(w http.ResponseWriter, r *http.Request) {
		resp := &translate.TranslateTextResponse{
			Translations: []*translate.Translation{
				{TranslatedText: "Hello World"},
			},
		}
		b, err := json.Marshal(resp)
		if err != nil {
			http.Error(w, "unable to marshal request: "+err.Error(), http.StatusBadRequest)
			return
		}
		w.Write(b)
	}))
	defer ts.Close()
	svc, err := translate.NewService(ctx, option.WithoutAuthentication(), option.WithEndpoint(ts.URL))
	if err != nil {
		t.Fatalf("unable to create client: %v", err)
	}
	text, err := TranslateText(svc, "Hola Mundo", "en-US")
	if err != nil {
		t.Fatal(err)
	}
	if text != "Hello World" {
		t.Fatalf("got %q, want Hello World", text)
	}
}
