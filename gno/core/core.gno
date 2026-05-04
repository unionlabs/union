package core

import (
	"chain"
	"strconv"
)

type ClientId uint32
var nextClientId ClientId
func generateClientId() ClientId {
	var id = nextClientId
	nextClientId += 1
	return id
}

func (id ClientId) String() string {
	return strconv.FormatUint(uint64(id), 10)
}

type ConnectionId uint32
var nextConnectionId ConnectionId
func generateConnectionId() ConnectionId {
	var id = nextConnectionId
	nextConnectionId += 1
	return id
}

type ChannelId uint32
var nextChannelId ChannelId
func generateChannelId() ChannelId {
	var id = nextChannelId
	nextChannelId += 1
	return id
}

type Timestamp uint64

var clientRegistry map[string]ILightClient

func init() {
	clientRegistry = make(map[string]ILightClient)
}

type ILightClient struct {
	CreateClient func(clientId ClientId, clientStateBytes, consensusStateBytes []byte)
}

func RegisterClient(_ realm, clientType string, client ILightClient) {
	clientRegistry[clientType] = client
}

const CreateClientEvent = "CreateClient"

func CreateClient(_ realm, clientType string, clientStateBytes, consensusStateBytes []byte) ClientId {
	var impl = clientRegistry[clientType]

	var clientId = generateClientId()

	impl.CreateClient(clientId, clientStateBytes, consensusStateBytes)

	chain.Emit(
		CreateClientEvent,
		"client_type", clientType,
		"client_id", clientId.String(),
	)

	return clientId
}

func Render(_ string) string {
	var s = ""

	s += "next client id: "
	s += nextClientId.String()

	s += "\n\n"

	s += "registered clients: ["

	for key := range clientRegistry {
		s += " "
		s += key
	}

	s += " ]"

	return s
}
