import { Decimal } from "@cosmjs/math"
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import { Cosmos, Evm, FungibleAssetOrder, Ucs03, Ucs05 } from "@unionlabs/sdk"
import { UniversalChainId } from "@unionlabs/sdk/schema/chain"
import { ChannelId } from "@unionlabs/sdk/schema/channel"
import { Effect, Exit, pipe } from "effect"
import { http, toHex } from "viem"
import { bobSepolia } from "viem/chains"

const MNEMONIC = process.env.MNEMONIC || "memo memo memo"

const SENDER = Ucs05.AddressCosmosZkgm.make(toHex("bbn122ny3mep2l7nhtafpwav2y9e5jrslhekrn8frh"))
const RECEIVER = Ucs05.AddressEvmZkgm.make("0xfaebe5bf141cc04a3f0598062b98d2df01ab3c4d")
const SOURCE_UCS03_ADDRESS =
  "bbn15zcptld878lux44lvc0chzhz7dcdh62nh0xehwa8y7czuz3yljlspm2re6" as const

const main = Effect.gen(function*() {
  const sourceClient = Cosmos.ClientSource.Live(
    "https://rpc.bbn-1.babylon.chain.kitchen",
  )
  const destinationClient = Evm.PublicClientDestination.Live({
    chain: bobSepolia,
    transport: http(),
  })
  const wallet = yield* Effect.tryPromise(() =>
    DirectSecp256k1HdWallet.fromMnemonic(MNEMONIC, { prefix: "bbn" })
  )
  const signingClient = Cosmos.SigningClientContext.Live(
    "https://rest.bbn-1.babylon.chain.kitchen",
    wallet,
    { gasPrice: { amount: Decimal.fromUserInput("1", 6), denom: "ubbn" } },
  )

  return yield* pipe(
    FungibleAssetOrder.cosmosToEvm({
      sender: SENDER,
      receiver: RECEIVER,
      baseToken: "ubbn",
      baseAmount: 100n,
      quoteAmount: 100n,
      sourceChainId: UniversalChainId.make("babylon.bbn-1"),
      sourceChannelId: ChannelId.make(1),
    }),
    Effect.map((intent) => Ucs03.Batch.make({ operand: [intent] })),
    Effect.andThen((instruction) => Cosmos.sendInstruction(instruction, SENDER)),
    Effect.provide(signingClient),
    Effect.provide(sourceClient),
    Effect.provide(destinationClient),
    Effect.provideService(Evm.ChannelDestination, {
      ucs03address: "0xe33534b7f8D38C6935a2F6Ad35E09228dA239962",
      channelId: 1,
    }),
    Effect.provideService(Cosmos.ChannelSource, {
      ucs03address: SOURCE_UCS03_ADDRESS,
      channelId: 1,
    }),
  )
})

Effect.runPromiseExit(main).then(Exit.match({
  onFailure: (cause) => console.error(JSON.stringify(cause, null, 2)),
  onSuccess: (result) => console.log(result),
}))
