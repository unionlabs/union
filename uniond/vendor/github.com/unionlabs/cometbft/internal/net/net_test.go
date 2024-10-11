package net

import (
	"testing"

	"github.com/stretchr/testify/assert"
)

func TestProtocolAndAddress(t *testing.T) {
	cases := []struct {
		fullAddr string
		proto    string
		addr     string
	}{
		{
			"tcp://mydomain:80",
			"tcp",
			"mydomain:80",
		},
		{
			"mydomain:80",
			"tcp",
			"mydomain:80",
		},
		{
			"unix://mydomain:80",
			"unix",
			"mydomain:80",
		},
	}

	for _, c := range cases {
		proto, addr := ProtocolAndAddress(c.fullAddr)
		assert.Equal(t, proto, c.proto)
		assert.Equal(t, addr, c.addr)
	}
}
