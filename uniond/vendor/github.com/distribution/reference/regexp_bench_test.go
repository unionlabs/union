package reference

import (
	"strings"
	"testing"
)

func BenchmarkParse(b *testing.B) {
	tests := []regexpMatch{
		{
			input: "",
			match: false,
		},
		{
			input: "short",
			match: true,
		},
		{
			input: "simple/name",
			match: true,
		},
		{
			input: "library/ubuntu",
			match: true,
		},
		{
			input: "docker/stevvooe/app",
			match: true,
		},
		{
			input: "aa/aa/aa/aa/aa/aa/aa/aa/aa/bb/bb/bb/bb/bb/bb",
			match: true,
		},
		{
			input: "aa/aa/bb/bb/bb",
			match: true,
		},
		{
			input: "a/a/a/a",
			match: true,
		},
		{
			input: "a/a/a/a/",
			match: false,
		},
		{
			input: "a//a/a",
			match: false,
		},
		{
			input: "a",
			match: true,
		},
		{
			input: "a/aa",
			match: true,
		},
		{
			input: "a/aa/a",
			match: true,
		},
		{
			input: "foo.com",
			match: true,
		},
		{
			input: "foo.com/",
			match: false,
		},
		{
			input: "foo.com:8080/bar",
			match: true,
		},
		{
			input: "foo.com:http/bar",
			match: false,
		},
		{
			input: "foo.com/bar",
			match: true,
		},
		{
			input: "foo.com/bar/baz",
			match: true,
		},
		{
			input: "localhost:8080/bar",
			match: true,
		},
		{
			input: "sub-dom1.foo.com/bar/baz/quux",
			match: true,
		},
		{
			input: "blog.foo.com/bar/baz",
			match: true,
		},
		{
			input: "a^a",
			match: false,
		},
		{
			input: "aa/asdf$$^/aa",
			match: false,
		},
		{
			input: "asdf$$^/aa",
			match: false,
		},
		{
			input: "aa-a/a",
			match: true,
		},
		{
			input: strings.Repeat("a/", 128) + "a",
			match: true,
		},
		{
			input: "a-/a/a/a",
			match: false,
		},
		{
			input: "foo.com/a-/a/a",
			match: false,
		},
		{
			input: "-foo/bar",
			match: false,
		},
		{
			input: "foo/bar-",
			match: false,
		},
		{
			input: "foo-/bar",
			match: false,
		},
		{
			input: "foo/-bar",
			match: false,
		},
		{
			input: "_foo/bar",
			match: false,
		},
		{
			input: "foo_bar",
			match: true,
		},
		{
			input: "foo_bar.com",
			match: true,
		},
		{
			input: "foo_bar.com:8080",
			match: false,
		},
		{
			input: "foo_bar.com:8080/app",
			match: false,
		},
		{
			input: "foo.com/foo_bar",
			match: true,
		},
		{
			input: "____/____",
			match: false,
		},
		{
			input: "_docker/_docker",
			match: false,
		},
		{
			input: "docker_/docker_",
			match: false,
		},
		{
			input: "b.gcr.io/test.example.com/my-app",
			match: true,
		},
		{
			input: "xn--n3h.com/myimage", // ☃.com in punycode
			match: true,
		},
		{
			input: "xn--7o8h.com/myimage", // 🐳.com in punycode
			match: true,
		},
		{
			input: "example.com/xn--7o8h.com/myimage", // 🐳.com in punycode
			match: true,
		},
		{
			input: "example.com/some_separator__underscore/myimage",
			match: true,
		},
		{
			input: "example.com/__underscore/myimage",
			match: false,
		},
		{
			input: "example.com/..dots/myimage",
			match: false,
		},
		{
			input: "example.com/.dots/myimage",
			match: false,
		},
		{
			input: "example.com/nodouble..dots/myimage",
			match: false,
		},
		{
			input: "example.com/nodouble..dots/myimage",
			match: false,
		},
		{
			input: "docker./docker",
			match: false,
		},
		{
			input: ".docker/docker",
			match: false,
		},
		{
			input: "docker-/docker",
			match: false,
		},
		{
			input: "-docker/docker",
			match: false,
		},
		{
			input: "do..cker/docker",
			match: false,
		},
		{
			input: "do__cker:8080/docker",
			match: false,
		},
		{
			input: "do__cker/docker",
			match: true,
		},
		{
			input: "b.gcr.io/test.example.com/my-app",
			match: true,
		},
		{
			input: "registry.io/foo/project--id.module--name.ver---sion--name",
			match: true,
		},
		{
			input: "Asdf.com/foo/bar", // uppercase character in hostname
			match: true,
		},
		{
			input: "Foo/FarB", // uppercase characters in remote name
			match: false,
		},
	}

	b.ReportAllocs()
	for i := 0; i < b.N; i++ {
		for _, tc := range tests {
			_, _ = Parse(tc.input)
		}
	}
}
