package coretypes

import (
	"testing"

	"github.com/stretchr/testify/assert"

	"github.com/cometbft/cometbft/p2p"
)

func TestStatusIndexer(t *testing.T) {
	var status *ResultStatus
	assert.False(t, status.TxIndexEnabled())

	status = &ResultStatus{}
	assert.False(t, status.TxIndexEnabled())

	status.NodeInfo = p2p.DefaultNodeInfo{}
	assert.False(t, status.TxIndexEnabled())

	cases := []struct {
		expected bool
		other    p2p.DefaultNodeInfoOther
	}{
		{false, p2p.DefaultNodeInfoOther{}},
		{false, p2p.DefaultNodeInfoOther{TxIndex: "aa"}},
		{false, p2p.DefaultNodeInfoOther{TxIndex: "off"}},
		{true, p2p.DefaultNodeInfoOther{TxIndex: "on"}},
	}

	for _, tc := range cases {
		status.NodeInfo.Other = tc.other
		assert.Equal(t, tc.expected, status.TxIndexEnabled())
	}
}
