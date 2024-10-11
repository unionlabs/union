package casing

import "testing"

func TestCamelIdentifier(t *testing.T) {
	casingTests := []struct {
		name  string
		input string
		want  string
	}{
		{
			"regular snake case identifier",
			"snake_case",
			"SnakeCase",
		},
		{
			"snake case identifier with digit",
			"snake_case_0_enum",
			"SnakeCase_0Enum",
		},
		{
			"regular snake case identifier with package",
			"pathenum.snake_case",
			"pathenum.SnakeCase",
		},
		{
			"snake case identifier with digit and package",
			"pathenum.snake_case_0_enum",
			"pathenum.SnakeCase_0Enum",
		},
		{
			"snake case identifier with digit and multiple dots",
			"path.pathenum.snake_case_0_enum",
			"path.pathenum.SnakeCase_0Enum",
		},
	}

	for _, ct := range casingTests {
		t.Run(ct.name, func(t *testing.T) {
			got := CamelIdentifier(ct.input)
			if ct.want != got {
				t.Errorf("want: %s, got: %s", ct.want, got)
			}
		})
	}
}
