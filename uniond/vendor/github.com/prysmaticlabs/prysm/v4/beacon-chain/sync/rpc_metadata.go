package sync

import (
	"context"

	libp2pcore "github.com/libp2p/go-libp2p/core"
	"github.com/libp2p/go-libp2p/core/peer"
	"github.com/pkg/errors"
	"github.com/prysmaticlabs/go-bitfield"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/blockchain"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/core/signing"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/types"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/prysmaticlabs/prysm/v4/consensus-types/wrapper"
	"github.com/prysmaticlabs/prysm/v4/encoding/bytesutil"
	"github.com/prysmaticlabs/prysm/v4/network/forks"
	pb "github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1"
	"github.com/prysmaticlabs/prysm/v4/proto/prysm/v1alpha1/metadata"
	"github.com/prysmaticlabs/prysm/v4/runtime/version"
	"github.com/prysmaticlabs/prysm/v4/time/slots"
)

// metaDataHandler reads the incoming metadata rpc request from the peer.
func (s *Service) metaDataHandler(_ context.Context, _ interface{}, stream libp2pcore.Stream) error {
	SetRPCStreamDeadlines(stream)

	if err := s.rateLimiter.validateRequest(stream, 1); err != nil {
		return err
	}
	s.rateLimiter.add(stream, 1)

	if s.cfg.p2p.Metadata() == nil || s.cfg.p2p.Metadata().IsNil() {
		nilErr := errors.New("nil metadata stored for host")
		resp, err := s.generateErrorResponse(responseCodeServerError, types.ErrGeneric.Error())
		if err != nil {
			log.WithError(err).Debug("Could not generate a response error")
		} else if _, err := stream.Write(resp); err != nil {
			log.WithError(err).Debug("Could not write to stream")
		}
		return nilErr
	}
	_, _, streamVersion, err := p2p.TopicDeconstructor(string(stream.Protocol()))
	if err != nil {
		resp, genErr := s.generateErrorResponse(responseCodeServerError, types.ErrGeneric.Error())
		if genErr != nil {
			log.WithError(genErr).Debug("Could not generate a response error")
		} else if _, wErr := stream.Write(resp); wErr != nil {
			log.WithError(wErr).Debug("Could not write to stream")
		}
		return err
	}
	currMd := s.cfg.p2p.Metadata()
	switch streamVersion {
	case p2p.SchemaVersionV1:
		// We have a v1 metadata object saved locally, so we
		// convert it back to a v0 metadata object.
		if currMd.Version() != version.Phase0 {
			currMd = wrapper.WrappedMetadataV0(
				&pb.MetaDataV0{
					Attnets:   currMd.AttnetsBitfield(),
					SeqNumber: currMd.SequenceNumber(),
				})
		}
	case p2p.SchemaVersionV2:
		// We have a v0 metadata object saved locally, so we
		// convert it to a v1 metadata object.
		if currMd.Version() != version.Altair {
			currMd = wrapper.WrappedMetadataV1(
				&pb.MetaDataV1{
					Attnets:   currMd.AttnetsBitfield(),
					SeqNumber: currMd.SequenceNumber(),
					Syncnets:  bitfield.Bitvector4{byte(0x00)},
				})
		}
	}
	if _, err := stream.Write([]byte{responseCodeSuccess}); err != nil {
		return err
	}
	_, err = s.cfg.p2p.Encoding().EncodeWithMaxLength(stream, currMd)
	if err != nil {
		return err
	}
	closeStream(stream, log)
	return nil
}

func (s *Service) sendMetaDataRequest(ctx context.Context, id peer.ID) (metadata.Metadata, error) {
	ctx, cancel := context.WithTimeout(ctx, respTimeout)
	defer cancel()

	topic, err := p2p.TopicFromMessage(p2p.MetadataMessageName, slots.ToEpoch(s.cfg.clock.CurrentSlot()))
	if err != nil {
		return nil, err
	}
	stream, err := s.cfg.p2p.Send(ctx, new(interface{}), topic, id)
	if err != nil {
		return nil, err
	}
	defer closeStream(stream, log)
	code, errMsg, err := ReadStatusCode(stream, s.cfg.p2p.Encoding())
	if err != nil {
		s.cfg.p2p.Peers().Scorers().BadResponsesScorer().Increment(stream.Conn().RemotePeer())
		return nil, err
	}
	if code != 0 {
		s.cfg.p2p.Peers().Scorers().BadResponsesScorer().Increment(stream.Conn().RemotePeer())
		return nil, errors.New(errMsg)
	}
	valRoot := s.cfg.clock.GenesisValidatorsRoot()
	rpcCtx, err := forks.ForkDigestFromEpoch(slots.ToEpoch(s.cfg.clock.CurrentSlot()), valRoot[:])
	if err != nil {
		return nil, err
	}
	msg, err := extractMetaDataType(rpcCtx[:], s.cfg.clock)
	if err != nil {
		return nil, err
	}
	// Defensive check to ensure valid objects are being sent.
	topicVersion := ""
	switch msg.Version() {
	case version.Phase0:
		topicVersion = p2p.SchemaVersionV1
	case version.Altair:
		topicVersion = p2p.SchemaVersionV2
	}
	if err := validateVersion(topicVersion, stream); err != nil {
		return nil, err
	}
	if err := s.cfg.p2p.Encoding().DecodeWithMaxLength(stream, msg); err != nil {
		s.cfg.p2p.Peers().Scorers().BadResponsesScorer().Increment(stream.Conn().RemotePeer())
		return nil, err
	}
	return msg, nil
}

func extractMetaDataType(digest []byte, tor blockchain.TemporalOracle) (metadata.Metadata, error) {
	if len(digest) == 0 {
		mdFunc, ok := types.MetaDataMap[bytesutil.ToBytes4(params.BeaconConfig().GenesisForkVersion)]
		if !ok {
			return nil, errors.New("no metadata type exists for the genesis fork version.")
		}
		return mdFunc(), nil
	}
	if len(digest) != forkDigestLength {
		return nil, errors.Errorf("invalid digest returned, wanted a length of %d but received %d", forkDigestLength, len(digest))
	}
	vRoot := tor.GenesisValidatorsRoot()
	for k, mdFunc := range types.MetaDataMap {
		rDigest, err := signing.ComputeForkDigest(k[:], vRoot[:])
		if err != nil {
			return nil, err
		}
		if rDigest == bytesutil.ToBytes4(digest) {
			return mdFunc(), nil
		}
	}
	return nil, errors.Wrapf(ErrNoValidDigest, "could not extract metadata type, saw digest=%#x, genesis=%v, vr=%#x", digest, tor.GenesisTime(), tor.GenesisValidatorsRoot())
}
