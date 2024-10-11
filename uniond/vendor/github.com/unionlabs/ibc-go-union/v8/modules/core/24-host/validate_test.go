package host

import (
	"fmt"
	"strings"
	"testing"

	"github.com/stretchr/testify/require"
)

// 195 characters
var longID = "Lorem ipsum dolor sit amet, consectetur adipiscing elit. Duis eros neque, ultricies vel ligula ac, convallis porttitor elit. Maecenas tincidunt turpis elit, vel faucibus nisl pellentesque sodales"

type testCase struct {
	msg     string
	id      string
	expPass bool
}

func TestDefaultIdentifierValidator(t *testing.T) {
	testCases := []testCase{
		{"valid lowercase", "lowercaseid", true},
		{"valid id special chars", "._+-#[]<>._+-#[]<>", true},
		{"valid id lower and special chars", "lower._+-#[]<>", true},
		{"numeric id", "1234567890", true},
		{"uppercase id", "NOTLOWERCASE", true},
		{"numeric id", "1234567890", true},
		{"blank id", "               ", false},
		{"id length out of range", "1", false},
		{"id is too long", "this identifier is too long to be used as a valid identifier", false},
		{"path-like id", "lower/case/id", false},
		{"invalid id", "(clientid)", false},
		{"empty string", "", false},
	}

	for _, tc := range testCases {
		tc := tc

		err := ClientIdentifierValidator(tc.id)
		err1 := ConnectionIdentifierValidator(tc.id)
		err2 := ChannelIdentifierValidator(tc.id)
		err3 := PortIdentifierValidator(tc.id)
		if tc.expPass {
			require.NoError(t, err, tc.msg)
			require.NoError(t, err1, tc.msg)
			require.NoError(t, err2, tc.msg)
			require.NoError(t, err3, tc.msg)
		} else {
			require.Error(t, err, tc.msg)
			require.Error(t, err1, tc.msg)
			require.Error(t, err2, tc.msg)
			require.Error(t, err3, tc.msg)
		}
	}
}

func TestPortIdentifierValidator(t *testing.T) {
	testCases := []testCase{
		{"valid lowercase", "transfer", true},
		{"valid id special chars", "._+-#[]<>._+-#[]<>", true},
		{"valid id lower and special chars", "lower._+-#[]<>", true},
		{"numeric id", "1234567890", true},
		{"uppercase id", "NOTLOWERCASE", true},
		{"numeric id", "1234567890", true},
		{"blank id", "               ", false},
		{"id length out of range", "1", false},
		{"id is too long", longID, false},
		{"path-like id", "lower/case/id", false},
		{"invalid id", "(clientid)", false},
		{"empty string", "", false},
	}

	for _, tc := range testCases {
		tc := tc

		err := PortIdentifierValidator(tc.id)
		if tc.expPass {
			require.NoError(t, err, tc.msg)
		} else {
			require.Error(t, err, tc.msg)
		}
	}
}

func TestPathValidator(t *testing.T) {
	testCases := []testCase{
		{"valid lowercase", "p/lowercaseid", true},
		{"numeric path", "p/239123", true},
		{"valid id special chars", "p/._+-#[]<>._+-#[]<>", true},
		{"valid id lower and special chars", "lower/._+-#[]<>", true},
		{"id length out of range", "p/l", true},
		{"uppercase id", "p/NOTLOWERCASE", true},
		{"invalid path", "lowercaseid", false},
		{"blank id", "p/               ", false},
		{"id length out of range", "p/12345678901234567890123456789012345678901234567890123456789012345", false},
		{"invalid id", "p/(clientid)", false},
		{"empty string", "", false},
		{"separators only", "////", false},
		{"just separator", "/", false},
		{"begins with separator", "/id", false},
		{"blank before separator", "    /id", false},
		{"ends with separator", "id/", false},
		{"blank after separator", "id/       ", false},
		{"blanks with separator", "  /  ", false},
	}

	for _, tc := range testCases {
		tc := tc

		f := NewPathValidator(func(path string) error {
			return nil
		})

		err := f(tc.id)

		if tc.expPass {
			seps := strings.Count(tc.id, "/")
			require.Equal(t, 1, seps)
			require.NoError(t, err, tc.msg)
		} else {
			require.Error(t, err, tc.msg)
		}
	}
}

func TestCustomPathValidator(t *testing.T) {
	validateFn := NewPathValidator(func(path string) error {
		if !strings.HasPrefix(path, "id_") {
			return fmt.Errorf("identifier %s must start with 'id_", path)
		}
		return nil
	})

	testCases := []testCase{
		{"valid custom path", "id_client/id_one", true},
		{"invalid path", "client", false},
		{"invalid custom path", "id_one/client", false},
		{"invalid identifier", "id_client/id_1234567890123456789012345678901234567890123457890123456789012345", false},
		{"separators only", "////", false},
		{"just separator", "/", false},
		{"ends with separator", "id_client/id_one/", false},
		{"beings with separator", "/id_client/id_one", false},
	}

	for _, tc := range testCases {
		tc := tc

		err := validateFn(tc.id)
		if tc.expPass {
			require.NoError(t, err, tc.msg)
		} else {
			require.Error(t, err, tc.msg)
		}
	}
}
