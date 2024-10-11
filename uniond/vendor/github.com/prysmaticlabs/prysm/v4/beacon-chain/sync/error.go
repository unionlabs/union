package sync

import (
	"bytes"
	"errors"

	libp2pcore "github.com/libp2p/go-libp2p/core"
	"github.com/libp2p/go-libp2p/core/network"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/encoder"
	"github.com/prysmaticlabs/prysm/v4/beacon-chain/p2p/types"
	"github.com/prysmaticlabs/prysm/v4/config/params"
	"github.com/sirupsen/logrus"
)

var ErrNoValidDigest = errors.New("no valid digest matched")
var ErrUnrecognizedVersion = errors.New("cannot determine context bytes for unrecognized object")

var responseCodeSuccess = byte(0x00)
var responseCodeInvalidRequest = byte(0x01)
var responseCodeServerError = byte(0x02)
var responseCodeResourceUnavailable = byte(0x03)

func (s *Service) generateErrorResponse(code byte, reason string) ([]byte, error) {
	return createErrorResponse(code, reason, s.cfg.p2p)
}

// ReadStatusCode response from a RPC stream.
func ReadStatusCode(stream network.Stream, encoding encoder.NetworkEncoding) (uint8, string, error) {
	// Set ttfb deadline.
	SetStreamReadDeadline(stream, params.BeaconConfig().TtfbTimeoutDuration())
	b := make([]byte, 1)
	_, err := stream.Read(b)
	if err != nil {
		return 0, "", err
	}

	if b[0] == responseCodeSuccess {
		// Set response deadline on a successful response code.
		SetStreamReadDeadline(stream, params.BeaconConfig().RespTimeoutDuration())

		return 0, "", nil
	}

	// Set response deadline, when reading error message.
	SetStreamReadDeadline(stream, params.BeaconConfig().RespTimeoutDuration())
	msg := &types.ErrorMessage{}
	if err := encoding.DecodeWithMaxLength(stream, msg); err != nil {
		return 0, "", err
	}

	return b[0], string(*msg), nil
}

func writeErrorResponseToStream(responseCode byte, reason string, stream libp2pcore.Stream, encoder p2p.EncodingProvider) {
	resp, err := createErrorResponse(responseCode, reason, encoder)
	if err != nil {
		log.WithError(err).Debug("Could not generate a response error")
	} else if _, err := stream.Write(resp); err != nil {
		log.WithError(err).Debugf("Could not write to stream")
	} else {
		// If sending the error message succeeded, close to send an EOF.
		closeStream(stream, log)
	}
}

func createErrorResponse(code byte, reason string, encoder p2p.EncodingProvider) ([]byte, error) {
	buf := bytes.NewBuffer([]byte{code})
	errMsg := types.ErrorMessage(reason)
	if _, err := encoder.Encoding().EncodeWithMaxLength(buf, &errMsg); err != nil {
		return nil, err
	}

	return buf.Bytes(), nil
}

// reads data from the stream without applying any timeouts.
func readStatusCodeNoDeadline(stream network.Stream, encoding encoder.NetworkEncoding) (uint8, string, error) {
	b := make([]byte, 1)
	_, err := stream.Read(b)
	if err != nil {
		return 0, "", err
	}

	if b[0] == responseCodeSuccess {
		return 0, "", nil
	}

	msg := &types.ErrorMessage{}
	if err := encoding.DecodeWithMaxLength(stream, msg); err != nil {
		return 0, "", err
	}

	return b[0], string(*msg), nil
}

// only returns true for errors that are valid (no resets or expectedEOF errors).
func isValidStreamError(err error) bool {
	// check the error message itself as well as libp2p doesn't currently
	// return the correct error type from Close{Read,Write,}.
	return err != nil && !errors.Is(err, network.ErrReset) && err.Error() != network.ErrReset.Error()
}

func closeStream(stream network.Stream, log *logrus.Entry) {
	if err := stream.Close(); isValidStreamError(err) {
		log.WithError(err).Debugf("Could not reset stream with protocol %s", stream.Protocol())
	}
}

func closeStreamAndWait(stream network.Stream, log *logrus.Entry) {
	if err := stream.CloseWrite(); err != nil {
		_err := stream.Reset()
		_ = _err
		if isValidStreamError(err) {
			log.WithError(err).Debugf("Could not reset stream with protocol %s", stream.Protocol())
		}
		return
	}
	// Wait for the remote side to respond.
	//
	// 1. On success, we expect to read an EOF (remote side received our
	//    response and closed the stream.
	// 2. On failure (e.g., disconnect), we expect to receive an error.
	// 3. If the remote side misbehaves, we may receive data.
	//
	// However, regardless of what happens, we just close the stream and
	// walk away. We only read to wait for a response, we close regardless.
	_, _err := stream.Read([]byte{0})
	_ = _err
	_err = stream.Close()
	_ = _err
}
