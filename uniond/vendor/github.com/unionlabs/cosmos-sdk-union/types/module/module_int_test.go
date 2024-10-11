package module

import (
	"sort"
	"testing"

	"github.com/stretchr/testify/suite"

	"cosmossdk.io/core/appmodule"
)

func TestModuleIntSuite(t *testing.T) {
	suite.Run(t, new(TestSuite))
}

type TestSuite struct {
	suite.Suite
}

func (s *TestSuite) TestAssertNoForgottenModules() {
	m := Manager{
		Modules: map[string]appmodule.AppModule{"a": nil, "b": nil},
	}
	tcs := []struct {
		name     string
		positive bool
		modules  []string
		pass     func(string) bool
	}{
		{"less modules", false, []string{"a"}, nil},
		{"same modules", true, []string{"a", "b"}, nil},
		{"more modules", true, []string{"a", "b", "c"}, nil},
		{"pass module b", true, []string{"a"}, func(moduleName string) bool { return moduleName == "b" }},
	}

	for _, tc := range tcs {
		if tc.positive {
			m.assertNoForgottenModules("x", tc.modules, tc.pass)
		} else {
			s.Panics(func() { m.assertNoForgottenModules("x", tc.modules, tc.pass) })
		}
	}
}

func (s *TestSuite) TestModuleNames() {
	m := Manager{
		Modules: map[string]appmodule.AppModule{"a": nil, "b": nil},
	}
	ms := m.ModuleNames()
	sort.Strings(ms)
	s.Require().Equal([]string{"a", "b"}, ms)
}

func (s *TestSuite) TestDefaultMigrationsOrder() {
	require := s.Require()
	require.Equal(
		[]string{"auth2", "d", "z", "auth"},
		DefaultMigrationsOrder([]string{"d", "auth", "auth2", "z"}), "alphabetical, but auth should be last")
	require.Equal(
		[]string{"auth2", "d", "z"},
		DefaultMigrationsOrder([]string{"d", "auth2", "z"}), "alphabetical")
}
