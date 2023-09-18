package mempool_test

import (
	"fmt"
	"math/rand"
	"testing"

	"github.com/cometbft/cometbft/libs/log"
	tmproto "github.com/cometbft/cometbft/proto/tendermint/types"
	"github.com/stretchr/testify/require"
	"github.com/stretchr/testify/suite"

	cryptotypes "github.com/cosmos/cosmos-sdk/crypto/types"
	sdk "github.com/cosmos/cosmos-sdk/types"
	"github.com/cosmos/cosmos-sdk/types/mempool"
	moduletestutil "github.com/cosmos/cosmos-sdk/types/module/testutil"
	simtypes "github.com/cosmos/cosmos-sdk/types/simulation"
	txsigning "github.com/cosmos/cosmos-sdk/types/tx/signing"
	"github.com/cosmos/cosmos-sdk/x/auth/signing"
	"github.com/cosmos/cosmos-sdk/x/distribution"
	"github.com/cosmos/cosmos-sdk/x/gov"
)

// testPubKey is a dummy implementation of PubKey used for testing.
type testPubKey struct {
	address sdk.AccAddress
}

func (t testPubKey) Reset() { panic("not implemented") }

func (t testPubKey) String() string { panic("not implemented") }

func (t testPubKey) ProtoMessage() { panic("not implemented") }

func (t testPubKey) Address() cryptotypes.Address { return t.address.Bytes() }

func (t testPubKey) Bytes() []byte { panic("not implemented") }

func (t testPubKey) VerifySignature(msg []byte, sig []byte) bool { panic("not implemented") }

func (t testPubKey) Equals(key cryptotypes.PubKey) bool { panic("not implemented") }

func (t testPubKey) Type() string { panic("not implemented") }

// testTx is a dummy implementation of Tx used for testing.
type testTx struct {
	id       int
	priority int64
	nonce    uint64
	address  sdk.AccAddress
	// useful for debugging
	strAddress string
}

func (tx testTx) GetSigners() []sdk.AccAddress { panic("not implemented") }

func (tx testTx) GetPubKeys() ([]cryptotypes.PubKey, error) { panic("not implemented") }

func (tx testTx) GetSignaturesV2() (res []txsigning.SignatureV2, err error) {
	res = append(res, txsigning.SignatureV2{
		PubKey:   testPubKey{address: tx.address},
		Data:     nil,
		Sequence: tx.nonce,
	})

	return res, nil
}

var (
	_ sdk.Tx                  = (*testTx)(nil)
	_ signing.SigVerifiableTx = (*testTx)(nil)
	_ cryptotypes.PubKey      = (*testPubKey)(nil)
)

func (tx testTx) GetMsgs() []sdk.Msg { return nil }

func (tx testTx) ValidateBasic() error { return nil }

func (tx testTx) String() string {
	return fmt.Sprintf("tx a: %s, p: %d, n: %d", tx.address, tx.priority, tx.nonce)
}

type sigErrTx struct {
	getSigs func() ([]txsigning.SignatureV2, error)
}

func (_ sigErrTx) Size() int64 { return 0 }

func (_ sigErrTx) GetMsgs() []sdk.Msg { return nil }

func (_ sigErrTx) ValidateBasic() error { return nil }

func (_ sigErrTx) GetSigners() []sdk.AccAddress { return nil }

func (_ sigErrTx) GetPubKeys() ([]cryptotypes.PubKey, error) { return nil, nil }

func (t sigErrTx) GetSignaturesV2() ([]txsigning.SignatureV2, error) { return t.getSigs() }

type txSpec struct {
	i int
	p int
	n int
	a sdk.AccAddress
}

func (tx txSpec) String() string {
	return fmt.Sprintf("[tx i: %d, a: %s, p: %d, n: %d]", tx.i, tx.a, tx.p, tx.n)
}

func fetchTxs(iterator mempool.Iterator, maxBytes int64) []sdk.Tx {
	const txSize = 1
	var (
		txs      []sdk.Tx
		numBytes int64
	)
	for iterator != nil {
		if numBytes += txSize; numBytes > maxBytes {
			break
		}
		txs = append(txs, iterator.Tx())
		i := iterator.Next()
		iterator = i
	}
	return txs
}

func (s *MempoolTestSuite) TestDefaultMempool() {
	t := s.T()
	ctx := sdk.NewContext(nil, tmproto.Header{}, false, log.NewNopLogger())
	accounts := simtypes.RandomAccounts(rand.New(rand.NewSource(0)), 10)
	txCount := 1000
	var txs []testTx

	for i := 0; i < txCount; i++ {
		acc := accounts[i%len(accounts)]
		tx := testTx{
			nonce:    0,
			address:  acc.Address,
			priority: rand.Int63(),
		}
		txs = append(txs, tx)
	}

	// empty mempool behavior
	require.Equal(t, 0, s.mempool.CountTx())
	itr := s.mempool.Select(ctx, nil)
	require.Nil(t, itr)

	// same sender-nonce just overwrites a tx
	for _, tx := range txs {
		ctx = ctx.WithPriority(tx.priority)
		err := s.mempool.Insert(ctx, tx)
		require.NoError(t, err)
	}
	require.Equal(t, len(accounts), s.mempool.CountTx())

	// distinct sender-nonce should not overwrite a tx
	s.resetMempool()
	for i, tx := range txs {
		tx.nonce = uint64(i)
		err := s.mempool.Insert(ctx, tx)
		require.NoError(t, err)
	}
	require.Equal(t, txCount, s.mempool.CountTx())

	itr = s.mempool.Select(ctx, nil)
	sel := fetchTxs(itr, 13)
	require.Equal(t, 13, len(sel))

	// a tx which does not implement SigVerifiableTx should not be inserted
	tx := &sigErrTx{getSigs: func() ([]txsigning.SignatureV2, error) {
		return nil, fmt.Errorf("error")
	}}
	require.Error(t, s.mempool.Insert(ctx, tx))
	require.Error(t, s.mempool.Remove(tx))
	tx.getSigs = func() ([]txsigning.SignatureV2, error) {
		return nil, nil
	}
	require.Error(t, s.mempool.Insert(ctx, tx))
	require.Error(t, s.mempool.Remove(tx))

	// removing a tx not in the mempool should error
	s.resetMempool()
	require.NoError(t, s.mempool.Insert(ctx, txs[0]))
	require.ErrorIs(t, s.mempool.Remove(txs[1]), mempool.ErrTxNotFound)

	// inserting a tx with a different priority should overwrite the old tx
	newPriorityTx := testTx{
		address:  txs[0].address,
		priority: txs[0].priority + 1,
		nonce:    txs[0].nonce,
	}
	require.NoError(t, s.mempool.Insert(ctx, newPriorityTx))
	require.Equal(t, 1, s.mempool.CountTx())
}

type MempoolTestSuite struct {
	suite.Suite
	numTxs      int
	numAccounts int
	iterations  int
	mempool     mempool.Mempool
}

func (s *MempoolTestSuite) resetMempool() {
	s.iterations = 0
	s.mempool = mempool.NewSenderNonceMempool()
}

func (s *MempoolTestSuite) SetupTest() {
	s.numTxs = 1000
	s.numAccounts = 100
	s.resetMempool()
}

func TestMempoolTestSuite(t *testing.T) {
	suite.Run(t, new(MempoolTestSuite))
}

func (s *MempoolTestSuite) TestSampleTxs() {
	ctxt := sdk.NewContext(nil, tmproto.Header{}, false, log.NewNopLogger())
	t := s.T()
	s.resetMempool()
	mp := s.mempool
	delegatorTx, err := unmarshalTx(msgWithdrawDelegatorReward)

	require.NoError(t, err)
	require.NoError(t, mp.Insert(ctxt, delegatorTx))
	require.Equal(t, 1, mp.CountTx())

	proposalTx, err := unmarshalTx(msgMultiSigMsgSubmitProposal)
	require.NoError(t, err)
	require.NoError(t, mp.Insert(ctxt, proposalTx))
	require.Equal(t, 2, mp.CountTx())
}

func unmarshalTx(txBytes []byte) (sdk.Tx, error) {
	cfg := moduletestutil.MakeTestEncodingConfig(distribution.AppModuleBasic{}, gov.AppModuleBasic{})
	return cfg.TxConfig.TxJSONDecoder()(txBytes)
}

var (
	msgWithdrawDelegatorReward   = []byte("{\"body\":{\"messages\":[{\"@type\":\"\\/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward\",\"delegator_address\":\"cosmos16w6g0whmw703t8h2m9qmq2fd9dwaw6fjszzjsw\",\"validator_address\":\"cosmosvaloper1lzhlnpahvznwfv4jmay2tgaha5kmz5qxerarrl\"},{\"@type\":\"\\/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward\",\"delegator_address\":\"cosmos16w6g0whmw703t8h2m9qmq2fd9dwaw6fjszzjsw\",\"validator_address\":\"cosmosvaloper1sjllsnramtg3ewxqwwrwjxfgc4n4ef9u2lcnj0\"},{\"@type\":\"\\/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward\",\"delegator_address\":\"cosmos16w6g0whmw703t8h2m9qmq2fd9dwaw6fjszzjsw\",\"validator_address\":\"cosmosvaloper196ax4vc0lwpxndu9dyhvca7jhxp70rmcvrj90c\"},{\"@type\":\"\\/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward\",\"delegator_address\":\"cosmos16w6g0whmw703t8h2m9qmq2fd9dwaw6fjszzjsw\",\"validator_address\":\"cosmosvaloper1k2d9ed9vgfuk2m58a2d80q9u6qljkh4vfaqjfq\"},{\"@type\":\"\\/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward\",\"delegator_address\":\"cosmos16w6g0whmw703t8h2m9qmq2fd9dwaw6fjszzjsw\",\"validator_address\":\"cosmosvaloper1vygmh344ldv9qefss9ek7ggsnxparljlmj56q5\"},{\"@type\":\"\\/cosmos.distribution.v1beta1.MsgWithdrawDelegatorReward\",\"delegator_address\":\"cosmos16w6g0whmw703t8h2m9qmq2fd9dwaw6fjszzjsw\",\"validator_address\":\"cosmosvaloper1ej2es5fjztqjcd4pwa0zyvaevtjd2y5wxxp9gd\"}],\"memo\":\"\",\"timeout_height\":\"0\",\"extension_options\":[],\"non_critical_extension_options\":[]},\"auth_info\":{\"signer_infos\":[{\"public_key\":{\"@type\":\"\\/cosmos.crypto.secp256k1.PubKey\",\"key\":\"AmbXAy10a0SerEefTYQzqyGQdX5kiTEWJZ1PZKX1oswX\"},\"mode_info\":{\"single\":{\"mode\":\"SIGN_MODE_LEGACY_AMINO_JSON\"}},\"sequence\":\"119\"}],\"fee\":{\"amount\":[{\"denom\":\"uatom\",\"amount\":\"15968\"}],\"gas_limit\":\"638717\",\"payer\":\"\",\"granter\":\"\"}},\"signatures\":[\"ji+inUo4xGlN9piRQLdLCeJWa7irwnqzrMVPcmzJyG5y6NPc+ZuNaIc3uvk5NLDJytRB8AHX0GqNETR\\/Q8fz4Q==\"]}")
	msgMultiSigMsgSubmitProposal = []byte("{\"body\":{\"messages\":[{\"@type\":\"\\/cosmos.gov.v1beta1.MsgSubmitProposal\",\"content\":{\"@type\":\"\\/cosmos.distribution.v1beta1.CommunityPoolSpendProposal\",\"title\":\"ATOM \\ud83e\\udd1d Osmosis:  Allocate Community Pool to ATOM Liquidity Incentives\",\"description\":\"ATOMs should be the base money of Cosmos, just like ETH is the base money of the entire Ethereum DeFi ecosystem. ATOM is currently well positioned to play this role among Cosmos assets because it has the highest market cap, most liquidity, largest brand, and many integrations with fiat onramps. ATOM is the gateway to Cosmos.\\n\\nIn the Cosmos Hub Port City vision, ATOMs are pitched as equity in the Cosmos Hub.  However, this alone is insufficient to establish ATOM as the base currency of the Cosmos ecosystem as a whole. Instead, the ATOM community must work to actively promote the use of ATOMs throughout the Cosmos ecosystem, rather than passively relying on the Hub's reputation to create ATOM's value.\\n\\nIn order to cement the role of ATOMs in Cosmos DeFi, the Cosmos Hub should leverage its community pool to help align incentives with other protocols within the Cosmos ecosystem. We propose beginning this initiative by using the community pool ATOMs to incentivize deep ATOM base pair liquidity pools on the Osmosis Network.\\n\\nOsmosis is the first IBC-enabled DeFi application. Within its 3 weeks of existence, it has already 100x\\u2019d the number of IBC transactions ever created, demonstrating the power of IBC and the ability of the Cosmos SDK to bootstrap DeFi protocols with $100M+ TVL in a short period of time. Since its announcement Osmosis has helped bring renewed attention and interest to Cosmos from the crypto community at large and kickstarted the era of Cosmos DeFi.\\n\\nOsmosis has already helped in establishing ATOM as the Schelling Point of the Cosmos ecosystem.  The genesis distribution of OSMO was primarily based on an airdrop to ATOM holders specifically, acknowledging the importance of ATOM to all future projects within the Cosmos. Furthermore, the Osmosis LP rewards currently incentivize ATOMs to be one of the main base pairs of the platform.\\n\\nOsmosis has the ability to incentivize AMM liquidity, a feature not available on any other IBC-enabled DEX. Osmosis already uses its own native OSMO liquidity rewards to incentivize ATOMs to be one of the main base pairs, leading to ~2.2 million ATOMs already providing liquidity on the platform.\\n\\nIn addition to these native OSMO LP Rewards, the platform also includes a feature called \\u201cexternal incentives\\u201d that allows anyone to permissionlessly add additional incentives in any token to the LPs of any AMM pools they wish. You can read more about this mechanism here: https:\\/\\/medium.com\\/osmosis\\/osmosis-liquidity-mining-101-2fa58d0e9d4d#f413 . Pools containing Cosmos assets such as AKT and XPRT are already planned to receive incentives from their respective community pools and\\/or foundations.\\n\\nWe propose the Cosmos Hub dedicate 100,000 ATOMs from its Community Pool to be allocated towards liquidity incentives on Osmosis over the next 3 months. This community fund proposal will transfer 100,000 ATOMs to a multisig group who will then allocate the ATOMs to bonded liquidity gauges on Osmosis on a biweekly basis, according to direction given by Cosmos Hub governance.  For simplicity, we propose setting the liquidity incentives to initially point to Osmosis Pool #1, the ATOM\\/OSMO pool, which is the pool with by far the highest TVL and Volume. Cosmos Hub governance can then use Text Proposals to further direct the multisig members to reallocate incentives to new pools.\\n\\nThe multisig will consist of a 2\\/3 key holder set consisting of the following individuals whom have all agreed to participate in this process shall this proposal pass:\\n\\n- Zaki Manian\\n- Federico Kunze\\n- Marko Baricevic\\n\\nThis is one small step for the Hub, but one giant leap for ATOM-aligned.\\n\",\"recipient\":\"cosmos157n0d38vwn5dvh64rc39q3lyqez0a689g45rkc\",\"amount\":[{\"denom\":\"uatom\",\"amount\":\"100000000000\"}]},\"initial_deposit\":[{\"denom\":\"uatom\",\"amount\":\"64000000\"}],\"proposer\":\"cosmos1ey69r37gfxvxg62sh4r0ktpuc46pzjrmz29g45\"}],\"memo\":\"\",\"timeout_height\":\"0\",\"extension_options\":[],\"non_critical_extension_options\":[]},\"auth_info\":{\"signer_infos\":[{\"public_key\":{\"@type\":\"\\/cosmos.crypto.multisig.LegacyAminoPubKey\",\"threshold\":2,\"public_keys\":[{\"@type\":\"\\/cosmos.crypto.secp256k1.PubKey\",\"key\":\"AldOvgv8dU9ZZzuhGydQD5FYreLhfhoBgrDKi8ZSTbCQ\"},{\"@type\":\"\\/cosmos.crypto.secp256k1.PubKey\",\"key\":\"AxUMR\\/GKoycWplR+2otzaQZ9zhHRQWJFt3h1bPg1ltha\"},{\"@type\":\"\\/cosmos.crypto.secp256k1.PubKey\",\"key\":\"AlI9yVj2Aejow6bYl2nTRylfU+9LjQLEl3keq0sERx9+\"},{\"@type\":\"\\/cosmos.crypto.secp256k1.PubKey\",\"key\":\"A0UvHPcvCCaIoFY9Ygh0Pxq9SZTAWtduOyinit\\/8uo+Q\"},{\"@type\":\"\\/cosmos.crypto.secp256k1.PubKey\",\"key\":\"As7R9fDUnwsUVLDr1cxspp+cY9UfXfUf7i9\\/w+N0EzKA\"}]},\"mode_info\":{\"multi\":{\"bitarray\":{\"extra_bits_stored\":5,\"elems\":\"SA==\"},\"mode_infos\":[{\"single\":{\"mode\":\"SIGN_MODE_LEGACY_AMINO_JSON\"}},{\"single\":{\"mode\":\"SIGN_MODE_LEGACY_AMINO_JSON\"}}]}},\"sequence\":\"102\"}],\"fee\":{\"amount\":[],\"gas_limit\":\"10000000\",\"payer\":\"\",\"granter\":\"\"}},\"signatures\":[\"CkB\\/KKWTFntEWbg1A0vu7DCHffJ4x4db\\/EI8dIVzRFFW7iuZBzvq+jYBtrcTlVpEVfmCY3ggIMnWfbMbb1egIlYbCkAmDf6Eaj1NbyXY8JZZtYAX3Qj81ZuKZUBeLW1ZvH1XqAg9sl\\/sqpLMnsJzKfmqEXvhoMwu1YxcSzrY6CJfuYL6\"]}")
)
