package filepathx

import (
	"os"
	"strings"
	"testing"
)

func TestGlob_ZeroDoubleStars_oneMatch(t *testing.T) {
	// test passthru to vanilla path/filepath
	path := "./a/b/c.d/e.f"
	err := os.MkdirAll(path, 0755)
	if err != nil {
		t.Fatalf("os.MkdirAll: %s", err)
	}
	matches, err := Glob("./*/*/*.d")
	if err != nil {
		t.Fatalf("Glob: %s", err)
	}
	if len(matches) != 1 {
		t.Fatalf("got %d matches, expected 1", len(matches))
	}
	expected := strings.Join([]string{"a", "b", "c.d"}, string(os.PathSeparator))
	if matches[0] != expected {
		t.Fatalf("matched [%s], expected [%s]", matches[0], expected)
	}
}

func TestGlob_OneDoubleStar_oneMatch(t *testing.T) {
	// test a single double-star
	path := "./a/b/c.d/e.f"
	err := os.MkdirAll(path, 0755)
	if err != nil {
		t.Fatalf("os.MkdirAll: %s", err)
	}
	matches, err := Glob("./**/*.f")
	if err != nil {
		t.Fatalf("Glob: %s", err)
	}
	if len(matches) != 1 {
		t.Fatalf("got %d matches, expected 1", len(matches))
	}
	expected := strings.Join([]string{"a", "b", "c.d", "e.f"}, string(os.PathSeparator))
	if matches[0] != expected {
		t.Fatalf("matched [%s], expected [%s]", matches[0], expected)
	}
}

func TestGlob_OneDoubleStar_twoMatches(t *testing.T) {
	// test a single double-star
	path := "./a/b/c.d/e.f"
	err := os.MkdirAll(path, 0755)
	if err != nil {
		t.Fatalf("os.MkdirAll: %s", err)
	}
	matches, err := Glob("./a/**/*.*")
	if err != nil {
		t.Fatalf("Glob: %s", err)
	}
	if len(matches) != 2 {
		t.Fatalf("got %d matches, expected 2", len(matches))
	}
	expected := []string{
		strings.Join([]string{"a", "b", "c.d"}, string(os.PathSeparator)),
		strings.Join([]string{"a", "b", "c.d", "e.f"}, string(os.PathSeparator)),
	}

	for i, match := range matches {
		if match != expected[i] {
			t.Fatalf("matched [%s], expected [%s]", match, expected[i])
		}
	}
}

func TestGlob_TwoDoubleStars_oneMatch(t *testing.T) {
	// test two double-stars
	path := "./a/b/c.d/e.f"
	err := os.MkdirAll(path, 0755)
	if err != nil {
		t.Fatalf("os.MkdirAll: %s", err)
	}
	matches, err := Glob("./**/b/**/*.f")
	if err != nil {
		t.Fatalf("Glob: %s", err)
	}
	if len(matches) != 1 {
		t.Fatalf("got %d matches, expected 1", len(matches))
	}
	expected := strings.Join([]string{"a", "b", "c.d", "e.f"}, string(os.PathSeparator))

	if matches[0] != expected {
		t.Fatalf("matched [%s], expected [%s]", matches[0], expected)
	}
}

func TestExpand_DirectCall_emptySlice(t *testing.T) {
	var empty []string
	matches, err := Globs(empty).Expand()
	if err != nil {
		t.Fatalf("Glob: %s", err)
	}
	if len(matches) != 0 {
		t.Fatalf("got %d matches, expected 0", len(matches))
	}
}
