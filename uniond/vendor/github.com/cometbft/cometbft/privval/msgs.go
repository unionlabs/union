package privval

import (
	"fmt"

	"github.com/cosmos/gogoproto/proto"

	pvproto "github.com/cometbft/cometbft/api/cometbft/privval/v1"
)

// TODO: Add ChainIDRequest

func mustWrapMsg(pb proto.Message) pvproto.Message {
	msg := pvproto.Message{}

	switch pb := pb.(type) {
	case *pvproto.Message:
		msg = *pb
	case *pvproto.PubKeyRequest:
		msg.Sum = &pvproto.Message_PubKeyRequest{PubKeyRequest: pb}
	case *pvproto.PubKeyResponse:
		msg.Sum = &pvproto.Message_PubKeyResponse{PubKeyResponse: pb}
	case *pvproto.SignVoteRequest:
		msg.Sum = &pvproto.Message_SignVoteRequest{SignVoteRequest: pb}
	case *pvproto.SignBytesRequest:
		msg.Sum = &pvproto.Message_SignBytesRequest{SignBytesRequest: pb}
	case *pvproto.SignBytesResponse:
		msg.Sum = &pvproto.Message_SignBytesResponse{SignBytesResponse: pb}
	case *pvproto.SignedVoteResponse:
		msg.Sum = &pvproto.Message_SignedVoteResponse{SignedVoteResponse: pb}
	case *pvproto.SignedProposalResponse:
		msg.Sum = &pvproto.Message_SignedProposalResponse{SignedProposalResponse: pb}
	case *pvproto.SignProposalRequest:
		msg.Sum = &pvproto.Message_SignProposalRequest{SignProposalRequest: pb}
	case *pvproto.PingRequest:
		msg.Sum = &pvproto.Message_PingRequest{PingRequest: pb}
	case *pvproto.PingResponse:
		msg.Sum = &pvproto.Message_PingResponse{PingResponse: pb}
	default:
		panic(fmt.Errorf("unknown message type %T", pb))
	}

	return msg
}
