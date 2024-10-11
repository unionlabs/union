// Copyright 2017 Google LLC.
// Use of this source code is governed by a BSD-style
// license that can be found in the LICENSE file.

//go:build go1.19
// +build go1.19

package main

import (
	"bytes"
	"flag"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"testing"

	"github.com/google/go-cmp/cmp"
	"google.golang.org/api/google-api-go-generator/internal/disco"
	"google.golang.org/api/internal"
)

var updateGolden = flag.Bool("update_golden", false, "If true, causes TestAPIs to update golden files")

func TestAPIs(t *testing.T) {
	*copyrightYear = "YEAR"

	names := []string{
		"any",
		"arrayofarray-1",
		"arrayofenum",
		"arrayofmapofobjects",
		"arrayofmapofstrings",
		"blogger-3",
		"floats",
		"getwithoutbody",
		"http-body",
		"json-body",
		"mapofany",
		"mapofarrayofobjects",
		"mapprotostruct",
		"mapofint64strings",
		"mapofobjects",
		"mapofstrings-1",
		"param-rename",
		"quotednum",
		"repeated",
		"repeated_any_query_error",
		"required-query",
		"resource-named-service", // appengine/v1/appengine-api.json
		"unfortunatedefaults",
		"variants",
		"wrapnewlines",
	}
	for _, name := range names {
		t.Run(name, func(t *testing.T) {
			defer func() {
				r := recover()
				wantPanic := strings.HasSuffix(name, "_error")
				if r != nil && !wantPanic {
					t.Fatal("unexpected panic", r)
				}
				if r == nil && !wantPanic {
					return
				}
				if r == nil && wantPanic {
					t.Fatal("wanted test to panic, but it didn't")
				}

				// compare panic message received vs. desired
				got, ok := r.(string)
				if !ok {
					gotE, okE := r.(error)
					if !okE {
						t.Fatalf("panic with non-string/error input: %v", r)
					}
					got = gotE.Error()
				}
				want, err := readOrUpdate(name, got)
				if err != nil {
					t.Fatal(err)
				}
				if diff := cmp.Diff(got, string(want)); diff != "" {
					t.Errorf("got(-),want(+):\n %s", diff)
				}
			}()
			api, err := apiFromFile(filepath.Join("testdata", name+".json"))
			if err != nil {
				t.Fatalf("Error loading API testdata/%s.json: %v", name, err)
			}
			clean, err := api.GenerateCode()
			if err != nil {
				t.Fatalf("Error generating code for %s: %v", name, err)
			}

			want, err := readOrUpdate(name, string(clean))
			if err != nil {
				t.Fatal(err)
			}

			wantStr := strings.Replace(string(want), "gdcl/00000000", fmt.Sprintf("gdcl/%s", internal.Version), -1)
			if !bytes.Equal([]byte(wantStr), clean) {
				tf, _ := os.CreateTemp("", "api-"+name+"-got-json.")
				if _, err := tf.Write(clean); err != nil {
					t.Fatal(err)
				}
				if err := tf.Close(); err != nil {
					t.Fatal(err)
				}
				// NOTE: update golden files with `go test -update_golden`
				t.Errorf("Output for API %s differs: diff -u %s %s", name, goldenFileName(name), tf.Name())
			}
		})
	}
}

func readOrUpdate(name, clean string) ([]byte, error) {
	goldenFile := goldenFileName(name)
	if *updateGolden {
		clean := strings.Replace(string(clean), fmt.Sprintf("gdcl/%s", internal.Version), "gdcl/00000000", -1)
		if err := os.WriteFile(goldenFile, []byte(clean), 0644); err != nil {
			return nil, err
		}
	}
	return os.ReadFile(goldenFile)
}

func goldenFileName(name string) string {
	return filepath.Join("testdata", name+".want")
}

func TestScope(t *testing.T) {
	tests := [][]string{
		{
			"https://www.googleapis.com/auth/somescope",
			"SomescopeScope",
		},
		{
			"https://mail.google.com/somescope",
			"MailGoogleComSomescopeScope",
		},
		{
			"https://mail.google.com/",
			"MailGoogleComScope",
		},
	}
	for _, test := range tests {
		if got := scopeIdentifier(disco.Scope{ID: test[0]}); got != test[1] {
			t.Errorf("scopeIdentifier(%q) got %q, want %q", test[0], got, test[1])
		}
	}
}

func TestDepunct(t *testing.T) {
	tests := []struct {
		needCap  bool
		in, want string
	}{
		{
			needCap: true,
			in:      "part__description",
			want:    "Part__description",
		},
		{
			needCap: true,
			in:      "Part_description",
			want:    "PartDescription",
		},
		{
			needCap: false,
			in:      "part_description",
			want:    "partDescription",
		},
		{
			needCap: false,
			in:      "part-description",
			want:    "partDescription",
		},
		{
			needCap: false,
			in:      "part.description",
			want:    "partDescription",
		},
		{
			needCap: false,
			in:      "part$description",
			want:    "partDescription",
		},
		{
			needCap: false,
			in:      "part/description",
			want:    "partDescription",
		},
		{
			needCap: true,
			in:      "Part__description_name",
			want:    "Part__descriptionName",
		},
		{
			needCap: true,
			in:      "Part_description_name",
			want:    "PartDescriptionName",
		},
		{
			needCap: true,
			in:      "Part__description__name",
			want:    "Part__description__name",
		},
		{
			needCap: true,
			in:      "Part_description__name",
			want:    "PartDescription__name",
		},
	}
	for _, test := range tests {
		if got := depunct(test.in, test.needCap); got != test.want {
			t.Errorf("depunct(%q,%v) = %q; want %q", test.in, test.needCap, got, test.want)
		}
	}
}

func TestRenameVersion(t *testing.T) {
	tests := []struct {
		version, want string
	}{
		{
			version: "directory_v1",
			want:    "directory/v1",
		},
		{
			version: "email_migration_v1",
			want:    "email_migration/v1",
		},
		{
			version: "my_api_v1.2",
			want:    "my_api/v1.2",
		},
	}
	for _, test := range tests {
		if got := renameVersion(test.version); got != test.want {
			t.Errorf("renameVersion(%q) = %q; want %q", test.version, got, test.want)
		}
	}
}

func TestSupportsPaging(t *testing.T) {
	api, err := apiFromFile(filepath.Join("testdata", "paging.json"))
	if err != nil {
		t.Fatalf("Error loading API testdata/paging.json: %v", err)
	}
	api.PopulateSchemas()
	res := api.doc.Resources[0]
	for _, meth := range api.resourceMethods(res) {
		_, _, got := meth.supportsPaging()
		want := strings.HasPrefix(meth.m.Name, "yes")
		if got != want {
			t.Errorf("method %s supports paging: got %t, want %t", meth.m.Name, got, want)
		}
	}
}

func TestIsNewerRevision(t *testing.T) {
	olderBytesPath, newerBytesPath := filepath.Join("testdata", "rev20200415.json"), filepath.Join("testdata", "rev20200416.json")
	olderBytes, err := os.ReadFile(olderBytesPath)
	if err != nil {
		t.Fatalf("os.ReadFile(%q) = %v; want nil", olderBytesPath, err)
	}
	newerBytes, err := os.ReadFile(newerBytesPath)
	if err != nil {
		t.Fatalf("os.ReadFile(%q) = %v; want nil", newerBytesPath, err)
	}

	// newBytes > oldBytes
	if err := isNewerRevision(olderBytes, newerBytes); err != nil {
		t.Fatalf("isNewerRevision(%q, %q) = %v; want nil", string(olderBytes), string(newerBytes), err)
	}
	// newBytes == newBytes
	if err := isNewerRevision(newerBytes, newerBytes); err != nil {
		t.Fatalf("isNewerRevision(%q, %q) = %v; want nil", string(newerBytes), string(newerBytes), err)
	}
	// newBytes < newBytes
	err = isNewerRevision(newerBytes, olderBytes)
	if err == nil {
		t.Fatalf("isNewerRevision(%q, %q) = nil; want %v", string(newerBytes), string(olderBytes), errOldRevision)
	}
	if err != errOldRevision {
		t.Fatalf("got %v, want %v", err, errOldRevision)
	}
}

func TestRemoveMarkdownLinks(t *testing.T) {
	tests := []struct {
		name  string
		input string
		want  string
	}{
		{name: "basic", input: "[name](link)", want: "name (link)"},
		{name: "no link", input: "name", want: "name"},
		{name: "empty string", input: "", want: ""},
		{name: "sentence", input: "This [is](link) a test.", want: "This is (link) a test."},
		{name: "two links", input: "This [is](link) a [test](link).", want: "This is (link) a test (link)."},
		{name: "first incomplete link", input: "This [is] a [test](link).", want: "This [is] a test (link)."},
		{name: "second incomplete link", input: "This [is](link) a (test).", want: "This is (link) a (test)."},
		{name: "seperated", input: "This [is] (a) test.", want: "This [is] (a) test."},
		{name: "don't process code blocks", input: "This is `[a](link)` test.", want: "This is `[a](link)` test."},
		{name: "At start", input: "[This](link) is a test.", want: "This (link) is a test."},
		{name: "At end ", input: "This is a [test.](link)", want: "This is a test. (link)"},
	}
	for _, tc := range tests {
		t.Run(tc.name, func(t *testing.T) {
			if got := removeMarkdownLinks(tc.input); got != tc.want {
				t.Errorf("removeMarkdownLinks(%q) = %q, want %q", tc.input, got, tc.want)
			}
		})
	}
}

func TestAsComment_LongLink(t *testing.T) {
	//input := "The specification of cohorts for a cohort report. Cohort reports create a time series of user retention for the cohort. For example, you could select the cohort of users that were acquired in the first week of September and follow that cohort for the next six weeks. Selecting the users acquired in the first week of September cohort is specified in the `cohort` object. Following that cohort for the next six weeks is specified in the `cohortsRange` object. For examples, see [Cohort Report Examples](https://developers.google.com/analytics/devguides/reporting/data/v1/advanced#cohort_report_examples). The report response could show a weekly time series where say your app has retained 60% of this cohort after three weeks and 25% of this cohort after six weeks. These two percentages can be calculated by the metric `cohortActiveUsers/cohortTotalUsers` and will be separate rows in the report."
	input := "This make sure we don't split long links (http://example.com/really/really/really/really/really/really/really/really/really/really/really/long). We want them to show up well in godoc."
	want := `// This make sure we don't split long links
// (http://example.com/really/really/really/really/really/really/really/really/really/really/really/long).
// We want them to show up well in godoc.
`
	got := asComment("", input)
	if got != want {
		t.Errorf("got %q, want %q", got, want)
	}
}

func TestApiBaseURLTemplate(t *testing.T) {
	tests := []struct {
		name, want string
	}{
		{
			name: "any.json",
			want: "https://logging.UNIVERSE_DOMAIN/",
		},
		{
			name: "blogger-3.json",
			want: "https://www.UNIVERSE_DOMAIN/blogger/v3/",
		},
		{
			name: "required-query.json",
			want: "https://www.UNIVERSE_DOMAIN/_ah/api/tshealth/v1/",
		},
	}
	for _, tt := range tests {
		api, err := apiFromFile(filepath.Join("testdata", tt.name))
		if err != nil {
			t.Fatalf("Error loading API testdata/%s: %v", tt.name, err)
		}
		got, err := api.apiBaseURLTemplate()
		if err != nil {
			t.Fatalf("%s: apiBaseURLTemplate(): %v", tt.name, err)
		}
		if got != tt.want {
			t.Errorf("%s: apiBaseURLTemplate() = %q; want %q", tt.name, got, tt.want)
		}
	}
}
