package p2p

import (
	"context"

	"github.com/kr/pretty"
	"github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/libp2p/go-libp2p/core/protocol"
	"github.com/pkg/errors"
	ssz "github.com/prysmaticlabs/fastssz"
	"github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	"github.com/sirupsen/logrus"
	"go.opencensus.io/trace"
)

// Send a message to a specific peer. The returned stream may be used for reading, but has been
// closed for writing.
//
// When done, the caller must Close or Reset on the stream.
func (s *Service) Send(ctx context.Context, message interface{}, baseTopic string, pid peer.ID) (network.Stream, error) {
	ctx, span := trace.StartSpan(ctx, "p2p.Send")
	defer span.End()
	if err := VerifyTopicMapping(baseTopic, message); err != nil {
		return nil, err
	}
	topic := baseTopic + s.Encoding().ProtocolSuffix()
	span.AddAttributes(trace.StringAttribute("topic", topic))

	log.WithFields(logrus.Fields{
		"topic":   topic,
		"request": pretty.Sprint(message),
	}).Tracef("Sending RPC request to peer %s", pid.String())

	// Apply max dial timeout when opening a new stream.
	ctx, cancel := context.WithTimeout(ctx, maxDialTimeout)
	defer cancel()

	stream, err := s.host.NewStream(ctx, pid, protocol.ID(topic))
	if err != nil {
		tracing.AnnotateError(span, err)
		return nil, err
	}
	// do not encode anything if we are sending a metadata request
	if baseTopic != RPCMetaDataTopicV1 && baseTopic != RPCMetaDataTopicV2 {
		castedMsg, ok := message.(ssz.Marshaler)
		if !ok {
			return nil, errors.Errorf("%T does not support the ssz marshaller interface", message)
		}
		if _, err := s.Encoding().EncodeWithMaxLength(stream, castedMsg); err != nil {
			tracing.AnnotateError(span, err)
			_err := stream.Reset()
			_ = _err
			return nil, err
		}
	}

	// Close stream for writing.
	if err := stream.CloseWrite(); err != nil {
		tracing.AnnotateError(span, err)
		_err := stream.Reset()
		_ = _err
		return nil, err
	}

	return stream, nil
}
