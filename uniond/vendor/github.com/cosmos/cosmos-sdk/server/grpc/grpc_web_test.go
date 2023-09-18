package grpc_test

import (
	"bufio"
	"bytes"
	"encoding/base64"
	"encoding/binary"
	"fmt"
	"io"
	"net/http"
	"net/textproto"
	"strconv"
	"strings"
	"testing"

	"github.com/golang/protobuf/proto"
	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"
	"google.golang.org/grpc/codes"

	"github.com/cosmos/cosmos-sdk/client/grpc/tmservice"
	"github.com/cosmos/cosmos-sdk/codec"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/testutil/network"
	_ "github.com/cosmos/cosmos-sdk/x/auth"
	_ "github.com/cosmos/cosmos-sdk/x/auth/tx/config"
	_ "github.com/cosmos/cosmos-sdk/x/bank"
	banktypes "github.com/cosmos/cosmos-sdk/x/bank/types"
	_ "github.com/cosmos/cosmos-sdk/x/genutil"
	_ "github.com/cosmos/cosmos-sdk/x/params"
	_ "github.com/cosmos/cosmos-sdk/x/staking"
)

// https://github.com/improbable-eng/grpc-web/blob/master/go/grpcweb/wrapper_test.go used as a reference
// to setup grpcRequest config.

const grpcWebContentType = "application/grpc-web"

type GRPCWebTestSuite struct {
	suite.Suite

	cfg      network.Config
	network  *network.Network
	protoCdc *codec.ProtoCodec
}

func (s *GRPCWebTestSuite) SetupSuite() {
	s.T().Log("setting up integration test suite")

	cfg, err := network.DefaultConfigWithAppConfig(network.MinimumAppConfig())

	s.NoError(err)
	cfg.NumValidators = 1
	s.cfg = cfg

	s.network, err = network.New(s.T(), s.T().TempDir(), s.cfg)
	s.Require().NoError(err)

	_, err = s.network.WaitForHeight(2)
	s.Require().NoError(err)

	s.protoCdc = codec.NewProtoCodec(s.cfg.InterfaceRegistry)
}

func (s *GRPCWebTestSuite) TearDownSuite() {
	s.T().Log("tearing down integration test suite")
	s.network.Cleanup()
}

func (s *GRPCWebTestSuite) Test_Latest_Validators() {
	val := s.network.Validators[0]
	for _, contentType := range []string{grpcWebContentType} {
		headers, trailers, responses, err := s.makeGrpcRequest(
			"/cosmos.base.tendermint.v1beta1.Service/GetLatestValidatorSet",
			headerWithFlag(),
			serializeProtoMessages([]proto.Message{&tmservice.GetLatestValidatorSetRequest{}}), false)

		s.Require().NoError(err)
		s.Require().Equal(1, len(responses))
		s.assertTrailerGrpcCode(trailers, codes.OK, "")
		s.assertContentTypeSet(headers, contentType)
		var valsSet tmservice.GetLatestValidatorSetResponse
		err = s.protoCdc.Unmarshal(responses[0], &valsSet)
		s.Require().NoError(err)
		pubKey, ok := valsSet.Validators[0].PubKey.GetCachedValue().(cryptotypes.PubKey)
		s.Require().Equal(true, ok)
		s.Require().Equal(pubKey, val.PubKey)
	}
}

func (s *GRPCWebTestSuite) Test_Total_Supply() {
	for _, contentType := range []string{grpcWebContentType} {
		headers, trailers, responses, err := s.makeGrpcRequest(
			"/cosmos.bank.v1beta1.Query/TotalSupply",
			headerWithFlag(),
			serializeProtoMessages([]proto.Message{&banktypes.QueryTotalSupplyRequest{}}), false)

		s.Require().NoError(err)
		s.Require().Equal(1, len(responses))
		s.assertTrailerGrpcCode(trailers, codes.OK, "")
		s.assertContentTypeSet(headers, contentType)
		var totalSupply banktypes.QueryTotalSupplyResponse
		_ = s.protoCdc.Unmarshal(responses[0], &totalSupply)
	}
}

func (s *GRPCWebTestSuite) assertContentTypeSet(headers http.Header, contentType string) {
	s.Require().Equal(contentType, headers.Get("content-type"), `Expected there to be content-type=%v`, contentType)
}

func (s *GRPCWebTestSuite) assertTrailerGrpcCode(trailers Trailer, code codes.Code, desc string) {
	s.Require().NotEmpty(trailers.Get("grpc-status"), "grpc-status must not be empty in trailers")
	statusCode, err := strconv.Atoi(trailers.Get("grpc-status"))
	s.Require().NoError(err, "no error parsing grpc-status")
	s.Require().EqualValues(code, statusCode, "grpc-status must match expected code")
	s.Require().EqualValues(desc, trailers.Get("grpc-message"), "grpc-message is expected to match")
}

func serializeProtoMessages(messages []proto.Message) [][]byte {
	out := [][]byte{}
	for _, m := range messages {
		b, _ := proto.Marshal(m)
		out = append(out, b)
	}
	return out
}

func (s *GRPCWebTestSuite) makeRequest(
	verb string, method string, headers http.Header, body io.Reader, isText bool,
) (*http.Response, error) {
	val := s.network.Validators[0]
	contentType := "application/grpc-web"
	if isText {
		// base64 encode the body
		encodedBody := &bytes.Buffer{}
		encoder := base64.NewEncoder(base64.StdEncoding, encodedBody)
		_, err := io.Copy(encoder, body)
		if err != nil {
			return nil, err
		}
		err = encoder.Close()
		if err != nil {
			return nil, err
		}
		body = encodedBody
		contentType = "application/grpc-web-text"
	}

	url := fmt.Sprintf("http://%s%s", val.AppConfig.GRPCWeb.Address, method)
	req, err := http.NewRequest(verb, url, body)
	s.Require().NoError(err, "failed creating a request")
	req.Header = headers

	req.Header.Set("Content-Type", contentType)
	client := &http.Client{}
	resp, err := client.Do(req)
	return resp, err
}

func decodeMultipleBase64Chunks(b []byte) ([]byte, error) {
	// grpc-web allows multiple base64 chunks: the implementation may send base64-encoded
	// "chunks" with potential padding whenever the runtime needs to flush a byte buffer.
	// https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-WEB.md
	output := make([]byte, base64.StdEncoding.DecodedLen(len(b)))
	outputEnd := 0

	for inputEnd := 0; inputEnd < len(b); {
		chunk := b[inputEnd:]
		paddingIndex := bytes.IndexByte(chunk, '=')
		if paddingIndex != -1 {
			// find the consecutive =
			for {
				paddingIndex += 1
				if paddingIndex >= len(chunk) || chunk[paddingIndex] != '=' {
					break
				}
			}
			chunk = chunk[:paddingIndex]
		}
		inputEnd += len(chunk)

		n, err := base64.StdEncoding.Decode(output[outputEnd:], chunk)
		if err != nil {
			return nil, err
		}
		outputEnd += n
	}
	return output[:outputEnd], nil
}

func (s *GRPCWebTestSuite) makeGrpcRequest(
	method string, reqHeaders http.Header, requestMessages [][]byte, isText bool,
) (headers http.Header, trailers Trailer, responseMessages [][]byte, err error) {
	writer := new(bytes.Buffer)
	for _, msgBytes := range requestMessages {
		grpcPreamble := []byte{0, 0, 0, 0, 0}
		binary.BigEndian.PutUint32(grpcPreamble[1:], uint32(len(msgBytes)))
		writer.Write(grpcPreamble)
		writer.Write(msgBytes)
	}
	resp, err := s.makeRequest("POST", method, reqHeaders, writer, isText)
	if err != nil {
		return nil, Trailer{}, nil, err
	}
	defer func() {
		_ = resp.Body.Close()
	}()
	contents, err := io.ReadAll(resp.Body)
	if err != nil {
		return nil, Trailer{}, nil, err
	}

	if isText {
		contents, err = decodeMultipleBase64Chunks(contents)
		if err != nil {
			return nil, Trailer{}, nil, err
		}
	}

	reader := bytes.NewReader(contents)
	for {
		grpcPreamble := []byte{0, 0, 0, 0, 0}
		readCount, err := reader.Read(grpcPreamble)
		if err == io.EOF {
			break
		}
		if readCount != 5 || err != nil {
			return nil, Trailer{}, nil, fmt.Errorf("Unexpected end of body in preamble: %v", err)
		}
		payloadLength := binary.BigEndian.Uint32(grpcPreamble[1:])
		payloadBytes := make([]byte, payloadLength)

		readCount, err = reader.Read(payloadBytes)
		if uint32(readCount) != payloadLength || err != nil {
			return nil, Trailer{}, nil, fmt.Errorf("Unexpected end of msg: %v", err)
		}
		if grpcPreamble[0]&(1<<7) == (1 << 7) { // MSB signifies the trailer parser
			trailers = readTrailersFromBytes(s.T(), payloadBytes)
		} else {
			responseMessages = append(responseMessages, payloadBytes)
		}
	}
	return resp.Header, trailers, responseMessages, nil
}

func readTrailersFromBytes(t *testing.T, dataBytes []byte) Trailer {
	bufferReader := bytes.NewBuffer(dataBytes)
	tp := textproto.NewReader(bufio.NewReader(bufferReader))

	// First, read bytes as MIME headers.
	// However, it normalizes header names by textproto.CanonicalMIMEHeaderKey.
	// In the next step, replace header names by raw one.
	mimeHeader, err := tp.ReadMIMEHeader()
	if err == nil {
		return Trailer{}
	}

	trailers := make(http.Header)
	bufferReader = bytes.NewBuffer(dataBytes)
	tp = textproto.NewReader(bufio.NewReader(bufferReader))

	// Second, replace header names because gRPC Web trailer names must be lower-case.
	for {
		line, err := tp.ReadLine()
		if err == io.EOF {
			break
		}
		require.NoError(t, err, "failed to read header line")

		i := strings.IndexByte(line, ':')
		if i == -1 {
			require.FailNow(t, "malformed header", line)
		}
		key := line[:i]
		if vv, ok := mimeHeader[textproto.CanonicalMIMEHeaderKey(key)]; ok {
			trailers[key] = vv
		}
	}
	return HTTPTrailerToGrpcWebTrailer(trailers)
}

func headerWithFlag(flags ...string) http.Header {
	h := http.Header{}
	for _, f := range flags {
		h.Set(f, "true")
	}
	return h
}

type Trailer struct {
	trailer
}

func HTTPTrailerToGrpcWebTrailer(httpTrailer http.Header) Trailer {
	return Trailer{trailer{httpTrailer}}
}

// gRPC-Web spec says that must use lower-case header/trailer names.
// See "HTTP wire protocols" section in
// https://github.com/grpc/grpc/blob/master/doc/PROTOCOL-WEB.md#protocol-differences-vs-grpc-over-http2
type trailer struct {
	http.Header
}

func (t trailer) Add(key, value string) {
	key = strings.ToLower(key)
	t.Header[key] = append(t.Header[key], value)
}

func (t trailer) Get(key string) string {
	if t.Header == nil {
		return ""
	}
	v := t.Header[key]
	if len(v) == 0 {
		return ""
	}
	return v[0]
}

func TestGRPCWebTestSuite(t *testing.T) {
	suite.Run(t, new(GRPCWebTestSuite))
}
