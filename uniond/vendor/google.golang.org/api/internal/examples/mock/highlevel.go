// Copyright 2021 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

// Package mock demonstrates how to use interfaces to mock interactions with
// service in tests.
package mock

import (
	"context"
	"fmt"
	"log"
	"os"

	"google.golang.org/api/option"
	"google.golang.org/api/translate/v3"
)

// TranslateService is a facade of a `translate.Service`, specifically used to
// for translating text.
type TranslateService interface {
	TranslateText(text, language string) (string, error)
}

// TranslateTextHighLevel translates text to the given language using the
// provided service.
func TranslateTextHighLevel(service TranslateService, text, language string) (string, error) {
	return service.TranslateText(text, language)
}

type translateService struct {
	svc *translate.Service
}

// NewTranslateService creates a TranslateService.
func NewTranslateService(ctx context.Context, opts ...option.ClientOption) TranslateService {
	svc, err := translate.NewService(ctx, opts...)
	if err != nil {
		log.Fatalf("unable to create translate service, shutting down: %v", err)
	}
	return &translateService{svc}
}

func (t *translateService) TranslateText(text, language string) (string, error) {
	parent := fmt.Sprintf("projects/%s/locations/global", os.Getenv("GOOGLE_CLOUD_PROJECT"))
	resp, err := t.svc.Projects.Locations.TranslateText(parent, &translate.TranslateTextRequest{
		TargetLanguageCode: language,
		Contents:           []string{text},
	}).Do()
	if err != nil {
		return "", fmt.Errorf("unable to translate text: %v", err)
	}
	return resp.Translations[0].TranslatedText, nil
}
