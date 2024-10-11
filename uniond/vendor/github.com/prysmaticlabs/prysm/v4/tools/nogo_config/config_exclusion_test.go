package main

import "testing"

func TestAddExclusion(t *testing.T) {
	cfg := Configs{
		"foo": Config{},
	}

	cfg.AddExclusion("sa0000", []string{"foo.go", "bar.go"})

	if len(cfg["sa0000"].ExcludeFiles) != 2 {
		t.Errorf("Expected 2 exclusions, got %d", len(cfg["sa0000"].ExcludeFiles))
	}
	if cfg["sa0000"].ExcludeFiles["foo.go"] != exclusionMessage {
		t.Errorf("Expected exclusion message, got %s", cfg["sa0000"].ExcludeFiles["foo.go"])
	}
	if cfg["sa0000"].ExcludeFiles["bar.go"] != exclusionMessage {
		t.Errorf("Expected exclusion message, got %s", cfg["sa0000"].ExcludeFiles["bar.go"])
	}

	cfg.AddExclusion("sa0000", []string{"foo.go", "baz.go"})
	if len(cfg["sa0000"].ExcludeFiles) != 3 {
		t.Errorf("Expected 3 exclusions, got %d", len(cfg["sa0000"].ExcludeFiles))
	}
	if cfg["sa0000"].ExcludeFiles["baz.go"] != exclusionMessage {
		t.Errorf("Expected exclusion message, got %s", cfg["sa0000"].ExcludeFiles["baz.go"])
	}
}
