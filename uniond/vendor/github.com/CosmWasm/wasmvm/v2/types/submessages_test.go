package types

import (
	"encoding/json"
	"testing"

	"github.com/stretchr/testify/require"
)

func TestReplySerialization(t *testing.T) {
	reply1 := Reply{
		GasUsed: 4312324,
		ID:      75,
		Result: SubMsgResult{
			Ok: &SubMsgResponse{
				Events: Array[Event]{
					{
						Type: "hi",
						Attributes: Array[EventAttribute]{
							{
								Key:   "si",
								Value: "claro",
							},
						},
					},
				},
				Data: []byte{0x3f, 0x00, 0xaa, 0x5c, 0xab},
				MsgResponses: Array[MsgResponse]{
					MsgResponse{
						TypeURL: "/cosmos.bank.v1beta1.MsgSendResponse",
						Value:   []byte{},
					},
				},
			},
		},
		Payload: []byte("payload"),
	}
	serialized, err := json.Marshal(&reply1)
	require.NoError(t, err)
	require.Equal(t, `{"gas_used":4312324,"id":75,"result":{"ok":{"events":[{"type":"hi","attributes":[{"key":"si","value":"claro"}]}],"data":"PwCqXKs=","msg_responses":[{"type_url":"/cosmos.bank.v1beta1.MsgSendResponse","value":""}]}},"payload":"cGF5bG9hZA=="}`, string(serialized))

	withoutPayload := Reply{
		GasUsed: 4312324,
		ID:      75,
		Result: SubMsgResult{
			Err: "some error",
		},
	}
	serialized2, err := json.Marshal(&withoutPayload)
	require.NoError(t, err)
	require.Equal(t, `{"gas_used":4312324,"id":75,"result":{"error":"some error"}}`, string(serialized2))
}

func TestSubMsgResponseSerialization(t *testing.T) {
	response := SubMsgResponse{}
	document, err := json.Marshal(response)
	require.NoError(t, err)
	require.Equal(t, `{"events":[],"msg_responses":[]}`, string(document))

	// we really only care about marshal, but let's test unmarshal too
	document2 := []byte(`{}`)
	var response2 SubMsgResponse
	err = json.Unmarshal(document2, &response2)
	require.NoError(t, err)
	require.Equal(t, response, response2)
}
