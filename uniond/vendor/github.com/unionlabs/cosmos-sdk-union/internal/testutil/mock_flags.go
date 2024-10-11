package testutil

import "strings"

type MockFlagsWithComma struct {
	Ary     []string
	changed bool
}

func (m *MockFlagsWithComma) String() string {
	return strings.Join(m.Ary, ",")
}

func (m *MockFlagsWithComma) Set(value string) error {
	if m.changed {
		m.Ary = append(m.Ary, strings.Split(value, ",")...)
	} else {
		m.Ary = strings.Split(value, ",")
		m.changed = true
	}
	return nil
}

func (m *MockFlagsWithComma) Type() string {
	return "mock_flags"
}

func (m *MockFlagsWithComma) Replace(value []string) error {
	m.Ary = value
	return nil
}

func (m *MockFlagsWithComma) Append(value string) error {
	m.Ary = append(m.Ary, value)
	return nil
}

func (m *MockFlagsWithComma) GetSlice() []string {
	return m.Ary
}

type MockFlagsWithSemicolon struct {
	Ary     []string
	changed bool
}

func (m *MockFlagsWithSemicolon) String() string {
	return strings.Join(m.Ary, ";")
}

func (m *MockFlagsWithSemicolon) Set(value string) error {
	if m.changed {
		m.Ary = append(m.Ary, strings.Split(value, ";")...)
	} else {
		m.Ary = strings.Split(value, ";")
		m.changed = true
	}
	return nil
}

func (m *MockFlagsWithSemicolon) Type() string {
	return "mock_flags"
}

func (m *MockFlagsWithSemicolon) Replace(value []string) error {
	m.Ary = value
	return nil
}

func (m *MockFlagsWithSemicolon) Append(value string) error {
	m.Ary = append(m.Ary, value)
	return nil
}

func (m *MockFlagsWithSemicolon) GetSlice() []string {
	return m.Ary
}
