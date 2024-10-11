package genopenapi

import "testing"

func TestNaming(t *testing.T) {
	type expectedNames struct {
		fqn, legacy, simple string
	}
	messageNameToExpected := map[string]expectedNames{
		".A":     {"A", "A", "A"},
		".a.B.C": {"a.B.C", "aBC", "B.C"},
		".a.D.C": {"a.D.C", "aDC", "D.C"},
		".a.E.F": {"a.E.F", "aEF", "a.E.F"},
		".b.E.F": {"b.E.F", "bEF", "b.E.F"},
		".c.G.H": {"c.G.H", "GH", "H"},
	}

	allMessageNames := make([]string, 0, len(messageNameToExpected))
	for msgName := range messageNameToExpected {
		allMessageNames = append(allMessageNames, msgName)
	}

	t.Run("fqn", func(t *testing.T) {
		uniqueNames := resolveNamesFQN(allMessageNames)
		for _, msgName := range allMessageNames {
			expected := messageNameToExpected[msgName].fqn
			actual := uniqueNames[msgName]
			if expected != actual {
				t.Errorf("fqn unique name %q does not match expected name %q", actual, expected)
			}
		}
	})
	t.Run("legacy", func(t *testing.T) {
		uniqueNames := resolveNamesLegacy(allMessageNames)
		for _, msgName := range allMessageNames {
			expected := messageNameToExpected[msgName].legacy
			actual := uniqueNames[msgName]
			if expected != actual {
				t.Errorf("legacy unique name %q does not match expected name %q", actual, expected)
			}
		}
	})
	t.Run("simple", func(t *testing.T) {
		uniqueNames := resolveNamesSimple(allMessageNames)
		for _, msgName := range allMessageNames {
			expected := messageNameToExpected[msgName].simple
			actual := uniqueNames[msgName]
			if expected != actual {
				t.Errorf("simple unique name %q does not match expected name %q", actual, expected)
			}
		}
	})
}
