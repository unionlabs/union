// +build darwin ios

package bindtest

import (
	"testing"

	"github.com/keybase/go-keychain/bind"
	"github.com/stretchr/testify/require"
)

type test struct {
	t *testing.T
}

func (t test) Fail(s string) {
	require.Fail(t.t, s)
}

func TestGenericPassword(t *testing.T) {
	service := "Testing service as unicode テスト"
	accessGroup := ""
	bind.GenericPasswordTest(test{t}, service, accessGroup)
}
