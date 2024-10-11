package reference

import (
	"strconv"
	"testing"

	"github.com/opencontainers/go-digest"
)

func TestValidateReferenceName(t *testing.T) {
	t.Parallel()
	validRepoNames := []string{
		"docker/docker",
		"library/debian",
		"debian",
		"docker.io/docker/docker",
		"docker.io/library/debian",
		"docker.io/debian",
		"index.docker.io/docker/docker",
		"index.docker.io/library/debian",
		"index.docker.io/debian",
		"127.0.0.1:5000/docker/docker",
		"127.0.0.1:5000/library/debian",
		"127.0.0.1:5000/debian",
		"192.168.0.1",
		"192.168.0.1:80",
		"192.168.0.1:8/debian",
		"192.168.0.2:25000/debian",
		"thisisthesongthatneverendsitgoesonandonandonthisisthesongthatnev",
		"[fc00::1]:5000/docker",
		"[fc00::1]:5000/docker/docker",
		"[fc00:1:2:3:4:5:6:7]:5000/library/debian",

		// This test case was moved from invalid to valid since it is valid input
		// when specified with a hostname, it removes the ambiguity from about
		// whether the value is an identifier or repository name
		"docker.io/1a3f5e7d9c1b3a5f7e9d1c3b5a7f9e1d3c5b7a9f1e3d5d7c9b1a3f5e7d9c1b3a",
		"Docker/docker",
		"DOCKER/docker",
	}
	invalidRepoNames := []string{
		"https://github.com/docker/docker",
		"docker/Docker",
		"-docker",
		"-docker/docker",
		"-docker.io/docker/docker",
		"docker///docker",
		"docker.io/docker/Docker",
		"docker.io/docker///docker",
		"[fc00::1]",
		"[fc00::1]:5000",
		"fc00::1:5000/debian",
		"[fe80::1%eth0]:5000/debian",
		"[2001:db8:3:4::192.0.2.33]:5000/debian",
		"1a3f5e7d9c1b3a5f7e9d1c3b5a7f9e1d3c5b7a9f1e3d5d7c9b1a3f5e7d9c1b3a",
	}

	for _, name := range invalidRepoNames {
		_, err := ParseNormalizedNamed(name)
		if err == nil {
			t.Fatalf("Expected invalid repo name for %q", name)
		}
	}

	for _, name := range validRepoNames {
		_, err := ParseNormalizedNamed(name)
		if err != nil {
			t.Fatalf("Error parsing repo name %s, got: %q", name, err)
		}
	}
}

func TestValidateRemoteName(t *testing.T) {
	t.Parallel()
	validRepositoryNames := []string{
		// Sanity check.
		"docker/docker",

		// Allow 64-character non-hexadecimal names (hexadecimal names are forbidden).
		"thisisthesongthatneverendsitgoesonandonandonthisisthesongthatnev",

		// Allow embedded hyphens.
		"docker-rules/docker",

		// Allow multiple hyphens as well.
		"docker---rules/docker",

		// Username doc and image name docker being tested.
		"doc/docker",

		// single character names are now allowed.
		"d/docker",
		"jess/t",

		// Consecutive underscores.
		"dock__er/docker",
	}
	for _, repositoryName := range validRepositoryNames {
		_, err := ParseNormalizedNamed(repositoryName)
		if err != nil {
			t.Errorf("Repository name should be valid: %v. Error: %v", repositoryName, err)
		}
	}

	invalidRepositoryNames := []string{
		// Disallow capital letters.
		"docker/Docker",

		// Only allow one slash.
		"docker///docker",

		// Disallow 64-character hexadecimal.
		"1a3f5e7d9c1b3a5f7e9d1c3b5a7f9e1d3c5b7a9f1e3d5d7c9b1a3f5e7d9c1b3a",

		// Disallow leading and trailing hyphens in namespace.
		"-docker/docker",
		"docker-/docker",
		"-docker-/docker",

		// Don't allow underscores everywhere (as opposed to hyphens).
		"____/____",

		"_docker/_docker",

		// Disallow consecutive periods.
		"dock..er/docker",
		"dock_.er/docker",
		"dock-.er/docker",

		// No repository.
		"docker/",

		// namespace too long
		"this_is_not_a_valid_namespace_because_its_lenth_is_greater_than_255_this_is_not_a_valid_namespace_because_its_lenth_is_greater_than_255_this_is_not_a_valid_namespace_because_its_lenth_is_greater_than_255_this_is_not_a_valid_namespace_because_its_lenth_is_greater_than_255/docker",
	}
	for _, repositoryName := range invalidRepositoryNames {
		if _, err := ParseNormalizedNamed(repositoryName); err == nil {
			t.Errorf("Repository name should be invalid: %v", repositoryName)
		}
	}
}

func TestParseRepositoryInfo(t *testing.T) {
	t.Parallel()
	type tcase struct {
		RemoteName, FamiliarName, FullName, AmbiguousName, Domain string
	}

	tests := []tcase{
		{
			RemoteName:    "fooo/bar",
			FamiliarName:  "fooo/bar",
			FullName:      "docker.io/fooo/bar",
			AmbiguousName: "index.docker.io/fooo/bar",
			Domain:        "docker.io",
		},
		{
			RemoteName:    "library/ubuntu",
			FamiliarName:  "ubuntu",
			FullName:      "docker.io/library/ubuntu",
			AmbiguousName: "library/ubuntu",
			Domain:        "docker.io",
		},
		{
			RemoteName:    "nonlibrary/ubuntu",
			FamiliarName:  "nonlibrary/ubuntu",
			FullName:      "docker.io/nonlibrary/ubuntu",
			AmbiguousName: "",
			Domain:        "docker.io",
		},
		{
			RemoteName:    "other/library",
			FamiliarName:  "other/library",
			FullName:      "docker.io/other/library",
			AmbiguousName: "",
			Domain:        "docker.io",
		},
		{
			RemoteName:    "private/moonbase",
			FamiliarName:  "127.0.0.1:8000/private/moonbase",
			FullName:      "127.0.0.1:8000/private/moonbase",
			AmbiguousName: "",
			Domain:        "127.0.0.1:8000",
		},
		{
			RemoteName:    "privatebase",
			FamiliarName:  "127.0.0.1:8000/privatebase",
			FullName:      "127.0.0.1:8000/privatebase",
			AmbiguousName: "",
			Domain:        "127.0.0.1:8000",
		},
		{
			RemoteName:    "private/moonbase",
			FamiliarName:  "example.com/private/moonbase",
			FullName:      "example.com/private/moonbase",
			AmbiguousName: "",
			Domain:        "example.com",
		},
		{
			RemoteName:    "privatebase",
			FamiliarName:  "example.com/privatebase",
			FullName:      "example.com/privatebase",
			AmbiguousName: "",
			Domain:        "example.com",
		},
		{
			RemoteName:    "private/moonbase",
			FamiliarName:  "example.com:8000/private/moonbase",
			FullName:      "example.com:8000/private/moonbase",
			AmbiguousName: "",
			Domain:        "example.com:8000",
		},
		{
			RemoteName:    "privatebasee",
			FamiliarName:  "example.com:8000/privatebasee",
			FullName:      "example.com:8000/privatebasee",
			AmbiguousName: "",
			Domain:        "example.com:8000",
		},
		{
			RemoteName:    "library/ubuntu-12.04-base",
			FamiliarName:  "ubuntu-12.04-base",
			FullName:      "docker.io/library/ubuntu-12.04-base",
			AmbiguousName: "index.docker.io/library/ubuntu-12.04-base",
			Domain:        "docker.io",
		},
		{
			RemoteName:    "library/foo",
			FamiliarName:  "foo",
			FullName:      "docker.io/library/foo",
			AmbiguousName: "docker.io/foo",
			Domain:        "docker.io",
		},
		{
			RemoteName:    "library/foo/bar",
			FamiliarName:  "library/foo/bar",
			FullName:      "docker.io/library/foo/bar",
			AmbiguousName: "",
			Domain:        "docker.io",
		},
		{
			RemoteName:    "store/foo/bar",
			FamiliarName:  "store/foo/bar",
			FullName:      "docker.io/store/foo/bar",
			AmbiguousName: "",
			Domain:        "docker.io",
		},
		{
			RemoteName:    "bar",
			FamiliarName:  "Foo/bar",
			FullName:      "Foo/bar",
			AmbiguousName: "",
			Domain:        "Foo",
		},
		{
			RemoteName:    "bar",
			FamiliarName:  "FOO/bar",
			FullName:      "FOO/bar",
			AmbiguousName: "",
			Domain:        "FOO",
		},
	}

	for i, tc := range tests {
		tc := tc
		refStrings := []string{tc.FamiliarName, tc.FullName}
		if tc.AmbiguousName != "" {
			refStrings = append(refStrings, tc.AmbiguousName)
		}

		for _, r := range refStrings {
			r := r
			t.Run(strconv.Itoa(i)+"/"+r, func(t *testing.T) {
				t.Parallel()
				named, err := ParseNormalizedNamed(r)
				if err != nil {
					t.Fatalf("ref=%s: %v", r, err)
				}
				t.Run("FamiliarName", func(t *testing.T) {
					if expected, actual := tc.FamiliarName, FamiliarName(named); expected != actual {
						t.Errorf("Invalid familiar name for %q. Expected %q, got %q", named, expected, actual)
					}
				})
				t.Run("FullName", func(t *testing.T) {
					if expected, actual := tc.FullName, named.String(); expected != actual {
						t.Errorf("Invalid canonical reference for %q. Expected %q, got %q", named, expected, actual)
					}
				})
				t.Run("Domain", func(t *testing.T) {
					if expected, actual := tc.Domain, Domain(named); expected != actual {
						t.Errorf("Invalid domain for %q. Expected %q, got %q", named, expected, actual)
					}
				})
				t.Run("RemoteName", func(t *testing.T) {
					if expected, actual := tc.RemoteName, Path(named); expected != actual {
						t.Errorf("Invalid remoteName for %q. Expected %q, got %q", named, expected, actual)
					}
				})
			})
		}
	}
}

func TestParseReferenceWithTagAndDigest(t *testing.T) {
	t.Parallel()
	shortRef := "busybox:latest@sha256:86e0e091d0da6bde2456dbb48306f3956bbeb2eae1b5b9a43045843f69fe4aaa"
	ref, err := ParseNormalizedNamed(shortRef)
	if err != nil {
		t.Fatal(err)
	}
	if expected, actual := "docker.io/library/"+shortRef, ref.String(); actual != expected {
		t.Fatalf("Invalid parsed reference for %q: expected %q, got %q", ref, expected, actual)
	}

	if _, isTagged := ref.(NamedTagged); !isTagged {
		t.Fatalf("Reference from %q should support tag", ref)
	}
	if _, isCanonical := ref.(Canonical); !isCanonical {
		t.Fatalf("Reference from %q should support digest", ref)
	}
	if expected, actual := shortRef, FamiliarString(ref); actual != expected {
		t.Fatalf("Invalid parsed reference for %q: expected %q, got %q", ref, expected, actual)
	}
}

func TestInvalidReferenceComponents(t *testing.T) {
	t.Parallel()
	if _, err := ParseNormalizedNamed("-foo"); err == nil {
		t.Fatal("Expected WithName to detect invalid name")
	}
	ref, err := ParseNormalizedNamed("busybox")
	if err != nil {
		t.Fatal(err)
	}
	if _, err := WithTag(ref, "-foo"); err == nil {
		t.Fatal("Expected WithName to detect invalid tag")
	}
	if _, err := WithDigest(ref, digest.Digest("foo")); err == nil {
		t.Fatal("Expected WithDigest to detect invalid digest")
	}
}

func equalReference(r1, r2 Reference) bool {
	switch v1 := r1.(type) {
	case digestReference:
		if v2, ok := r2.(digestReference); ok {
			return v1 == v2
		}
	case repository:
		if v2, ok := r2.(repository); ok {
			return v1 == v2
		}
	case taggedReference:
		if v2, ok := r2.(taggedReference); ok {
			return v1 == v2
		}
	case canonicalReference:
		if v2, ok := r2.(canonicalReference); ok {
			return v1 == v2
		}
	case reference:
		if v2, ok := r2.(reference); ok {
			return v1 == v2
		}
	}
	return false
}

func TestParseAnyReference(t *testing.T) {
	t.Parallel()
	tests := []struct {
		Reference  string
		Equivalent string
		Expected   Reference
	}{
		{
			Reference:  "redis",
			Equivalent: "docker.io/library/redis",
		},
		{
			Reference:  "redis:latest",
			Equivalent: "docker.io/library/redis:latest",
		},
		{
			Reference:  "docker.io/library/redis:latest",
			Equivalent: "docker.io/library/redis:latest",
		},
		{
			Reference:  "redis@sha256:dbcc1c35ac38df41fd2f5e4130b32ffdb93ebae8b3dbe638c23575912276fc9c",
			Equivalent: "docker.io/library/redis@sha256:dbcc1c35ac38df41fd2f5e4130b32ffdb93ebae8b3dbe638c23575912276fc9c",
		},
		{
			Reference:  "docker.io/library/redis@sha256:dbcc1c35ac38df41fd2f5e4130b32ffdb93ebae8b3dbe638c23575912276fc9c",
			Equivalent: "docker.io/library/redis@sha256:dbcc1c35ac38df41fd2f5e4130b32ffdb93ebae8b3dbe638c23575912276fc9c",
		},
		{
			Reference:  "dmcgowan/myapp",
			Equivalent: "docker.io/dmcgowan/myapp",
		},
		{
			Reference:  "dmcgowan/myapp:latest",
			Equivalent: "docker.io/dmcgowan/myapp:latest",
		},
		{
			Reference:  "docker.io/mcgowan/myapp:latest",
			Equivalent: "docker.io/mcgowan/myapp:latest",
		},
		{
			Reference:  "dmcgowan/myapp@sha256:dbcc1c35ac38df41fd2f5e4130b32ffdb93ebae8b3dbe638c23575912276fc9c",
			Equivalent: "docker.io/dmcgowan/myapp@sha256:dbcc1c35ac38df41fd2f5e4130b32ffdb93ebae8b3dbe638c23575912276fc9c",
		},
		{
			Reference:  "docker.io/dmcgowan/myapp@sha256:dbcc1c35ac38df41fd2f5e4130b32ffdb93ebae8b3dbe638c23575912276fc9c",
			Equivalent: "docker.io/dmcgowan/myapp@sha256:dbcc1c35ac38df41fd2f5e4130b32ffdb93ebae8b3dbe638c23575912276fc9c",
		},
		{
			Reference:  "dbcc1c35ac38df41fd2f5e4130b32ffdb93ebae8b3dbe638c23575912276fc9c",
			Expected:   digestReference("sha256:dbcc1c35ac38df41fd2f5e4130b32ffdb93ebae8b3dbe638c23575912276fc9c"),
			Equivalent: "sha256:dbcc1c35ac38df41fd2f5e4130b32ffdb93ebae8b3dbe638c23575912276fc9c",
		},
		{
			Reference:  "sha256:dbcc1c35ac38df41fd2f5e4130b32ffdb93ebae8b3dbe638c23575912276fc9c",
			Expected:   digestReference("sha256:dbcc1c35ac38df41fd2f5e4130b32ffdb93ebae8b3dbe638c23575912276fc9c"),
			Equivalent: "sha256:dbcc1c35ac38df41fd2f5e4130b32ffdb93ebae8b3dbe638c23575912276fc9c",
		},
		{
			Reference:  "dbcc1c35ac38df41fd2f5e4130b32ffdb93ebae8b3dbe638c23575912276fc9",
			Equivalent: "docker.io/library/dbcc1c35ac38df41fd2f5e4130b32ffdb93ebae8b3dbe638c23575912276fc9",
		},
		{
			Reference:  "dbcc1",
			Equivalent: "docker.io/library/dbcc1",
		},
	}

	for _, tc := range tests {
		tc := tc
		t.Run(tc.Reference, func(t *testing.T) {
			t.Parallel()
			var ref Reference
			var err error
			ref, err = ParseAnyReference(tc.Reference)
			if err != nil {
				t.Fatalf("Error parsing reference %s: %v", tc.Reference, err)
			}
			if ref.String() != tc.Equivalent {
				t.Fatalf("Unexpected string: %s, expected %s", ref.String(), tc.Equivalent)
			}

			expected := tc.Expected
			if expected == nil {
				expected, err = Parse(tc.Equivalent)
				if err != nil {
					t.Fatalf("Error parsing reference %s: %v", tc.Equivalent, err)
				}
			}
			if !equalReference(ref, expected) {
				t.Errorf("Unexpected reference %#v, expected %#v", ref, expected)
			}
		})
	}
}

func TestNormalizedSplitHostname(t *testing.T) {
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
			domain: "docker.io",
			name:   "test_com/foo",
		},
		{
			input:  "docker/migrator",
			domain: "docker.io",
			name:   "docker/migrator",
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
			input:  "foo",
			domain: "docker.io",
			name:   "library/foo",
		},
		{
			input:  "xn--n3h.com/foo",
			domain: "xn--n3h.com",
			name:   "foo",
		},
		{
			input:  "xn--n3h.com:18080/foo",
			domain: "xn--n3h.com:18080",
			name:   "foo",
		},
		{
			input:  "docker.io/foo",
			domain: "docker.io",
			name:   "library/foo",
		},
		{
			input:  "docker.io/library/foo",
			domain: "docker.io",
			name:   "library/foo",
		},
		{
			input:  "docker.io/library/foo/bar",
			domain: "docker.io",
			name:   "library/foo/bar",
		},
	}
	for _, tc := range tests {
		tc := tc
		t.Run(tc.input, func(t *testing.T) {
			t.Parallel()
			named, err := ParseNormalizedNamed(tc.input)
			if err != nil {
				t.Errorf("error parsing name: %s", err)
			}
			domain, name := SplitHostname(named) //nolint:staticcheck // Ignore SA1019: SplitHostname is deprecated.
			if domain != tc.domain {
				t.Errorf("unexpected domain: got %q, expected %q", domain, tc.domain)
			}
			if name != tc.name {
				t.Errorf("unexpected name: got %q, expected %q", name, tc.name)
			}
		})
	}
}

func TestMatchError(t *testing.T) {
	t.Parallel()
	named, err := ParseAnyReference("foo")
	if err != nil {
		t.Fatal(err)
	}
	_, err = FamiliarMatch("[-x]", named)
	if err == nil {
		t.Fatalf("expected an error, got nothing")
	}
}

func TestMatch(t *testing.T) {
	t.Parallel()
	tests := []struct {
		reference string
		pattern   string
		expected  bool
	}{
		{
			reference: "foo",
			pattern:   "foo/**/ba[rz]",
			expected:  false,
		},
		{
			reference: "foo/any/bat",
			pattern:   "foo/**/ba[rz]",
			expected:  false,
		},
		{
			reference: "foo/a/bar",
			pattern:   "foo/**/ba[rz]",
			expected:  true,
		},
		{
			reference: "foo/b/baz",
			pattern:   "foo/**/ba[rz]",
			expected:  true,
		},
		{
			reference: "foo/c/baz:tag",
			pattern:   "foo/**/ba[rz]",
			expected:  true,
		},
		{
			reference: "foo/c/baz:tag",
			pattern:   "foo/*/baz:tag",
			expected:  true,
		},
		{
			reference: "foo/c/baz:tag",
			pattern:   "foo/c/baz:tag",
			expected:  true,
		},
		{
			reference: "example.com/foo/c/baz:tag",
			pattern:   "*/foo/c/baz",
			expected:  true,
		},
		{
			reference: "example.com/foo/c/baz:tag",
			pattern:   "example.com/foo/c/baz",
			expected:  true,
		},
	}
	for _, tc := range tests {
		tc := tc
		t.Run(tc.reference, func(t *testing.T) {
			t.Parallel()
			named, err := ParseAnyReference(tc.reference)
			if err != nil {
				t.Fatal(err)
			}
			actual, err := FamiliarMatch(tc.pattern, named)
			if err != nil {
				t.Fatal(err)
			}
			if actual != tc.expected {
				t.Fatalf("expected %s match %s to be %v, was %v", tc.reference, tc.pattern, tc.expected, actual)
			}
		})
	}
}

func TestParseDockerRef(t *testing.T) {
	t.Parallel()
	tests := []struct {
		name     string
		input    string
		expected string
	}{
		{
			name:     "nothing",
			input:    "busybox",
			expected: "docker.io/library/busybox:latest",
		},
		{
			name:     "tag only",
			input:    "busybox:latest",
			expected: "docker.io/library/busybox:latest",
		},
		{
			name:     "digest only",
			input:    "busybox@sha256:e6693c20186f837fc393390135d8a598a96a833917917789d63766cab6c59582",
			expected: "docker.io/library/busybox@sha256:e6693c20186f837fc393390135d8a598a96a833917917789d63766cab6c59582",
		},
		{
			name:     "path only",
			input:    "library/busybox",
			expected: "docker.io/library/busybox:latest",
		},
		{
			name:     "hostname only",
			input:    "docker.io/busybox",
			expected: "docker.io/library/busybox:latest",
		},
		{
			name:     "no tag",
			input:    "docker.io/library/busybox",
			expected: "docker.io/library/busybox:latest",
		},
		{
			name:     "no path",
			input:    "docker.io/busybox:latest",
			expected: "docker.io/library/busybox:latest",
		},
		{
			name:     "no hostname",
			input:    "library/busybox:latest",
			expected: "docker.io/library/busybox:latest",
		},
		{
			name:     "full reference with tag",
			input:    "docker.io/library/busybox:latest",
			expected: "docker.io/library/busybox:latest",
		},
		{
			name:     "gcr reference without tag",
			input:    "gcr.io/library/busybox",
			expected: "gcr.io/library/busybox:latest",
		},
		{
			name:     "both tag and digest",
			input:    "gcr.io/library/busybox:latest@sha256:e6693c20186f837fc393390135d8a598a96a833917917789d63766cab6c59582",
			expected: "gcr.io/library/busybox@sha256:e6693c20186f837fc393390135d8a598a96a833917917789d63766cab6c59582",
		},
	}
	for _, tc := range tests {
		tc := tc
		t.Run(tc.name, func(t *testing.T) {
			t.Parallel()
			normalized, err := ParseDockerRef(tc.input)
			if err != nil {
				t.Fatal(err)
			}
			output := normalized.String()
			if output != tc.expected {
				t.Fatalf("expected %q to be parsed as %v, got %v", tc.input, tc.expected, output)
			}
			_, err = Parse(output)
			if err != nil {
				t.Fatalf("%q should be a valid reference, but got an error: %v", output, err)
			}
		})
	}
}
