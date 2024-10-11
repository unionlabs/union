package keyring

import (
	"strings"
	"testing"

	"github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/codec"
	codectypes "github.com/cosmos/cosmos-sdk/codec/types"
	"github.com/cosmos/cosmos-sdk/crypto/hd"
	"github.com/cosmos/cosmos-sdk/crypto/keys/ed25519"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
)

type RecordTestSuite struct {
	suite.Suite

	appName string
	cdc     codec.Codec
	priv    cryptotypes.PrivKey
	pub     cryptotypes.PubKey
}

func (s *RecordTestSuite) SetupSuite() {
	s.appName = "cosmos"
	s.cdc = getCodec()
	s.priv = cryptotypes.PrivKey(ed25519.GenPrivKey())
	s.pub = s.priv.PubKey()
}

func (s *RecordTestSuite) TestOfflineRecordMarshaling() {
	k, err := NewOfflineRecord("testrecord", s.pub)
	s.Require().NoError(err)

	bz, err := s.cdc.Marshal(k)
	s.Require().NoError(err)

	var k2 Record
	s.Require().NoError(s.cdc.Unmarshal(bz, &k2))
	s.Require().Equal(k.Name, k2.Name)
	s.Require().True(k.PubKey.Equal(k2.PubKey))

	pk2, err := k2.GetPubKey()
	s.Require().NoError(err)
	s.Require().True(s.pub.Equals(pk2))
}

func (s *RecordTestSuite) TestLocalRecordMarshaling() {
	dir := s.T().TempDir()
	mockIn := strings.NewReader("")

	kb, err := New(s.appName, BackendTest, dir, mockIn, s.cdc)
	s.Require().NoError(err)

	k, err := NewLocalRecord("testrecord", s.priv, s.pub)
	s.Require().NoError(err)

	ks, ok := kb.(keystore)
	s.Require().True(ok)

	bz, err := ks.cdc.Marshal(k)
	s.Require().NoError(err)

	k2, err := ks.protoUnmarshalRecord(bz)
	s.Require().NoError(err)
	s.Require().Equal(k.Name, k2.Name)
	// not sure if this will work -- we can remove this line, the later check is better.
	s.Require().True(k.PubKey.Equal(k2.PubKey))

	pub2, err := k2.GetPubKey()
	s.Require().NoError(err)
	s.Require().True(s.pub.Equals(pub2))

	localRecord2 := k2.GetLocal()
	s.Require().NotNil(localRecord2)
	anyPrivKey, err := codectypes.NewAnyWithValue(s.priv)
	s.Require().NoError(err)
	s.Require().Equal(localRecord2.PrivKey, anyPrivKey)
}

func (s *RecordTestSuite) TestLedgerRecordMarshaling() {
	dir := s.T().TempDir()
	mockIn := strings.NewReader("")

	kb, err := New(s.appName, BackendTest, dir, mockIn, s.cdc)
	s.Require().NoError(err)

	path := hd.NewFundraiserParams(4, 12345, 57)
	k, err := NewLedgerRecord("testrecord", s.pub, path)
	s.Require().NoError(err)

	ks, ok := kb.(keystore)
	s.Require().True(ok)

	bz, err := ks.cdc.Marshal(k)
	s.Require().NoError(err)

	k2, err := ks.protoUnmarshalRecord(bz)
	s.Require().NoError(err)
	s.Require().Equal(k.Name, k2.Name)
	// not sure if this will work -- we can remove this line, the later check is better.
	s.Require().True(k.PubKey.Equal(k2.PubKey))

	pub2, err := k2.GetPubKey()
	s.Require().NoError(err)
	s.Require().True(s.pub.Equals(pub2))

	ledgerRecord2 := k2.GetLedger()
	s.Require().NotNil(ledgerRecord2)
	s.Require().Nil(k2.GetLocal())

	s.Require().Equal(ledgerRecord2.Path.String(), path.String())
}

func (s *RecordTestSuite) TestExtractPrivKeyFromLocalRecord() {
	// use proto serialize
	k, err := NewLocalRecord("testrecord", s.priv, s.pub)
	s.Require().NoError(err)

	privKey2, err := extractPrivKeyFromRecord(k)
	s.Require().NoError(err)
	s.Require().True(privKey2.Equals(s.priv))
}

func (s *RecordTestSuite) TestExtractPrivKeyFromOfflineRecord() {
	k, err := NewOfflineRecord("testrecord", s.pub)
	s.Require().NoError(err)

	privKey2, err := extractPrivKeyFromRecord(k)
	s.Require().Error(err)
	s.Require().Nil(privKey2)
}

func TestRecordTestSuite(t *testing.T) {
	suite.Run(t, new(RecordTestSuite))
}
