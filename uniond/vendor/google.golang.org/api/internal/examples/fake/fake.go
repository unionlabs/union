// Copyright 2021 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Package fake demonstrates how to use concrete services and fake the
// interactions with them in tests.
package fake

import (
	"fmt"
	"os"

	"google.golang.org/api/translate/v3"
)

// TranslateText translates text to the given language using the provided
// service.
func TranslateText(service *translate.Service, text, language string) (string, error) {
	parent := fmt.Sprintf("projects/%s/locations/global", os.Getenv("GOOGLE_CLOUD_PROJECT"))
	req := &translate.TranslateTextRequest{
		TargetLanguageCode: language,
		Contents:           []string{text},
	}
	resp, err := service.Projects.Locations.TranslateText(parent, req).Do()
	if err != nil {
		return "", fmt.Errorf("unable to translate text: %v", err)
	}
	return resp.Translations[0].TranslatedText, nil
}
