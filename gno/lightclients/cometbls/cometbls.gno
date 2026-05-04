package cometbls

import "gno.land/r/union/core"

// var coreAddress address;

func Register(_ realm) {
	core.RegisterClient(cross, "cometbls", core.ILightClient {
		CreateClient: func(clientId core.ClientId, clientStateBytes, consensusStateBytes []byte) {
			println(clientId)
		},
	})
}

// func Render(_ string) string {
// 	var s = "registered clients: {";

//     for key := range clientRegistry {
//         s += " "
//         s += key
//     }

//     s += "}"

//     return s
// }
