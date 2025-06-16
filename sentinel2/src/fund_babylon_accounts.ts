import type { SigningCosmWasmClientOptions } from "@cosmjs/cosmwasm-stargate"
import { DirectSecp256k1HdWallet } from "@cosmjs/proto-signing"
import { coins } from "@cosmjs/proto-signing"
import { GasPrice } from "@cosmjs/stargate"
import { Effect, Logger, Schedule } from "effect"
import {
  createSigningCosmWasmClient
} from "@unionlabs/sdk/cosmos"

import { gql, request } from "graphql-request"
import { isFunded, addFunded } from "./db_queries.js"
import { Config } from "./helpers.js"
import { db } from "./sentinel2.js"

process.on("uncaughtException", err => {
    console.error("❌ Uncaught Exception:", err.stack || err)
  })
  process.on("unhandledRejection", (reason, promise) => {
    console.error("❌ Unhandled Rejection at:", promise, "reason:", reason)
  })
  
interface FundableAccounts {
    receiver_display: string
    traces: Array<{
      type: string
      transaction_hash: string
    }>
  }
  
  
const fetchFundableAccounts = (hasuraEndpoint: string) =>
  Effect.gen(function*() {
    const query = gql`
      query {
        v2_transfers(args: { p_destination_universal_chain_id: "babylon.bbn-1" }) {
          receiver_display
          traces {
            type
            transaction_hash
          }
        }
      }
    `

    const response: any = yield* Effect.tryPromise({
      try: () => request(hasuraEndpoint, query),
      catch: error => {
        console.error("fetchFundableAccounts failed:", error)
        throw error
      },
    })

    const tokens: Array<FundableAccounts> = response?.v2_transfers || []
    const filtered: Array<FundableAccounts> = tokens
      .map(({ receiver_display, traces }) => ({
        receiver_display,
        traces: traces
          .filter(
            trace =>
              trace.type === "WRITE_ACK"
              && trace.transaction_hash != null
              && !isFunded(db, trace.transaction_hash),
          )
          // biome-ignore lint/style/noNonNullAssertion: <explanation>
          .map(trace => ({ type: trace.type, transaction_hash: trace.transaction_hash! })),
      }))
      .filter(acc => acc.traces.length > 0)

    return filtered
  })


export const fundBabylonAccounts = Effect.repeat(
  Effect.gen(function*(_) {
    yield* Effect.log("Funding babylon accounts loop started")
    let config = (yield* Config).config
    if (config.isLocal) {
      yield* Effect.log("Local mode: skipping funding babylon accounts")
      return
    }

    const wallet = yield* Effect.tryPromise(() =>
      DirectSecp256k1HdWallet.fromMnemonic(config.signer_account_mnemonic, { prefix: "bbn" })
    )
    const options: SigningCosmWasmClientOptions = {
      gasPrice: GasPrice.fromString("0.025bbn"),
    }
    const [senderAccount] = yield* Effect.tryPromise(() => wallet.getAccounts())

    const client = yield* createSigningCosmWasmClient(
      "https://rpc.bbn-1.babylon.chain.kitchen",
      wallet,
      options,
    )

    if (!senderAccount?.address) {
      yield* Effect.logError("Sender account couldnt found!")
      return
    }
    const balance = yield* Effect.tryPromise(() => client.getBalance(senderAccount.address, "ubbn"))

    if (Number.parseInt(balance.amount) < 1_000_000) {
      const errLog = Effect.annotateLogs({
        issueType: "SPENDER_BALANCE_LOW",
        balance: balance.amount,
        chainId: "babylon.bbn-1",
        tokenAddr: "ubbn",
        account: senderAccount.address,
      })(Effect.logError("SPENDER_BALANCE_LOW"))

      Effect.runFork(errLog.pipe(Effect.provide(Logger.json)))
      return
    }

    const fee = {
      amount: coins(500, "ubbn"),
      gas: "200000",
    }

    const accs = yield* fetchFundableAccounts(config.hasuraEndpoint)
    for (const acc of accs) {
      const receiver = acc.receiver_display
      const result = yield* Effect.tryPromise({
        try: () =>
          client.sendTokens(
            senderAccount.address,
            receiver,
            coins(10000, "ubbn"), // send 0.01 bbn
            fee,
          ),
        catch: err => {
          console.error("raw sendTokens error:", err)
          throw err
        },
      })

      addFunded(db, result.transactionHash)

      const okLog = Effect.annotateLogs({
        sentAmount: "0.01",
        chainId: "babylon.bbn-1",
        tokenAddr: "ubbn",
        account: senderAccount.address,
        receiver,
        transactionHash: result.transactionHash,
      })(Effect.logInfo("SENT_OK"))
      Effect.runFork(okLog.pipe(Effect.provide(Logger.json)))
    }
  }),
  Schedule.spaced("1 minutes"),
)
