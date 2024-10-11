package main

import (
	"errors"
	"flag"
	"reflect"
	"testing"
)

func TestParseReqParam(t *testing.T) {

	testcases := []struct {
		name                       string
		expected                   map[string]string
		request                    string
		expectedError              error
		allowDeleteBodyV           bool
		allowMergeV                bool
		allowRepeatedFieldsInBodyV bool
		includePackageInTagsV      bool
		fileV                      string
		importPathV                string
		mergeFileNameV             string
		useFQNForSwaggerNameV      bool
	}{
		{
			// this one must be first - with no leading clearFlags call it
			// verifies our expectation of default values as we reset by
			// clearFlags
			name:             "Test 0",
			expected:         map[string]string{},
			request:          "",
			allowDeleteBodyV: false, allowMergeV: false, allowRepeatedFieldsInBodyV: false, includePackageInTagsV: false,
			fileV: "-", importPathV: "", mergeFileNameV: "apidocs",
		},
		{
			name:             "Test 1",
			expected:         map[string]string{"google/api/annotations.proto": "github.com/grpc-ecosystem/grpc-gateway/third_party/googleapis/google/api"},
			request:          "allow_delete_body,allow_merge,allow_repeated_fields_in_body,include_package_in_tags,file=./foo.pb,import_prefix=/bar/baz,Mgoogle/api/annotations.proto=github.com/grpc-ecosystem/grpc-gateway/third_party/googleapis/google/api",
			allowDeleteBodyV: true, allowMergeV: true, allowRepeatedFieldsInBodyV: true, includePackageInTagsV: true,
			fileV: "./foo.pb", importPathV: "/bar/baz", mergeFileNameV: "apidocs",
		},
		{
			name:             "Test 2",
			expected:         map[string]string{"google/api/annotations.proto": "github.com/grpc-ecosystem/grpc-gateway/third_party/googleapis/google/api"},
			request:          "allow_delete_body=true,allow_merge=true,allow_repeated_fields_in_body=true,include_package_in_tags=true,merge_file_name=test_name,file=./foo.pb,import_prefix=/bar/baz,Mgoogle/api/annotations.proto=github.com/grpc-ecosystem/grpc-gateway/third_party/googleapis/google/api",
			allowDeleteBodyV: true, allowMergeV: true, allowRepeatedFieldsInBodyV: true, includePackageInTagsV: true,
			fileV: "./foo.pb", importPathV: "/bar/baz", mergeFileNameV: "test_name",
		},
		{
			name:             "Test 3",
			expected:         map[string]string{"a/b/c.proto": "github.com/x/y/z", "f/g/h.proto": "github.com/1/2/3/"},
			request:          "allow_delete_body=false,allow_merge=false,Ma/b/c.proto=github.com/x/y/z,Mf/g/h.proto=github.com/1/2/3/",
			allowDeleteBodyV: false, allowMergeV: false, allowRepeatedFieldsInBodyV: false, includePackageInTagsV: false,
			fileV: "stdin", importPathV: "", mergeFileNameV: "apidocs",
		},
		{
			name:             "Test 4",
			expected:         map[string]string{},
			request:          "",
			allowDeleteBodyV: false, allowMergeV: false, allowRepeatedFieldsInBodyV: false, includePackageInTagsV: false,
			fileV: "stdin", importPathV: "", mergeFileNameV: "apidocs",
		},
		{
			name:             "Test 5",
			expected:         map[string]string{},
			request:          "unknown_param=17",
			expectedError:    errors.New("Cannot set flag unknown_param=17: no such flag -unknown_param"),
			allowDeleteBodyV: false, allowMergeV: false, allowRepeatedFieldsInBodyV: false, includePackageInTagsV: false,
			fileV: "stdin", importPathV: "", mergeFileNameV: "apidocs",
		},
		{
			name:             "Test 6",
			expected:         map[string]string{},
			request:          "Mfoo",
			expectedError:    errors.New("Cannot set flag Mfoo: no such flag -Mfoo"),
			allowDeleteBodyV: false, allowMergeV: false, allowRepeatedFieldsInBodyV: false, includePackageInTagsV: false,
			fileV: "stdin", importPathV: "", mergeFileNameV: "apidocs",
		},
		{
			name:             "Test 7",
			expected:         map[string]string{},
			request:          "allow_delete_body,file,import_prefix,allow_merge,allow_repeated_fields_in_body,include_package_in_tags,merge_file_name",
			allowDeleteBodyV: true, allowMergeV: true, allowRepeatedFieldsInBodyV: true, includePackageInTagsV: true,
			fileV: "", importPathV: "", mergeFileNameV: "",
		},
		{
			name:             "Test 8",
			expected:         map[string]string{},
			request:          "allow_delete_body,file,import_prefix,allow_merge,allow_repeated_fields_in_body=3,merge_file_name",
			expectedError:    errors.New(`Cannot set flag allow_repeated_fields_in_body=3: parse error`),
			allowDeleteBodyV: true, allowMergeV: true, allowRepeatedFieldsInBodyV: false, includePackageInTagsV: false,
			fileV: "", importPathV: "", mergeFileNameV: "apidocs",
		},
		{
			name:             "Test 9",
			expected:         map[string]string{},
			request:          "include_package_in_tags=3",
			expectedError:    errors.New(`Cannot set flag include_package_in_tags=3: parse error`),
			allowDeleteBodyV: false, allowMergeV: false, allowRepeatedFieldsInBodyV: false, includePackageInTagsV: false,
			fileV: "stdin", importPathV: "", mergeFileNameV: "apidocs",
		},
		{
			name:             "Test 10",
			expected:         map[string]string{},
			request:          "fqn_for_swagger_name=3",
			expectedError:    errors.New(`Cannot set flag fqn_for_swagger_name=3: parse error`),
			allowDeleteBodyV: false, allowMergeV: false, allowRepeatedFieldsInBodyV: false, includePackageInTagsV: false, useFQNForSwaggerNameV: false,
			fileV: "stdin", importPathV: "", mergeFileNameV: "apidocs",
		},
		{
			name:             "Test 11",
			expected:         map[string]string{},
			request:          "fqn_for_swagger_name=true",
			allowDeleteBodyV: false, allowMergeV: false, allowRepeatedFieldsInBodyV: false, includePackageInTagsV: false, useFQNForSwaggerNameV: true,
			fileV: "stdin", importPathV: "", mergeFileNameV: "apidocs",
		},
	}

	for i, tc := range testcases {
		t.Run(tc.name, func(tt *testing.T) {
			f := flag.CommandLine
			pkgMap := make(map[string]string)
			err := parseReqParam(tc.request, f, pkgMap)
			if tc.expectedError == nil {
				if err != nil {
					tt.Errorf("unexpected parse error '%v'", err)
				}
				if !reflect.DeepEqual(pkgMap, tc.expected) {
					tt.Errorf("pkgMap parse error, expected '%v', got '%v'", tc.expected, pkgMap)
				}
			} else {
				if err == nil {
					tt.Error("expected parse error not returned")
				}
				if !reflect.DeepEqual(pkgMap, tc.expected) {
					tt.Errorf("pkgMap parse error, expected '%v', got '%v'", tc.expected, pkgMap)
				}
				if err.Error() != tc.expectedError.Error() {
					tt.Errorf("expected error malformed, expected %q, got %q", tc.expectedError.Error(), err.Error())
				}
			}
			checkFlags(tc.allowDeleteBodyV, tc.allowMergeV, tc.allowRepeatedFieldsInBodyV, tc.includePackageInTagsV, tc.useFQNForSwaggerNameV, tc.fileV, tc.importPathV, tc.mergeFileNameV, tt, i)

			clearFlags()
		})
	}

}

func checkFlags(allowDeleteV, allowMergeV, allowRepeatedFieldsInBodyV, includePackageInTagsV bool, useFQNForSwaggerNameV bool, fileV, importPathV, mergeFileNameV string, t *testing.T, tid int) {
	if *importPrefix != importPathV {
		t.Errorf("Test %v: import_prefix misparsed, expected '%v', got '%v'", tid, importPathV, *importPrefix)
	}
	if *file != fileV {
		t.Errorf("Test %v: file misparsed, expected '%v', got '%v'", tid, fileV, *file)
	}
	if *allowDeleteBody != allowDeleteV {
		t.Errorf("Test %v: allow_delete_body misparsed, expected '%v', got '%v'", tid, allowDeleteV, *allowDeleteBody)
	}
	if *allowMerge != allowMergeV {
		t.Errorf("Test %v: allow_merge misparsed, expected '%v', got '%v'", tid, allowMergeV, *allowMerge)
	}
	if *mergeFileName != mergeFileNameV {
		t.Errorf("Test %v: merge_file_name misparsed, expected '%v', got '%v'", tid, mergeFileNameV, *mergeFileName)
	}
	if *allowRepeatedFieldsInBody != allowRepeatedFieldsInBodyV {
		t.Errorf("Test %v: allow_repeated_fields_in_body misparsed, expected '%v', got '%v'", tid, allowRepeatedFieldsInBodyV, *allowRepeatedFieldsInBody)
	}
	if *includePackageInTags != includePackageInTagsV {
		t.Errorf("Test %v: include_package_in_tags misparsed, expected '%v', got '%v'", tid, includePackageInTagsV, *includePackageInTags)
	}
	if *useFQNForSwaggerName != useFQNForSwaggerNameV {
		t.Errorf("Test %v: fqn_for_swagger_name misparsed, expected '%v', got '%v'", tid, useFQNForSwaggerNameV, *useFQNForSwaggerName)
	}
}

func clearFlags() {
	*importPrefix = ""
	*file = "stdin"
	*allowDeleteBody = false
	*allowMerge = false
	*allowRepeatedFieldsInBody = false
	*includePackageInTags = false
	*mergeFileName = "apidocs"
}
