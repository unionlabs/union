package getter

import (
	"net/url"
	"testing"
)

func TestAddAuthFromNetrc(t *testing.T) {
	defer tempEnv(t, "NETRC", "./testdata/netrc/basic")()

	u, err := url.Parse("http://example.com")
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	if err := addAuthFromNetrc(u); err != nil {
		t.Fatalf("err: %s", err)
	}

	expected := "http://foo:bar@example.com"
	actual := u.String()
	if expected != actual {
		t.Fatalf("Mismatch: %q != %q", actual, expected)
	}
}

func TestAddAuthFromNetrc_hasAuth(t *testing.T) {
	defer tempEnv(t, "NETRC", "./testdata/netrc/basic")()

	u, err := url.Parse("http://username:password@example.com")
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	expected := u.String()
	if err := addAuthFromNetrc(u); err != nil {
		t.Fatalf("err: %s", err)
	}

	actual := u.String()
	if expected != actual {
		t.Fatalf("Mismatch: %q != %q", actual, expected)
	}
}

func TestAddAuthFromNetrc_hasUsername(t *testing.T) {
	defer tempEnv(t, "NETRC", "./testdata/netrc/basic")()

	u, err := url.Parse("http://username@example.com")
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	expected := u.String()
	if err := addAuthFromNetrc(u); err != nil {
		t.Fatalf("err: %s", err)
	}

	actual := u.String()
	if expected != actual {
		t.Fatalf("Mismatch: %q != %q", actual, expected)
	}
}

func TestAddAuthFromNetrc_isNotExist(t *testing.T) {
	defer tempEnv(t, "NETRC", "./testdata/netrc/_does_not_exist")()

	u, err := url.Parse("http://example.com")
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	if err := addAuthFromNetrc(u); err != nil {
		t.Fatalf("err: %s", err)
	}

	// no netrc, no change:
	expected := "http://example.com"
	actual := u.String()
	if expected != actual {
		t.Fatalf("Mismatch: %q != %q", actual, expected)
	}
}

func TestAddAuthFromNetrc_isNotADirectory(t *testing.T) {
	defer tempEnv(t, "NETRC", "./testdata/netrc/basic/parent-not-a-dir")()

	u, err := url.Parse("http://example.com")
	if err != nil {
		t.Fatalf("err: %s", err)
	}

	if err := addAuthFromNetrc(u); err != nil {
		t.Fatalf("err: %s", err)
	}

	// no netrc, no change:
	expected := "http://example.com"
	actual := u.String()
	if expected != actual {
		t.Fatalf("Mismatch: %q != %q", actual, expected)
	}
}
