package keys

import (
	"context"
	"fmt"
	"strings"
	"testing"

	design99keyring "github.com/99designs/keyring"
	"github.com/stretchr/testify/suite"

	"github.com/cosmos/cosmos-sdk/client"
	"github.com/cosmos/cosmos-sdk/client/flags"
	"github.com/cosmos/cosmos-sdk/codec"
	addresscodec "github.com/cosmos/cosmos-sdk/codec/address"
	codectestutil "github.com/cosmos/cosmos-sdk/codec/testutil"
	"github.com/cosmos/cosmos-sdk/crypto/keyring"
	"github.com/cosmos/cosmos-sdk/crypto/keys/multisig"
	"github.com/cosmos/cosmos-sdk/crypto/keys/secp256k1"
	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	"github.com/cosmos/cosmos-sdk/testutil"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
)

type setter interface {
	SetItem(item design99keyring.Item) error
}

type MigrateTestSuite struct {
	suite.Suite

	dir     string
	appName string
	cdc     codec.Codec
	priv    cryptotypes.PrivKey
	pub     cryptotypes.PubKey
}

func TestMigrateTestSuite(t *testing.T) {
	suite.Run(t, new(MigrateTestSuite))
}

func (s *MigrateTestSuite) SetupSuite() {
	s.dir = s.T().TempDir()
	s.cdc = moduletestutil.MakeTestEncodingConfig(codectestutil.CodecOptions{}).Codec
	s.appName = "cosmos"
	s.priv = cryptotypes.PrivKey(secp256k1.GenPrivKey())
	s.pub = s.priv.PubKey()
}

func (s *MigrateTestSuite) Test_runListAndShowCmd() {
	// adding LegacyInfo item into keyring
	multi := multisig.NewLegacyAminoPubKey(
		1, []cryptotypes.PubKey{
			s.pub,
		},
	)
	legacyMultiInfo, err := keyring.NewLegacyMultiInfo(s.appName, multi)
	s.Require().NoError(err)
	serializedLegacyMultiInfo := keyring.MarshalInfo(legacyMultiInfo)

	item := design99keyring.Item{
		Key:         s.appName + ".info",
		Data:        serializedLegacyMultiInfo,
		Description: "SDK keyring version",
	}

	// run test simd keys list - to see that the migrated key is there
	cmd := ListKeysCmd()
	cmd.Flags().AddFlagSet(Commands().PersistentFlags())

	mockIn := testutil.ApplyMockIODiscardOutErr(cmd)
	kb, err := keyring.New(s.appName, keyring.BackendTest, s.dir, mockIn, s.cdc)
	s.Require().NoError(err)

	setter, ok := kb.(setter)
	s.Require().True(ok)
	s.Require().NoError(setter.SetItem(item))

	clientCtx := client.Context{}.
		WithKeyring(kb).
		WithAddressCodec(addresscodec.NewBech32Codec("cosmos")).
		WithValidatorAddressCodec(addresscodec.NewBech32Codec("cosmosvaloper")).
		WithConsensusAddressCodec(addresscodec.NewBech32Codec("cosmosvalcons"))

	ctx := context.WithValue(context.Background(), client.ClientContextKey, &clientCtx)

	cmd.SetArgs([]string{
		fmt.Sprintf("--%s=%s", flags.FlagKeyringDir, s.dir),
		fmt.Sprintf("--%s=false", flagListNames),
	})

	s.Require().NoError(cmd.ExecuteContext(ctx))

	// simd show n1 - to see that the migration worked
	cmd = ShowKeysCmd()
	cmd.SetArgs([]string{s.appName})
	clientCtx = clientCtx.WithCodec(s.cdc)
	ctx = context.WithValue(context.Background(), client.ClientContextKey, &clientCtx)
	s.Require().NoError(cmd.ExecuteContext(ctx))
}

func (s *MigrateTestSuite) Test_runMigrateCmdRecord() {
	k, err := keyring.NewLocalRecord("test record", s.priv, s.pub)
	s.Require().NoError(err)
	serializedRecord, err := s.cdc.Marshal(k)
	s.Require().NoError(err)

	item := design99keyring.Item{
		Key:         s.appName,
		Data:        serializedRecord,
		Description: "SDK kerying version",
	}

	cmd := MigrateCommand()
	mockIn := strings.NewReader("")
	kb, err := keyring.New(s.appName, keyring.BackendTest, s.dir, mockIn, s.cdc)
	s.Require().NoError(err)

	setter, ok := kb.(setter)
	s.Require().True(ok)
	s.Require().NoError(setter.SetItem(item))

	clientCtx := client.Context{}.WithKeyring(kb)
	ctx := context.WithValue(context.Background(), client.ClientContextKey, &clientCtx)
	s.Require().NoError(cmd.ExecuteContext(ctx))
}

func (s *MigrateTestSuite) Test_runMigrateCmdLegacyMultiInfo() {
	// adding LegacyInfo item into keyring
	multi := multisig.NewLegacyAminoPubKey(
		1, []cryptotypes.PubKey{
			s.pub,
		},
	)

	legacyMultiInfo, err := keyring.NewLegacyMultiInfo(s.appName, multi)
	s.Require().NoError(err)
	serializedLegacyMultiInfo := keyring.MarshalInfo(legacyMultiInfo)

	item := design99keyring.Item{
		Key:         s.appName,
		Data:        serializedLegacyMultiInfo,
		Description: "SDK kerying version",
	}

	cmd := MigrateCommand()
	mockIn := testutil.ApplyMockIODiscardOutErr(cmd)

	kb, err := keyring.New(s.appName, keyring.BackendTest, s.dir, mockIn, s.cdc)
	s.Require().NoError(err)

	setter, ok := kb.(setter)
	s.Require().True(ok)
	s.Require().NoError(setter.SetItem(item))

	clientCtx := client.Context{}.WithKeyring(kb)
	ctx := context.WithValue(context.Background(), client.ClientContextKey, &clientCtx)
	s.Require().NoError(cmd.ExecuteContext(ctx))
}
