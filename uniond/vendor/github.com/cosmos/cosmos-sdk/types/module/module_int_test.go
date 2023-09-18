package module

import (
	"sort"
	"testing"

	"github.com/stretchr/testify/suite"
)

func TestModuleIntSuite(t *testing.T) {
	suite.Run(t, new(TestSuite))
}

type TestSuite struct {
	suite.Suite
}

func (s TestSuite) TestAssertNoForgottenModules() {
	m := Manager{
		Modules: map[string]interface{}{"a": nil, "b": nil},
	}
	tcs := []struct {
		name     string
		positive bool
		modules  []string
	}{
		{"same modules", true, []string{"a", "b"}},
		{"more modules", true, []string{"a", "b", "c"}},
	}

	for _, tc := range tcs {
		if tc.positive {
			m.assertNoForgottenModules("x", tc.modules)
		} else {
			s.Panics(func() { m.assertNoForgottenModules("x", tc.modules) })
		}
	}
}

func (s TestSuite) TestModuleNames() {
	m := Manager{
		Modules: map[string]interface{}{"a": nil, "b": nil},
	}
	ms := m.ModuleNames()
	sort.Strings(ms)
	s.Require().Equal([]string{"a", "b"}, ms)
}

func (s TestSuite) TestDefaultMigrationsOrder() {
	require := s.Require()
	require.Equal(
		[]string{"auth2", "d", "z", "auth"},
		DefaultMigrationsOrder([]string{"d", "auth", "auth2", "z"}), "alphabetical, but auth should be last")
	require.Equal(
		[]string{"auth2", "d", "z"},
		DefaultMigrationsOrder([]string{"d", "auth2", "z"}), "alphabetical")
}
