package p2p

import (
	"context"
	"reflect"
	"runtime/debug"
	"strings"

	libp2pcore "github.com/libp2p/go-libp2p/core"
	corenet "github.com/libp2p/go-libp2p/core/network"
	"github.com/libp2p/go-libp2p/core/protocol"
	ssz "github.com/prysmaticlabs/fastssz"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	p2ptypes "github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/types"
)

type rpcHandler func(context.Context, interface{}, libp2pcore.Stream) error

// registerRPC for a given topic with an expected protobuf message type.
func (c *client) registerRPCHandler(baseTopic string, handle rpcHandler) {
	topic := baseTopic + c.Encoding().ProtocolSuffix()
	c.host.SetStreamHandler(protocol.ID(topic), func(stream corenet.Stream) {
		defer func() {
			if r := recover(); r != nil {
				log.WithField("error", r).Error("Panic occurred")
				log.Errorf("%s", debug.Stack())
			}
		}()
		// Resetting after closing is a no-op so defer a reset in case something goes wrong.
		// It's up to the handler to Close the stream (send an EOF) if
		// it successfully writes a response. We don't blindly call
		// Close here because we may have only written a partial
		// response.
		defer func() {
			_err := stream.Reset()
			_ = _err
		}()

		log.WithField("peer", stream.Conn().RemotePeer().String()).WithField("topic", string(stream.Protocol()))

		base, ok := p2p.RPCTopicMappings[baseTopic]
		if !ok {
			log.Errorf("Could not retrieve base message for topic %s", baseTopic)
			return
		}
		t := reflect.TypeOf(base)
		// Copy Base
		base = reflect.New(t)

		// since metadata requests do not have any data in the payload, we
		// do not decode anything.
		if baseTopic == p2p.RPCMetaDataTopicV1 || baseTopic == p2p.RPCMetaDataTopicV2 {
			if err := handle(context.Background(), base, stream); err != nil {
				if err != p2ptypes.ErrWrongForkDigestVersion {
					log.WithError(err).Debug("Could not handle p2p RPC")
				}
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
			if err := c.Encoding().DecodeWithMaxLength(stream, msg); err != nil {
				// Trace logs for goodbye errors
				logStreamErrors(err, topic)
				return
			}
			if err := handle(context.Background(), msg, stream); err != nil {
				if err != p2ptypes.ErrWrongForkDigestVersion {
					log.WithError(err).Debug("Could not handle p2p RPC")
				}
			}
		} else {
			nTyp := reflect.New(t)
			msg, ok := nTyp.Interface().(ssz.Unmarshaler)
			if !ok {
				log.Errorf("message of %T does not support marshaller interface", msg)
				return
			}
			if err := c.Encoding().DecodeWithMaxLength(stream, msg); err != nil {
				logStreamErrors(err, topic)
				return
			}
			if err := handle(context.Background(), nTyp.Elem().Interface(), stream); err != nil {
				if err != p2ptypes.ErrWrongForkDigestVersion {
					log.WithError(err).Debug("Could not handle p2p RPC")
				}
			}
		}
	})
}

func logStreamErrors(err error, topic string) {
	if strings.Contains(topic, p2p.RPCGoodByeTopicV1) {
		log.WithError(err).Trace("Could not decode goodbye stream message")
		return
	}
	log.WithError(err).Debug("Could not decode stream message")
}
