package getter

import (
	"net/url"
	"testing"
)

func TestRedactURL(t *testing.T) {
	cases := []struct {
		name string
		url  *url.URL
		want string
	}{
		{
			name: "non-blank Password",
			url: &url.URL{
				Scheme: "http",
				Host:   "host.tld",
				Path:   "this:that",
				User:   url.UserPassword("user", "password"),
			},
			want: "http://user:redacted@host.tld/this:that",
		},
		{
			name: "blank Password",
			url: &url.URL{
				Scheme: "http",
				Host:   "host.tld",
				Path:   "this:that",
				User:   url.User("user"),
			},
			want: "http://user@host.tld/this:that",
		},
		{
			name: "nil User",
			url: &url.URL{
				Scheme: "http",
				Host:   "host.tld",
				Path:   "this:that",
				User:   url.UserPassword("", "password"),
			},
			want: "http://:redacted@host.tld/this:that",
		},
		{
			name: "blank Username, blank Password",
			url: &url.URL{
				Scheme: "http",
				Host:   "host.tld",
				Path:   "this:that",
			},
			want: "http://host.tld/this:that",
		},
		{
			name: "empty URL",
			url:  &url.URL{},
			want: "",
		},
		{
			name: "nil URL",
			url:  nil,
			want: "",
		},
		{
			name: "non-blank SSH key in URL query parameter",
			url: &url.URL{
				Scheme:   "ssh",
				User:     url.User("git"),
				Host:     "github.com",
				Path:     "hashicorp/go-getter-test-private.git",
				RawQuery: "sshkey=LS0tLS1CRUdJTiBPUE",
			},
			want: "ssh://git@github.com/hashicorp/go-getter-test-private.git?sshkey=redacted",
		},
		{
			name: "blank SSH key in URL query parameter",
			url: &url.URL{
				Scheme:   "ssh",
				User:     url.User("git"),
				Host:     "github.com",
				Path:     "hashicorp/go-getter-test-private.git",
				RawQuery: "sshkey=",
			},
			want: "ssh://git@github.com/hashicorp/go-getter-test-private.git?sshkey=",
		},
	}

	for _, tt := range cases {
		t := t
		t.Run(tt.name, func(t *testing.T) {
			if g, w := RedactURL(tt.url), tt.want; g != w {
				t.Fatalf("got: %q\nwant: %q", g, w)
			}
		})
	}
}
