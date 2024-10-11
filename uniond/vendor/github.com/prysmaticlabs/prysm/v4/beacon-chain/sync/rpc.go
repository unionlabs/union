package sync

import (
	"context"
	"reflect"
	"runtime/debug"
	"strings"
	"time"

	libp2pcore "github.com/libp2p/go-libp2p/core"
	"github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/protocol"
	ssz "github.com/prysmaticlabs/fastssz"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	p2ptypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/types"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/monitoring/tracing"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
	"go.opencensus.io/trace"
)

// Time to first byte timeout. The maximum time to wait for first byte of
// request response (time-to-first-byte). The client is expected to give up if
// they don't receive the first byte within 5 seconds.
var ttfbTimeout = params.BeaconConfig().TtfbTimeoutDuration()

// respTimeout is the maximum time for complete response transfer.
var respTimeout = params.BeaconConfig().RespTimeoutDuration()

// rpcHandler is responsible for handling and responding to any incoming message.
// This method may return an error to internal monitoring, but the error will
// not be relayed to the peer.
type rpcHandler func(context.Context, interface{}, libp2pcore.Stream) error

// registerRPCHandlers for p2p RPC.
func (s *Service) registerRPCHandlers() {
	currEpoch := slots.ToEpoch(s.cfg.clock.CurrentSlot())
	// Register V2 handlers if we are past altair fork epoch.
	if currEpoch >= params.BeaconConfig().AltairForkEpoch {
		s.registerRPC(
			p2p.RPCStatusTopicV1,
			s.statusRPCHandler,
		)
		s.registerRPC(
			p2p.RPCGoodByeTopicV1,
			s.goodbyeRPCHandler,
		)
		s.registerRPC(
			p2p.RPCPingTopicV1,
			s.pingHandler,
		)
		s.registerRPCHandlersAltair()
		if currEpoch >= params.BeaconConfig().DenebForkEpoch {
			s.registerRPCHandlersDeneb()
		}
		return
	}
	s.registerRPC(
		p2p.RPCStatusTopicV1,
		s.statusRPCHandler,
	)
	s.registerRPC(
		p2p.RPCGoodByeTopicV1,
		s.goodbyeRPCHandler,
	)
	s.registerRPC(
		p2p.RPCBlocksByRangeTopicV1,
		s.beaconBlocksByRangeRPCHandler,
	)
	s.registerRPC(
		p2p.RPCBlocksByRootTopicV1,
		s.beaconBlocksRootRPCHandler,
	)
	s.registerRPC(
		p2p.RPCPingTopicV1,
		s.pingHandler,
	)
	s.registerRPC(
		p2p.RPCMetaDataTopicV1,
		s.metaDataHandler,
	)
}

// registerRPCHandlers for altair.
func (s *Service) registerRPCHandlersAltair() {
	s.registerRPC(
		p2p.RPCBlocksByRangeTopicV2,
		s.beaconBlocksByRangeRPCHandler,
	)
	s.registerRPC(
		p2p.RPCBlocksByRootTopicV2,
		s.beaconBlocksRootRPCHandler,
	)
	s.registerRPC(
		p2p.RPCMetaDataTopicV2,
		s.metaDataHandler,
	)
}

func (s *Service) registerRPCHandlersDeneb() {
	s.registerRPC(
		p2p.RPCBlobSidecarsByRangeTopicV1,
		s.blobSidecarsByRangeRPCHandler,
	)
	s.registerRPC(
		p2p.RPCBlobSidecarsByRootTopicV1,
		s.blobSidecarByRootRPCHandler,
	)
}

// Remove all v1 Stream handlers that are no longer supported
// from altair onwards.
func (s *Service) unregisterPhase0Handlers() {
	fullBlockRangeTopic := p2p.RPCBlocksByRangeTopicV1 + s.cfg.p2p.Encoding().ProtocolSuffix()
	fullBlockRootTopic := p2p.RPCBlocksByRootTopicV1 + s.cfg.p2p.Encoding().ProtocolSuffix()
	fullMetadataTopic := p2p.RPCMetaDataTopicV1 + s.cfg.p2p.Encoding().ProtocolSuffix()

	s.cfg.p2p.Host().RemoveStreamHandler(protocol.ID(fullBlockRangeTopic))
	s.cfg.p2p.Host().RemoveStreamHandler(protocol.ID(fullBlockRootTopic))
	s.cfg.p2p.Host().RemoveStreamHandler(protocol.ID(fullMetadataTopic))
}

// registerRPC for a given topic with an expected protobuf message type.
func (s *Service) registerRPC(baseTopic string, handle rpcHandler) {
	topic := baseTopic + s.cfg.p2p.Encoding().ProtocolSuffix()
	log := log.WithField("topic", topic)
	s.cfg.p2p.SetStreamHandler(topic, func(stream network.Stream) {
		defer func() {
			if r := recover(); r != nil {
				log.WithField("error", r).
					WithField("recovered_at", "registerRPC").
					WithField("stack", string(debug.Stack())).
					Error("Panic occurred")
			}
		}()

		ctx, cancel := context.WithTimeout(s.ctx, ttfbTimeout)
		defer cancel()

		// Resetting after closing is a no-op so defer a reset in case something goes wrong.
		// It's up to the handler to Close the stream (send an EOF) if
		// it successfully writes a response. We don't blindly call
		// Close here because we may have only written a partial
		// response.
		defer func() {
			_err := stream.Reset()
			_ = _err
		}()

		ctx, span := trace.StartSpan(ctx, "sync.rpc")
		defer span.End()
		span.AddAttributes(trace.StringAttribute("topic", topic))
		span.AddAttributes(trace.StringAttribute("peer", stream.Conn().RemotePeer().String()))
		log := log.WithField("peer", stream.Conn().RemotePeer().String()).WithField("topic", string(stream.Protocol()))

		// Check before hand that peer is valid.
		if s.cfg.p2p.Peers().IsBad(stream.Conn().RemotePeer()) {
			if err := s.sendGoodByeAndDisconnect(ctx, p2ptypes.GoodbyeCodeBanned, stream.Conn().RemotePeer()); err != nil {
				log.WithError(err).Debug("Could not disconnect from peer")
			}
			return
		}
		// Validate request according to peer limits.
		if err := s.rateLimiter.validateRawRpcRequest(stream); err != nil {
			log.WithError(err).Debug("Could not validate rpc request from peer")
			return
		}
		s.rateLimiter.addRawStream(stream)

		if err := stream.SetReadDeadline(time.Now().Add(ttfbTimeout)); err != nil {
			log.WithError(err).Debug("Could not set stream read deadline")
			return
		}

		base, ok := p2p.RPCTopicMappings[baseTopic]
		if !ok {
			log.Errorf("Could not retrieve base message for topic %s", baseTopic)
			return
		}
		t := reflect.TypeOf(base)
		// Copy Base
		base = reflect.New(t)

		// Increment message received counter.
		messageReceivedCounter.WithLabelValues(topic).Inc()

		// since metadata requests do not have any data in the payload, we
		// do not decode anything.
		if baseTopic == p2p.RPCMetaDataTopicV1 || baseTopic == p2p.RPCMetaDataTopicV2 {
			if err := handle(ctx, base, stream); err != nil {
				messageFailedProcessingCounter.WithLabelValues(topic).Inc()
				if err != p2ptypes.ErrWrongForkDigestVersion {
					log.WithError(err).Debug("Could not handle p2p RPC")
				}
				tracing.AnnotateError(span, err)
			}
			return
		}

		// Given we have an input argument that can be pointer or the actual object, this gives us
		// a way to check for its reflect.Kind and based on the result, we can decode
		// accordingly.
		if t.Kind() == reflect.Ptr {
			msg, ok := reflect.New(t.Elem()).Interface().(ssz.Unmarshaler)
			if !ok {
				log.Errorf("message of %T does not support marshaller interface", msg)
				return
			}
			if err := s.cfg.p2p.Encoding().DecodeWithMaxLength(stream, msg); err != nil {
				logStreamErrors(err, topic)
				tracing.AnnotateError(span, err)
				s.cfg.p2p.Peers().Scorers().BadResponsesScorer().Increment(stream.Conn().RemotePeer())
				return
			}
			if err := handle(ctx, msg, stream); err != nil {
				messageFailedProcessingCounter.WithLabelValues(topic).Inc()
				if err != p2ptypes.ErrWrongForkDigestVersion {
					log.WithError(err).Debug("Could not handle p2p RPC")
				}
				tracing.AnnotateError(span, err)
			}
		} else {
			nTyp := reflect.New(t)
			msg, ok := nTyp.Interface().(ssz.Unmarshaler)
			if !ok {
				log.Errorf("message of %T does not support marshaller interface", msg)
				return
			}
			if err := s.cfg.p2p.Encoding().DecodeWithMaxLength(stream, msg); err != nil {
				logStreamErrors(err, topic)
				tracing.AnnotateError(span, err)
				s.cfg.p2p.Peers().Scorers().BadResponsesScorer().Increment(stream.Conn().RemotePeer())
				return
			}
			if err := handle(ctx, nTyp.Elem().Interface(), stream); err != nil {
				messageFailedProcessingCounter.WithLabelValues(topic).Inc()
				if err != p2ptypes.ErrWrongForkDigestVersion {
					log.WithError(err).Debug("Could not handle p2p RPC")
				}
				tracing.AnnotateError(span, err)
			}
		}
	})
}

func logStreamErrors(err error, topic string) {
	if strings.Contains(topic, p2p.RPCGoodByeTopicV1) {
		log.WithError(err).WithField("topic", topic).Trace("Could not decode goodbye stream message")
		return
	}
	log.WithError(err).WithField("topic", topic).Debug("Could not decode stream message")
}
