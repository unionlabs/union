// Copyright 2015 The Prometheus Authors
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

package model

import (
	"strings"
	"testing"
	"time"
)

func TestMatcherValidate(t *testing.T) {
	cases := []struct {
		matcher   *Matcher
		legacyErr string
		utf8Err   string
	}{
		{
			matcher: &Matcher{
				Name:  "name",
				Value: "value",
			},
		},
		{
			matcher: &Matcher{
				Name:    "name",
				Value:   "value",
				IsRegex: true,
			},
		},
		{
			matcher: &Matcher{
				Name:  "name!",
				Value: "value",
			},
			legacyErr: "invalid name",
		},
		{
			matcher: &Matcher{
				Name:  "",
				Value: "value",
			},
			legacyErr: "invalid name",
			utf8Err:   "invalid name",
		},
		{
			matcher: &Matcher{
				Name:  "name",
				Value: "value\xff",
			},
			legacyErr: "invalid value",
			utf8Err:   "invalid value",
		},
		{
			matcher: &Matcher{
				Name:  "name",
				Value: "",
			},
			legacyErr: "invalid value",
			utf8Err:   "invalid value",
		},
		{
			matcher: &Matcher{
				Name:  "a\xc5z",
				Value: "",
			},
			legacyErr: "invalid name",
			utf8Err:   "invalid name",
		},
	}

	for i, c := range cases {
		NameValidationScheme = LegacyValidation
		legacyErr := c.matcher.Validate()
		NameValidationScheme = UTF8Validation
		utf8Err := c.matcher.Validate()
		if legacyErr == nil && utf8Err == nil {
			if c.legacyErr == "" && c.utf8Err == "" {
				continue
			}
			if c.legacyErr != "" {
				t.Errorf("%d. Expected error for legacy validation %q but got none", i, c.legacyErr)
			}
			if c.utf8Err != "" {
				t.Errorf("%d. Expected error for utf-8 validation %q but got none", i, c.utf8Err)
			}
			continue
		}
		if legacyErr != nil {
			if c.legacyErr == "" {
				t.Errorf("%d. Expected no legacy validation error but got %q", i, legacyErr)
			} else if !strings.Contains(legacyErr.Error(), c.legacyErr) {
				t.Errorf("%d. Expected error to contain %q but got %q", i, c.legacyErr, legacyErr)
			}
		}
		if utf8Err != nil {
			if c.utf8Err == "" {
				t.Errorf("%d. Expected no utf-8 validation error but got %q", i, utf8Err)
				continue
			}
			if !strings.Contains(utf8Err.Error(), c.utf8Err) {
				t.Errorf("%d. Expected error to contain %q but got %q", i, c.utf8Err, utf8Err)
			}
		}
	}
}

func TestSilenceValidate(t *testing.T) {
	ts := time.Now()

	cases := []struct {
		sil *Silence
		err string
	}{
		{
			sil: &Silence{
				Matchers: []*Matcher{
					{Name: "name", Value: "value"},
				},
				StartsAt:  ts,
				EndsAt:    ts,
				CreatedAt: ts,
				CreatedBy: "name",
				Comment:   "comment",
			},
		},
		{
			sil: &Silence{
				Matchers: []*Matcher{
					{Name: "name", Value: "value"},
					{Name: "name", Value: "value"},
					{Name: "name", Value: "value"},
					{Name: "name", Value: "value", IsRegex: true},
				},
				StartsAt:  ts,
				EndsAt:    ts,
				CreatedAt: ts,
				CreatedBy: "name",
				Comment:   "comment",
			},
		},
		{
			sil: &Silence{
				Matchers: []*Matcher{
					{Name: "name", Value: "value"},
				},
				StartsAt:  ts,
				EndsAt:    ts.Add(-1 * time.Minute),
				CreatedAt: ts,
				CreatedBy: "name",
				Comment:   "comment",
			},
			err: "start time must be before end time",
		},
		{
			sil: &Silence{
				Matchers: []*Matcher{
					{Name: "name", Value: "value"},
				},
				StartsAt:  ts,
				CreatedAt: ts,
				CreatedBy: "name",
				Comment:   "comment",
			},
			err: "end time missing",
		},
		{
			sil: &Silence{
				Matchers: []*Matcher{
					{Name: "name", Value: "value"},
				},
				EndsAt:    ts,
				CreatedAt: ts,
				CreatedBy: "name",
				Comment:   "comment",
			},
			err: "start time missing",
		},
		{
			sil: &Silence{
				Matchers: []*Matcher{
					{Name: "!name", Value: "value"},
				},
				StartsAt:  ts,
				EndsAt:    ts,
				CreatedAt: ts,
				CreatedBy: "name",
				Comment:   "comment",
			},
			err: "invalid matcher",
		},
		{
			sil: &Silence{
				Matchers: []*Matcher{
					{Name: "name", Value: "value"},
				},
				StartsAt:  ts,
				EndsAt:    ts,
				CreatedAt: ts,
				CreatedBy: "name",
			},
			err: "comment missing",
		},
		{
			sil: &Silence{
				Matchers: []*Matcher{
					{Name: "name", Value: "value"},
				},
				StartsAt:  ts,
				EndsAt:    ts,
				CreatedBy: "name",
				Comment:   "comment",
			},
			err: "creation timestamp missing",
		},
		{
			sil: &Silence{
				Matchers: []*Matcher{
					{Name: "name", Value: "value"},
				},
				StartsAt:  ts,
				EndsAt:    ts,
				CreatedAt: ts,
				Comment:   "comment",
			},
			err: "creator information missing",
		},
		{
			sil: &Silence{
				Matchers:  []*Matcher{},
				StartsAt:  ts,
				EndsAt:    ts,
				CreatedAt: ts,
				Comment:   "comment",
			},
			err: "at least one matcher required",
		},
	}

	for i, c := range cases {
		NameValidationScheme = LegacyValidation
		err := c.sil.Validate()
		if err == nil {
			if c.err == "" {
				continue
			}
			t.Errorf("%d. Expected error %q but got none", i, c.err)
			continue
		}
		if c.err == "" {
			t.Errorf("%d. Expected no error but got %q", i, err)
			continue
		}
		if !strings.Contains(err.Error(), c.err) {
			t.Errorf("%d. Expected error to contain %q but got %q", i, c.err, err)
		}
	}
}
