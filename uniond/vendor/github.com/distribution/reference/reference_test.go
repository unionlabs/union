package reference

import (
	_ "crypto/sha256"
	_ "crypto/sha512"
	"encoding/json"
	"strings"
	"testing"

	"github.com/opencontainers/go-digest"
)

func TestReferenceParse(t *testing.T) {
	t.Parallel()
	// tests is a unified set of testcases for
	// testing the parsing of references
	tests := []struct {
		// input is the repository name or name component testcase
		input string
		// err is the error expected from Parse, or nil
		err error
		// repository is the string representation for the reference
		repository string
		// domain is the domain expected in the reference
		domain string
		// tag is the tag for the reference
		tag string
		// digest is the digest for the reference (enforces digest reference)
		digest string
	}{
		{
			input:      "test_com",
			repository: "test_com",
		},
		{
			input:      "test.com:tag",
			repository: "test.com",
			tag:        "tag",
		},
		{
			input:      "test.com:5000",
			repository: "test.com",
			tag:        "5000",
		},
		{
			input:      "test.com/repo:tag",
			domain:     "test.com",
			repository: "test.com/repo",
			tag:        "tag",
		},
		{
			input:      "test:5000/repo",
			domain:     "test:5000",
			repository: "test:5000/repo",
		},
		{
			input:      "test:5000/repo:tag",
			domain:     "test:5000",
			repository: "test:5000/repo",
			tag:        "tag",
		},
		{
			input:      "test:5000/repo@sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
			domain:     "test:5000",
			repository: "test:5000/repo",
			digest:     "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
		},
		{
			input:      "test:5000/repo:tag@sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
			domain:     "test:5000",
			repository: "test:5000/repo",
			tag:        "tag",
			digest:     "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
		},
		{
			input:      "test:5000/repo",
			domain:     "test:5000",
			repository: "test:5000/repo",
		},
		{
			input: "",
			err:   ErrNameEmpty,
		},
		{
			input: ":justtag",
			err:   ErrReferenceInvalidFormat,
		},
		{
			input: "@sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
			err:   ErrReferenceInvalidFormat,
		},
		{
			input: "repo@sha256:ffffffffffffffffffffffffffffffffff",
			err:   digest.ErrDigestInvalidLength,
		},
		{
			input: "validname@invaliddigest:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
			err:   digest.ErrDigestUnsupported,
		},
		{
			input: "Uppercase:tag",
			err:   ErrNameContainsUppercase,
		},
		// FIXME "Uppercase" is incorrectly handled as a domain-name here, therefore passes.
		// See https://github.com/distribution/distribution/pull/1778, and https://github.com/docker/docker/pull/20175
		// {
		//	input: "Uppercase/lowercase:tag",
		//	err:   ErrNameContainsUppercase,
		// },
		{
			input: "test:5000/Uppercase/lowercase:tag",
			err:   ErrNameContainsUppercase,
		},
		{
			input:      "lowercase:Uppercase",
			repository: "lowercase",
			tag:        "Uppercase",
		},
		{
			input: strings.Repeat("a/", 128) + "a:tag",
			err:   ErrNameTooLong,
		},
		{
			input:      strings.Repeat("a/", 127) + "a:tag-puts-this-over-max",
			domain:     "a",
			repository: strings.Repeat("a/", 127) + "a",
			tag:        "tag-puts-this-over-max",
		},
		{
			input: "aa/asdf$$^/aa",
			err:   ErrReferenceInvalidFormat,
		},
		{
			input:      "sub-dom1.foo.com/bar/baz/quux",
			domain:     "sub-dom1.foo.com",
			repository: "sub-dom1.foo.com/bar/baz/quux",
		},
		{
			input:      "sub-dom1.foo.com/bar/baz/quux:some-long-tag",
			domain:     "sub-dom1.foo.com",
			repository: "sub-dom1.foo.com/bar/baz/quux",
			tag:        "some-long-tag",
		},
		{
			input:      "b.gcr.io/test.example.com/my-app:test.example.com",
			domain:     "b.gcr.io",
			repository: "b.gcr.io/test.example.com/my-app",
			tag:        "test.example.com",
		},
		{
			input:      "xn--n3h.com/myimage:xn--n3h.com", // ☃.com in punycode
			domain:     "xn--n3h.com",
			repository: "xn--n3h.com/myimage",
			tag:        "xn--n3h.com",
		},
		{
			input:      "xn--7o8h.com/myimage:xn--7o8h.com@sha512:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", // 🐳.com in punycode
			domain:     "xn--7o8h.com",
			repository: "xn--7o8h.com/myimage",
			tag:        "xn--7o8h.com",
			digest:     "sha512:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
		},
		{
			input:      "foo_bar.com:8080",
			repository: "foo_bar.com",
			tag:        "8080",
		},
		{
			input:      "foo/foo_bar.com:8080",
			domain:     "foo",
			repository: "foo/foo_bar.com",
			tag:        "8080",
		},
		{
			input:      "192.168.1.1",
			repository: "192.168.1.1",
		},
		{
			input:      "192.168.1.1:tag",
			repository: "192.168.1.1",
			tag:        "tag",
		},
		{
			input:      "192.168.1.1:5000",
			repository: "192.168.1.1",
			tag:        "5000",
		},
		{
			input:      "192.168.1.1/repo",
			domain:     "192.168.1.1",
			repository: "192.168.1.1/repo",
		},
		{
			input:      "192.168.1.1:5000/repo",
			domain:     "192.168.1.1:5000",
			repository: "192.168.1.1:5000/repo",
		},
		{
			input:      "192.168.1.1:5000/repo:5050",
			domain:     "192.168.1.1:5000",
			repository: "192.168.1.1:5000/repo",
			tag:        "5050",
		},
		{
			input: "[2001:db8::1]",
			err:   ErrReferenceInvalidFormat,
		},
		{
			input: "[2001:db8::1]:5000",
			err:   ErrReferenceInvalidFormat,
		},
		{
			input: "[2001:db8::1]:tag",
			err:   ErrReferenceInvalidFormat,
		},
		{
			input:      "[2001:db8::1]/repo",
			domain:     "[2001:db8::1]",
			repository: "[2001:db8::1]/repo",
		},
		{
			input:      "[2001:db8:1:2:3:4:5:6]/repo:tag",
			domain:     "[2001:db8:1:2:3:4:5:6]",
			repository: "[2001:db8:1:2:3:4:5:6]/repo",
			tag:        "tag",
		},
		{
			input:      "[2001:db8::1]:5000/repo",
			domain:     "[2001:db8::1]:5000",
			repository: "[2001:db8::1]:5000/repo",
		},
		{
			input:      "[2001:db8::1]:5000/repo:tag",
			domain:     "[2001:db8::1]:5000",
			repository: "[2001:db8::1]:5000/repo",
			tag:        "tag",
		},
		{
			input:      "[2001:db8::1]:5000/repo@sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
			domain:     "[2001:db8::1]:5000",
			repository: "[2001:db8::1]:5000/repo",
			digest:     "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
		},
		{
			input:      "[2001:db8::1]:5000/repo:tag@sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
			domain:     "[2001:db8::1]:5000",
			repository: "[2001:db8::1]:5000/repo",
			tag:        "tag",
			digest:     "sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
		},
		{
			input:      "[2001:db8::]:5000/repo",
			domain:     "[2001:db8::]:5000",
			repository: "[2001:db8::]:5000/repo",
		},
		{
			input:      "[::1]:5000/repo",
			domain:     "[::1]:5000",
			repository: "[::1]:5000/repo",
		},
		{
			input: "[fe80::1%eth0]:5000/repo",
			err:   ErrReferenceInvalidFormat,
		},
		{
			input: "[fe80::1%@invalidzone]:5000/repo",
			err:   ErrReferenceInvalidFormat,
		},
	}
	for _, tc := range tests {
		tc := tc
		t.Run(tc.input, func(t *testing.T) {
			t.Parallel()
			repo, err := Parse(tc.input)
			if tc.err != nil {
				if err == nil {
					t.Errorf("missing expected error: %v", tc.err)
				} else if tc.err != err {
					t.Errorf("mismatched error: got %v, expected %v", err, tc.err)
				}
				return
			} else if err != nil {
				t.Errorf("unexpected parse error: %v", err)
				return
			}
			if repo.String() != tc.input {
				t.Errorf("mismatched repo: got %q, expected %q", repo.String(), tc.input)
			}

			if named, ok := repo.(Named); ok {
				if named.Name() != tc.repository {
					t.Errorf("unexpected repository: got %q, expected %q", named.Name(), tc.repository)
				}
				domain, _ := SplitHostname(named)
				if domain != tc.domain {
					t.Errorf("unexpected domain: got %q, expected %q", domain, tc.domain)
				}
			} else if tc.repository != "" || tc.domain != "" {
				t.Errorf("expected named type, got %T", repo)
			}

			tagged, ok := repo.(Tagged)
			if tc.tag != "" {
				if ok {
					if tagged.Tag() != tc.tag {
						t.Errorf("unexpected tag: got %q, expected %q", tagged.Tag(), tc.tag)
					}
				} else {
					t.Errorf("expected tagged type, got %T", repo)
				}
			} else if ok {
				t.Errorf("unexpected tagged type")
			}

			digested, ok := repo.(Digested)
			if tc.digest != "" {
				if ok {
					if digested.Digest().String() != tc.digest {
						t.Errorf("unexpected digest: got %q, expected %q", digested.Digest().String(), tc.digest)
					}
				} else {
					t.Errorf("expected digested type, got %T", repo)
				}
			} else if ok {
				t.Errorf("unexpected digested type")
			}
		})
	}
}

// TestWithNameFailure tests cases where WithName should fail. Cases where it
// should succeed are covered by TestSplitHostname, below.
func TestWithNameFailure(t *testing.T) {
	t.Parallel()
	tests := []struct {
		input string
		err   error
	}{
		{
			input: "",
			err:   ErrNameEmpty,
		},
		{
			input: ":justtag",
			err:   ErrReferenceInvalidFormat,
		},
		{
			input: "@sha256:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
			err:   ErrReferenceInvalidFormat,
		},
		{
			input: "validname@invaliddigest:ffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff",
			err:   ErrReferenceInvalidFormat,
		},
		{
			input: strings.Repeat("a/", 128) + "a:tag",
			err:   ErrNameTooLong,
		},
		{
			input: "aa/asdf$$^/aa",
			err:   ErrReferenceInvalidFormat,
		},
	}
	for _, tc := range tests {
		tc := tc
		t.Run(tc.input, func(t *testing.T) {
			t.Parallel()
			_, err := WithName(tc.input)
			if err == nil {
				t.Errorf("no error parsing name. expected: %s", tc.err)
			}
		})
	}
}

func TestSplitHostname(t *testing.T) {
	t.Parallel()
	tests := []struct {
		input  string
		domain string
		name   string
	}{
		{
			input:  "test.com/foo",
			domain: "test.com",
			name:   "foo",
		},
		{
			input:  "test_com/foo",
			domain: "",
			name:   "test_com/foo",
		},
		{
			input:  "test:8080/foo",
			domain: "test:8080",
			name:   "foo",
		},
		{
			input:  "test.com:8080/foo",
			domain: "test.com:8080",
			name:   "foo",
		},
		{
			input:  "test-com:8080/foo",
			domain: "test-com:8080",
			name:   "foo",
		},
		{
			input:  "xn--n3h.com:18080/foo",
			domain: "xn--n3h.com:18080",
			name:   "foo",
		},
	}
	for _, tc := range tests {
		tc := tc
		t.Run(tc.input, func(t *testing.T) {
			t.Parallel()
			named, err := WithName(tc.input)
			if err != nil {
				t.Errorf("error parsing name: %s", err)
			}
			domain, name := SplitHostname(named)
			if domain != tc.domain {
				t.Errorf("unexpected domain: got %q, expected %q", domain, tc.domain)
			}
			if name != tc.name {
				t.Errorf("unexpected name: got %q, expected %q", name, tc.name)
			}
		})
	}
}

type serializationType struct {
	Description string
	Field       Field
}

func TestSerialization(t *testing.T) {
	t.Parallel()
	tests := []struct {
		description string
		input       string
		name        string
		tag         string
		digest      string
		err         error
	}{
		{
			description: "empty value",
			err:         ErrNameEmpty,
		},
		{
			description: "just a name",
			input:       "example.com:8000/named",
			name:        "example.com:8000/named",
		},
		{
			description: "name with a tag",
			input:       "example.com:8000/named:tagged",
			name:        "example.com:8000/named",
			tag:         "tagged",
		},
		{
			description: "name with digest",
			input:       "other.com/named@sha256:1234567890098765432112345667890098765432112345667890098765432112",
			name:        "other.com/named",
			digest:      "sha256:1234567890098765432112345667890098765432112345667890098765432112",
		},
	}
	for _, tc := range tests {
		tc := tc
		t.Run(tc.description, func(t *testing.T) {
			t.Parallel()
			m := map[string]string{
				"Description": tc.description,
				"Field":       tc.input,
			}
			b, err := json.Marshal(m)
			if err != nil {
				t.Errorf("error marshalling: %v", err)
			}
			st := serializationType{}

			if err := json.Unmarshal(b, &st); err != nil {
				if tc.err == nil {
					t.Errorf("error unmarshalling: %v", err)
				}
				if err != tc.err {
					t.Errorf("wrong error, expected %v, got %v", tc.err, err)
				}

				return
			} else if tc.err != nil {
				t.Errorf("expected error unmarshalling: %v", tc.err)
			}

			if st.Description != tc.description {
				t.Errorf("wrong description, expected %q, got %q", tc.description, st.Description)
			}

			ref := st.Field.Reference()

			if named, ok := ref.(Named); ok {
				if named.Name() != tc.name {
					t.Errorf("unexpected repository: got %q, expected %q", named.Name(), tc.name)
				}
			} else if tc.name != "" {
				t.Errorf("expected named type, got %T", ref)
			}

			tagged, ok := ref.(Tagged)
			if tc.tag != "" {
				if ok {
					if tagged.Tag() != tc.tag {
						t.Errorf("unexpected tag: got %q, expected %q", tagged.Tag(), tc.tag)
					}
				} else {
					t.Errorf("expected tagged type, got %T", ref)
				}
			} else if ok {
				t.Errorf("unexpected tagged type")
			}

			digested, ok := ref.(Digested)
			if tc.digest != "" {
				if ok {
					if digested.Digest().String() != tc.digest {
						t.Errorf("unexpected digest: got %q, expected %q", digested.Digest().String(), tc.digest)
					}
				} else {
					t.Errorf("expected digested type, got %T", ref)
				}
			} else if ok {
				t.Errorf("unexpected digested type")
			}

			st = serializationType{
				Description: tc.description,
				Field:       AsField(ref),
			}

			b2, err := json.Marshal(st)
			if err != nil {
				t.Errorf("error marshing serialization type: %v", err)
			}

			if string(b) != string(b2) {
				t.Errorf("unexpected serialized value: expected %q, got %q", string(b), string(b2))
			}

			// Ensure st.Field is not implementing "Reference" directly, getting
			// around the Reference type system
			var fieldInterface interface{} = st.Field
			if _, ok := fieldInterface.(Reference); ok {
				t.Errorf("field should not implement Reference interface")
			}
		})
	}
}

func TestWithTag(t *testing.T) {
	t.Parallel()
	tests := []struct {
		name     string
		digest   digest.Digest
		tag      string
		combined string
	}{
		{
			name:     "test.com/foo",
			tag:      "tag",
			combined: "test.com/foo:tag",
		},
		{
			name:     "foo",
			tag:      "tag2",
			combined: "foo:tag2",
		},
		{
			name:     "test.com:8000/foo",
			tag:      "tag4",
			combined: "test.com:8000/foo:tag4",
		},
		{
			name:     "test.com:8000/foo",
			tag:      "TAG5",
			combined: "test.com:8000/foo:TAG5",
		},
		{
			name:     "test.com:8000/foo",
			digest:   "sha256:1234567890098765432112345667890098765",
			tag:      "TAG5",
			combined: "test.com:8000/foo:TAG5@sha256:1234567890098765432112345667890098765",
		},
	}
	for _, tc := range tests {
		tc := tc
		t.Run(tc.combined, func(t *testing.T) {
			t.Parallel()
			named, err := WithName(tc.name)
			if err != nil {
				t.Errorf("error parsing name: %s", err)
			}
			if tc.digest != "" {
				canonical, err := WithDigest(named, tc.digest)
				if err != nil {
					t.Errorf("error adding digest")
				}
				named = canonical
			}

			tagged, err := WithTag(named, tc.tag)
			if err != nil {
				t.Errorf("WithTag failed: %s", err)
			}
			if tagged.String() != tc.combined {
				t.Errorf("unexpected: got %q, expected %q", tagged.String(), tc.combined)
			}
		})
	}
}

func TestWithDigest(t *testing.T) {
	t.Parallel()
	tests := []struct {
		name     string
		digest   digest.Digest
		tag      string
		combined string
	}{
		{
			name:     "test.com/foo",
			digest:   "sha256:1234567890098765432112345667890098765",
			combined: "test.com/foo@sha256:1234567890098765432112345667890098765",
		},
		{
			name:     "foo",
			digest:   "sha256:1234567890098765432112345667890098765",
			combined: "foo@sha256:1234567890098765432112345667890098765",
		},
		{
			name:     "test.com:8000/foo",
			digest:   "sha256:1234567890098765432112345667890098765",
			combined: "test.com:8000/foo@sha256:1234567890098765432112345667890098765",
		},
		{
			name:     "test.com:8000/foo",
			digest:   "sha256:1234567890098765432112345667890098765",
			tag:      "latest",
			combined: "test.com:8000/foo:latest@sha256:1234567890098765432112345667890098765",
		},
	}
	for _, tc := range tests {
		tc := tc
		t.Run(tc.combined, func(t *testing.T) {
			t.Parallel()
			named, err := WithName(tc.name)
			if err != nil {
				t.Errorf("error parsing name: %s", err)
			}
			if tc.tag != "" {
				tagged, err := WithTag(named, tc.tag)
				if err != nil {
					t.Errorf("error adding tag")
				}
				named = tagged
			}
			digested, err := WithDigest(named, tc.digest)
			if err != nil {
				t.Errorf("WithDigest failed: %s", err)
			}
			if digested.String() != tc.combined {
				t.Errorf("unexpected: got %q, expected %q", digested.String(), tc.combined)
			}
		})
	}
}

func TestParseNamed(t *testing.T) {
	t.Parallel()
	tests := []struct {
		input  string
		domain string
		name   string
		err    error
	}{
		{
			input:  "test.com/foo",
			domain: "test.com",
			name:   "foo",
		},
		{
			input:  "test:8080/foo",
			domain: "test:8080",
			name:   "foo",
		},
		{
			input: "test_com/foo",
			err:   ErrNameNotCanonical,
		},
		{
			input: "test.com",
			err:   ErrNameNotCanonical,
		},
		{
			input: "foo",
			err:   ErrNameNotCanonical,
		},
		{
			input: "library/foo",
			err:   ErrNameNotCanonical,
		},
		{
			input:  "docker.io/library/foo",
			domain: "docker.io",
			name:   "library/foo",
		},
		// Ambiguous case, parser will add "library/" to foo
		{
			input: "docker.io/foo",
			err:   ErrNameNotCanonical,
		},
	}
	for _, tc := range tests {
		tc := tc
		t.Run(tc.input, func(t *testing.T) {
			t.Parallel()
			named, err := ParseNamed(tc.input)
			if err != nil && tc.err == nil {
				t.Errorf("error parsing name: %s", err)
				return
			} else if err == nil && tc.err != nil {
				t.Errorf("parsing succeeded: expected error %v", tc.err)
				return
			} else if err != tc.err {
				t.Errorf("unexpected error %v, expected %v", err, tc.err)
				return
			} else if err != nil {
				return
			}

			domain, name := SplitHostname(named)
			if domain != tc.domain {
				t.Errorf("unexpected domain: got %q, expected %q", domain, tc.domain)
			}
			if name != tc.name {
				t.Errorf("unexpected name: got %q, expected %q", name, tc.name)
			}
		})
	}
}
