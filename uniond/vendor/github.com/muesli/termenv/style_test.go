package termenv

import (
	"testing"
)

func TestStyleWidth(t *testing.T) {
	s := String("Hello World")
	if s.Width() != 11 {
		t.Errorf("Expected width of 11, got %d", s.Width())
	}

	s = s.Bold()
	if s.Width() != 11 {
		t.Errorf("Expected width of 11, got %d", s.Width())
	}

	s = s.Italic()
	if s.Width() != 11 {
		t.Errorf("Expected width of 11, got %d", s.Width())
	}

	s = s.Foreground(TrueColor.Color("#abcdef"))
	s = s.Background(TrueColor.Color("69"))
	if s.Width() != 11 {
		t.Errorf("Expected width of 11, got %d", s.Width())
	}
}
