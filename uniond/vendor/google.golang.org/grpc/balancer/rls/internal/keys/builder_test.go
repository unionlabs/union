/*
 *
 * Copyright 2020 gRPC authors.
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

package keys

import (
	"fmt"
	"strings"
	"testing"

	"github.com/google/go-cmp/cmp"
	rlspb "google.golang.org/grpc/internal/proto/grpc_lookup_v1"
	"google.golang.org/grpc/metadata"
)

var (
	goodKeyBuilder1 = &rlspb.GrpcKeyBuilder{
		Names: []*rlspb.GrpcKeyBuilder_Name{
			{Service: "gFoo"},
		},
		Headers: []*rlspb.NameMatcher{
			{Key: "k1", Names: []string{"n1"}},
			{Key: "k2", Names: []string{"n1"}},
		},
		ExtraKeys: &rlspb.GrpcKeyBuilder_ExtraKeys{
			Host:    "host",
			Service: "service",
			Method:  "method",
		},
		ConstantKeys: map[string]string{
			"const-key-1": "const-val-1",
			"const-key-2": "const-val-2",
		},
	}
	goodKeyBuilder2 = &rlspb.GrpcKeyBuilder{
		Names: []*rlspb.GrpcKeyBuilder_Name{
			{Service: "gBar", Method: "method1"},
			{Service: "gFoobar"},
		},
		Headers: []*rlspb.NameMatcher{
			{Key: "k1", Names: []string{"n1", "n2"}},
		},
	}
)

func TestMakeBuilderMap(t *testing.T) {
	gFooBuilder := builder{
		headerKeys: []matcher{{key: "k1", names: []string{"n1"}}, {key: "k2", names: []string{"n1"}}},
		constantKeys: map[string]string{
			"const-key-1": "const-val-1",
			"const-key-2": "const-val-2",
		},
		hostKey:    "host",
		serviceKey: "service",
		methodKey:  "method",
	}
	wantBuilderMap1 := map[string]builder{"/gFoo/": gFooBuilder}
	wantBuilderMap2 := map[string]builder{
		"/gFoo/":        gFooBuilder,
		"/gBar/method1": {headerKeys: []matcher{{key: "k1", names: []string{"n1", "n2"}}}},
		"/gFoobar/":     {headerKeys: []matcher{{key: "k1", names: []string{"n1", "n2"}}}},
	}

	tests := []struct {
		desc           string
		cfg            *rlspb.RouteLookupConfig
		wantBuilderMap BuilderMap
	}{
		{
			desc: "One good GrpcKeyBuilder",
			cfg: &rlspb.RouteLookupConfig{
				GrpcKeybuilders: []*rlspb.GrpcKeyBuilder{goodKeyBuilder1},
			},
			wantBuilderMap: wantBuilderMap1,
		},
		{
			desc: "Two good GrpcKeyBuilders",
			cfg: &rlspb.RouteLookupConfig{
				GrpcKeybuilders: []*rlspb.GrpcKeyBuilder{goodKeyBuilder1, goodKeyBuilder2},
			},
			wantBuilderMap: wantBuilderMap2,
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			builderMap, err := MakeBuilderMap(test.cfg)
			if err != nil || !builderMap.Equal(test.wantBuilderMap) {
				t.Errorf("MakeBuilderMap(%+v) returned {%v, %v}, want: {%v, nil}", test.cfg, builderMap, err, test.wantBuilderMap)
			}
		})
	}
}

func TestMakeBuilderMapErrors(t *testing.T) {
	tests := []struct {
		desc          string
		cfg           *rlspb.RouteLookupConfig
		wantErrPrefix string
	}{
		{
			desc:          "No GrpcKeyBuilder",
			cfg:           &rlspb.RouteLookupConfig{},
			wantErrPrefix: "rls: RouteLookupConfig does not contain any GrpcKeyBuilder",
		},
		{
			desc: "Two GrpcKeyBuilders with same Name",
			cfg: &rlspb.RouteLookupConfig{
				GrpcKeybuilders: []*rlspb.GrpcKeyBuilder{goodKeyBuilder1, goodKeyBuilder1},
			},
			wantErrPrefix: "rls: GrpcKeyBuilder in RouteLookupConfig contains repeated Name field",
		},
		{
			desc: "GrpcKeyBuilder with empty Service field",
			cfg: &rlspb.RouteLookupConfig{
				GrpcKeybuilders: []*rlspb.GrpcKeyBuilder{
					{
						Names: []*rlspb.GrpcKeyBuilder_Name{
							{Service: "bFoo", Method: "method1"},
							{Service: "bBar"},
							{Method: "method1"},
						},
						Headers: []*rlspb.NameMatcher{{Key: "k1", Names: []string{"n1", "n2"}}},
					},
					goodKeyBuilder1,
				},
			},
			wantErrPrefix: "rls: GrpcKeyBuilder in RouteLookupConfig contains a Name field with no Service",
		},
		{
			desc: "GrpcKeyBuilder with no Name",
			cfg: &rlspb.RouteLookupConfig{
				GrpcKeybuilders: []*rlspb.GrpcKeyBuilder{{}, goodKeyBuilder1},
			},
			wantErrPrefix: "rls: GrpcKeyBuilder in RouteLookupConfig does not contain any Name",
		},
		{
			desc: "GrpcKeyBuilder with requiredMatch field set",
			cfg: &rlspb.RouteLookupConfig{
				GrpcKeybuilders: []*rlspb.GrpcKeyBuilder{
					{
						Names:   []*rlspb.GrpcKeyBuilder_Name{{Service: "bFoo", Method: "method1"}},
						Headers: []*rlspb.NameMatcher{{Key: "k1", Names: []string{"n1", "n2"}, RequiredMatch: true}},
					},
					goodKeyBuilder1,
				},
			},
			wantErrPrefix: "rls: GrpcKeyBuilder in RouteLookupConfig has required_match field set",
		},
		{
			desc: "GrpcKeyBuilder two headers with same key",
			cfg: &rlspb.RouteLookupConfig{
				GrpcKeybuilders: []*rlspb.GrpcKeyBuilder{
					{
						Names: []*rlspb.GrpcKeyBuilder_Name{
							{Service: "gBar", Method: "method1"},
							{Service: "gFoobar"},
						},
						Headers: []*rlspb.NameMatcher{
							{Key: "k1", Names: []string{"n1", "n2"}},
							{Key: "k1", Names: []string{"n1", "n2"}},
						},
					},
					goodKeyBuilder1,
				},
			},
			wantErrPrefix: "rls: GrpcKeyBuilder in RouteLookupConfig contains repeated key \"k1\" across headers, constant_keys and extra_keys",
		},
		{
			desc: "GrpcKeyBuilder repeated keys across headers and constant_keys",
			cfg: &rlspb.RouteLookupConfig{
				GrpcKeybuilders: []*rlspb.GrpcKeyBuilder{
					{
						Names: []*rlspb.GrpcKeyBuilder_Name{
							{Service: "gBar", Method: "method1"},
							{Service: "gFoobar"},
						},
						Headers:      []*rlspb.NameMatcher{{Key: "k1", Names: []string{"n1", "n2"}}},
						ConstantKeys: map[string]string{"k1": "v1"},
					},
					goodKeyBuilder1,
				},
			},
			wantErrPrefix: "rls: GrpcKeyBuilder in RouteLookupConfig contains repeated key \"k1\" across headers, constant_keys and extra_keys",
		},
		{
			desc: "GrpcKeyBuilder repeated keys across headers and extra_keys",
			cfg: &rlspb.RouteLookupConfig{
				GrpcKeybuilders: []*rlspb.GrpcKeyBuilder{
					{
						Names: []*rlspb.GrpcKeyBuilder_Name{
							{Service: "gBar", Method: "method1"},
							{Service: "gFoobar"},
						},
						Headers:   []*rlspb.NameMatcher{{Key: "k1", Names: []string{"n1", "n2"}}},
						ExtraKeys: &rlspb.GrpcKeyBuilder_ExtraKeys{Method: "k1"},
					},
					goodKeyBuilder1,
				},
			},
			wantErrPrefix: "rls: GrpcKeyBuilder in RouteLookupConfig contains repeated key \"k1\" in extra_keys from constant_keys or headers",
		},
		{
			desc: "GrpcKeyBuilder repeated keys across constant_keys and extra_keys",
			cfg: &rlspb.RouteLookupConfig{
				GrpcKeybuilders: []*rlspb.GrpcKeyBuilder{
					{
						Names: []*rlspb.GrpcKeyBuilder_Name{
							{Service: "gBar", Method: "method1"},
							{Service: "gFoobar"},
						},
						Headers:      []*rlspb.NameMatcher{{Key: "k1", Names: []string{"n1", "n2"}}},
						ConstantKeys: map[string]string{"host": "v1"},
						ExtraKeys:    &rlspb.GrpcKeyBuilder_ExtraKeys{Host: "host"},
					},
					goodKeyBuilder1,
				},
			},
			wantErrPrefix: "rls: GrpcKeyBuilder in RouteLookupConfig contains repeated key \"host\" in extra_keys from constant_keys or headers",
		},
		{
			desc: "GrpcKeyBuilder with slash in method name",
			cfg: &rlspb.RouteLookupConfig{
				GrpcKeybuilders: []*rlspb.GrpcKeyBuilder{
					{
						Names:   []*rlspb.GrpcKeyBuilder_Name{{Service: "gBar", Method: "method1/foo"}},
						Headers: []*rlspb.NameMatcher{{Key: "k1", Names: []string{"n1", "n2"}}},
					},
				},
			},
			wantErrPrefix: "rls: GrpcKeyBuilder in RouteLookupConfig contains a method with a slash",
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			builderMap, err := MakeBuilderMap(test.cfg)
			if builderMap != nil || !strings.HasPrefix(fmt.Sprint(err), test.wantErrPrefix) {
				t.Errorf("MakeBuilderMap(%+v) returned {%v, %v}, want: {nil, %v}", test.cfg, builderMap, err, test.wantErrPrefix)
			}
		})
	}
}

func TestRLSKey(t *testing.T) {
	bm, err := MakeBuilderMap(&rlspb.RouteLookupConfig{
		GrpcKeybuilders: []*rlspb.GrpcKeyBuilder{goodKeyBuilder1, goodKeyBuilder2},
	})
	if err != nil {
		t.Fatalf("MakeBuilderMap() failed: %v", err)
	}

	tests := []struct {
		desc   string
		path   string
		md     metadata.MD
		wantKM KeyMap
	}{
		{
			// No keyBuilder is found for the provided service.
			desc:   "service not found in key builder map",
			path:   "/notFoundService/method",
			md:     metadata.Pairs("n1", "v1", "n2", "v2"),
			wantKM: KeyMap{},
		},
		{
			// No keyBuilder is found for the provided method.
			desc:   "method not found in key builder map",
			path:   "/gBar/notFoundMethod",
			md:     metadata.Pairs("n1", "v1", "n2", "v2"),
			wantKM: KeyMap{},
		},
		{
			// A keyBuilder is found, but none of the headers match.
			desc:   "directPathMatch-NoMatchingKey",
			path:   "/gBar/method1",
			md:     metadata.Pairs("notMatchingKey", "v1"),
			wantKM: KeyMap{Map: map[string]string{}, Str: ""},
		},
		{
			// A keyBuilder is found, and a single headers matches.
			desc:   "directPathMatch-SingleKey",
			path:   "/gBar/method1",
			md:     metadata.Pairs("n1", "v1"),
			wantKM: KeyMap{Map: map[string]string{"k1": "v1"}, Str: "k1=v1"},
		},
		{
			// A keyBuilder is found, and multiple headers match, but the first
			// match is chosen.
			desc:   "directPathMatch-FirstMatchingKey",
			path:   "/gBar/method1",
			md:     metadata.Pairs("n2", "v2", "n1", "v1"),
			wantKM: KeyMap{Map: map[string]string{"k1": "v1"}, Str: "k1=v1"},
		},
		{
			// A keyBuilder is found as a wildcard match, but none of the
			// headers match.
			desc:   "wildcardPathMatch-NoMatchingKey",
			path:   "/gFoobar/method1",
			md:     metadata.Pairs("notMatchingKey", "v1"),
			wantKM: KeyMap{Map: map[string]string{}, Str: ""},
		},
		{
			// A keyBuilder is found as a wildcard match, and a single headers
			// matches.
			desc:   "wildcardPathMatch-SingleKey",
			path:   "/gFoobar/method1",
			md:     metadata.Pairs("n1", "v1"),
			wantKM: KeyMap{Map: map[string]string{"k1": "v1"}, Str: "k1=v1"},
		},
		{
			// A keyBuilder is found as a wildcard match, and multiple headers
			// match, but the first match is chosen.
			desc:   "wildcardPathMatch-FirstMatchingKey",
			path:   "/gFoobar/method1",
			md:     metadata.Pairs("n2", "v2", "n1", "v1"),
			wantKM: KeyMap{Map: map[string]string{"k1": "v1"}, Str: "k1=v1"},
		},
		{
			// Multiple headerKeys find hits in the provided request headers.
			desc: "multipleMatchers",
			path: "/gFoo/method1",
			md:   metadata.Pairs("n2", "v2", "n1", "v1"),
			wantKM: KeyMap{
				Map: map[string]string{
					"const-key-1": "const-val-1",
					"const-key-2": "const-val-2",
					"host":        "dummy-host",
					"service":     "gFoo",
					"method":      "method1",
					"k1":          "v1",
					"k2":          "v1",
				},
				Str: "const-key-1=const-val-1,const-key-2=const-val-2,host=dummy-host,k1=v1,k2=v1,method=method1,service=gFoo",
			},
		},
		{
			// A match is found for a header which is specified multiple times.
			// So, the values are joined with commas separating them.
			desc:   "commaSeparated",
			path:   "/gBar/method1",
			md:     metadata.Pairs("n1", "v1", "n1", "v2", "n1", "v3"),
			wantKM: KeyMap{Map: map[string]string{"k1": "v1,v2,v3"}, Str: "k1=v1,v2,v3"},
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			if gotKM := bm.RLSKey(test.md, "dummy-host", test.path); !cmp.Equal(gotKM, test.wantKM) {
				t.Errorf("RLSKey(%+v, %s) = %+v, want %+v", test.md, test.path, gotKM, test.wantKM)
			}
		})
	}
}

func TestMapToString(t *testing.T) {
	tests := []struct {
		desc    string
		input   map[string]string
		wantStr string
	}{
		{
			desc:    "empty map",
			input:   nil,
			wantStr: "",
		},
		{
			desc: "one key",
			input: map[string]string{
				"k1": "v1",
			},
			wantStr: "k1=v1",
		},
		{
			desc: "sorted keys",
			input: map[string]string{
				"k1": "v1",
				"k2": "v2",
				"k3": "v3",
			},
			wantStr: "k1=v1,k2=v2,k3=v3",
		},
		{
			desc: "unsorted keys",
			input: map[string]string{
				"k3": "v3",
				"k1": "v1",
				"k2": "v2",
			},
			wantStr: "k1=v1,k2=v2,k3=v3",
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			if gotStr := mapToString(test.input); gotStr != test.wantStr {
				t.Errorf("mapToString(%v) = %s, want %s", test.input, gotStr, test.wantStr)
			}
		})
	}
}

func TestBuilderMapEqual(t *testing.T) {
	tests := []struct {
		desc      string
		a         BuilderMap
		b         BuilderMap
		wantEqual bool
	}{
		{
			desc:      "nil builder maps",
			a:         nil,
			b:         nil,
			wantEqual: true,
		},
		{
			desc:      "empty builder maps",
			a:         make(map[string]builder),
			b:         make(map[string]builder),
			wantEqual: true,
		},
		{
			desc:      "nil and non-nil builder maps",
			a:         nil,
			b:         map[string]builder{"/gFoo/": {headerKeys: []matcher{{key: "k1", names: []string{"n1"}}}}},
			wantEqual: false,
		},
		{
			desc:      "empty and non-empty builder maps",
			a:         make(map[string]builder),
			b:         map[string]builder{"/gFoo/": {headerKeys: []matcher{{key: "k1", names: []string{"n1"}}}}},
			wantEqual: false,
		},
		{
			desc: "different number of map keys",
			a: map[string]builder{
				"/gFoo/": {headerKeys: []matcher{{key: "k1", names: []string{"n1"}}}},
				"/gBar/": {headerKeys: []matcher{{key: "k1", names: []string{"n1"}}}},
			},
			b: map[string]builder{
				"/gFoo/": {headerKeys: []matcher{{key: "k1", names: []string{"n1"}}}},
			},
			wantEqual: false,
		},
		{
			desc: "different map keys",
			a: map[string]builder{
				"/gBar/": {headerKeys: []matcher{{key: "k1", names: []string{"n1"}}}},
			},
			b: map[string]builder{
				"/gFoo/": {headerKeys: []matcher{{key: "k1", names: []string{"n1"}}}},
			},
			wantEqual: false,
		},
		{
			desc: "equal keys different values",
			a: map[string]builder{
				"/gBar/": {headerKeys: []matcher{{key: "k1", names: []string{"n1"}}}},
				"/gFoo/": {headerKeys: []matcher{{key: "k1", names: []string{"n1", "n2"}}}},
			},
			b: map[string]builder{
				"/gBar/": {headerKeys: []matcher{{key: "k1", names: []string{"n1"}}}},
				"/gFoo/": {headerKeys: []matcher{{key: "k1", names: []string{"n1"}}}},
			},
			wantEqual: false,
		},
		{
			desc: "good match",
			a: map[string]builder{
				"/gBar/": {headerKeys: []matcher{{key: "k1", names: []string{"n1"}}}},
				"/gFoo/": {headerKeys: []matcher{{key: "k1", names: []string{"n1"}}}},
			},
			b: map[string]builder{
				"/gBar/": {headerKeys: []matcher{{key: "k1", names: []string{"n1"}}}},
				"/gFoo/": {headerKeys: []matcher{{key: "k1", names: []string{"n1"}}}},
			},
			wantEqual: true,
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			if gotEqual := test.a.Equal(test.b); gotEqual != test.wantEqual {
				t.Errorf("BuilderMap.Equal(%v, %v) = %v, want %v", test.a, test.b, gotEqual, test.wantEqual)
			}
		})
	}
}

func TestBuilderEqual(t *testing.T) {
	tests := []struct {
		desc      string
		a         builder
		b         builder
		wantEqual bool
	}{
		{
			desc:      "nil builders",
			a:         builder{headerKeys: nil},
			b:         builder{headerKeys: nil},
			wantEqual: true,
		},
		{
			desc:      "empty builders",
			a:         builder{headerKeys: []matcher{}},
			b:         builder{headerKeys: []matcher{}},
			wantEqual: true,
		},
		{
			desc:      "empty and non-empty builders",
			a:         builder{headerKeys: []matcher{}},
			b:         builder{headerKeys: []matcher{{key: "foo"}}},
			wantEqual: false,
		},
		{
			desc:      "different number of headerKeys",
			a:         builder{headerKeys: []matcher{{key: "foo"}, {key: "bar"}}},
			b:         builder{headerKeys: []matcher{{key: "foo"}}},
			wantEqual: false,
		},
		{
			desc:      "equal number but differing headerKeys",
			a:         builder{headerKeys: []matcher{{key: "bar"}}},
			b:         builder{headerKeys: []matcher{{key: "foo"}}},
			wantEqual: false,
		},
		{
			desc:      "different number of constantKeys",
			a:         builder{constantKeys: map[string]string{"k1": "v1"}},
			b:         builder{constantKeys: map[string]string{"k1": "v1", "k2": "v2"}},
			wantEqual: false,
		},
		{
			desc:      "equal number but differing constantKeys",
			a:         builder{constantKeys: map[string]string{"k1": "v1"}},
			b:         builder{constantKeys: map[string]string{"k2": "v2"}},
			wantEqual: false,
		},
		{
			desc:      "different hostKey",
			a:         builder{hostKey: "host1"},
			b:         builder{hostKey: "host2"},
			wantEqual: false,
		},
		{
			desc:      "different serviceKey",
			a:         builder{hostKey: "service1"},
			b:         builder{hostKey: "service2"},
			wantEqual: false,
		},
		{
			desc:      "different methodKey",
			a:         builder{hostKey: "method1"},
			b:         builder{hostKey: "method2"},
			wantEqual: false,
		},
		{
			desc: "equal",
			a: builder{
				headerKeys:   []matcher{{key: "foo"}},
				constantKeys: map[string]string{"k1": "v1"},
				hostKey:      "host",
				serviceKey:   "/service/",
				methodKey:    "method",
			},
			b: builder{
				headerKeys:   []matcher{{key: "foo"}},
				constantKeys: map[string]string{"k1": "v1"},
				hostKey:      "host",
				serviceKey:   "/service/",
				methodKey:    "method",
			},
			wantEqual: true,
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			t.Run(test.desc, func(t *testing.T) {
				if gotEqual := test.a.Equal(test.b); gotEqual != test.wantEqual {
					t.Errorf("builder.Equal(%v, %v) = %v, want %v", test.a, test.b, gotEqual, test.wantEqual)
				}
			})
		})
	}
}

// matcher helps extract a key from request headers based on a given name.
func TestMatcherEqual(t *testing.T) {
	tests := []struct {
		desc      string
		a         matcher
		b         matcher
		wantEqual bool
	}{
		{
			desc:      "different keys",
			a:         matcher{key: "foo"},
			b:         matcher{key: "bar"},
			wantEqual: false,
		},
		{
			desc:      "different number of names",
			a:         matcher{key: "foo", names: []string{"v1", "v2"}},
			b:         matcher{key: "foo", names: []string{"v1"}},
			wantEqual: false,
		},
		{
			desc:      "equal number but differing names",
			a:         matcher{key: "foo", names: []string{"v1", "v2"}},
			b:         matcher{key: "foo", names: []string{"v1", "v22"}},
			wantEqual: false,
		},
		{
			desc:      "same names in different order",
			a:         matcher{key: "foo", names: []string{"v2", "v1"}},
			b:         matcher{key: "foo", names: []string{"v1", "v3"}},
			wantEqual: false,
		},
		{
			desc:      "good match",
			a:         matcher{key: "foo", names: []string{"v1", "v2"}},
			b:         matcher{key: "foo", names: []string{"v1", "v2"}},
			wantEqual: true,
		},
	}

	for _, test := range tests {
		t.Run(test.desc, func(t *testing.T) {
			if gotEqual := test.a.Equal(test.b); gotEqual != test.wantEqual {
				t.Errorf("matcher.Equal(%v, %v) = %v, want %v", test.a, test.b, gotEqual, test.wantEqual)
			}
		})
	}
}
