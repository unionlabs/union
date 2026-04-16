import type { Environment } from "$lib/constants"
import { chains } from "$lib/stores/chains.svelte"
import { createQueryGraphql } from "$lib/utils/queries"
import { Chains } from "@unionlabs/sdk/schema"
import { Chain, UniversalChainId } from "@unionlabs/sdk/schema/chain"
import * as Ucs05 from "@unionlabs/sdk/Ucs05"
import * as Array from "effect/Array"
import * as Config from "effect/Config"
import * as Effect from "effect/Effect"
import * as Either from "effect/Either"
import { pipe } from "effect/Function"
import * as Option from "effect/Option"
import * as Record from "effect/Record"
import * as Schema from "effect/Schema"
import * as String from "effect/String"
import * as Struct from "effect/Struct"
import * as Tuple from "effect/Tuple"
import { graphql } from "gql.tada"

const Contract = Schema.Struct({
  name: Schema.String,
  salt: Schema.optional(Schema.String),
  height: Schema.BigIntFromNumber,
  commit: Schema.optional(Schema.String),
  code_id: Schema.optional(Schema.BigIntFromNumber),
})
export type Contract = typeof Contract.Type

const IbcInterface = Schema.Literal("ibc-solidity", "ibc-cosmwasm", "ibc-move/sui")
export type IbcInterface = typeof IbcInterface.Type

const ChainDeployment = Schema.Struct({
  ibc_interface: IbcInterface,
  deployer: Schema.optional(Schema.String),
  sender: Schema.optional(Schema.String),
  contracts: Schema.Record({ key: Schema.String, value: Contract }),
})
export type ChainDeployment = typeof ChainDeployment.Type

const Deployments = Schema.Record({
  key: Schema.String,
  value: ChainDeployment,
})
export type Deployments = typeof Deployments.Type

export const chainsQuery = Effect.fn(function*(environment: Environment) {
  const deployments = yield* pipe(
    Config.string("DEPLOYMENTS_JSON"),
    Effect.tap((x) => Effect.log("CONFIG", x)),
    Effect.map(JSON.parse),
    Effect.tap((x) => Effect.log("PARSED", x)),
    // Schema.decodeUnknown(Schema.parseJson()),
    Effect.flatMap(Schema.decodeUnknown(Deployments)),
    Effect.tap((x) => Effect.log("DECODED", x)),
  )

  // NOTE: attempts to match contract name to supply minter address
  const minterDisplayAddressFromUniversalChainId = (id: UniversalChainId) =>
    pipe(
      deployments,
      Record.findFirst((_value, key) => key === id),
      Option.map(Tuple.getSecond),
      Option.map(Struct.get("contracts")),
      Option.flatMap(Record.findFirst((value, _key) => String.includes("minter")(value.name))),
      Option.map(Tuple.getFirst),
      Option.flatMap(Schema.decodeOption(Ucs05.AnyDisplayFromString)),
    )

  return yield* createQueryGraphql({
    schema: Schema.Struct({ v2_chains: Chains }),
    document: graphql(`
          query Chains($environment: String!) @cached(ttl: 60) {
              v2_chains {
                  chain_id
                  universal_chain_id
                  minter_address_display
                  display_name
                  addr_prefix
                  rpc_type
                  testnet
                  editions {
                      environment
                      name
                  }
                  features(where: { environment: { _eq: $environment } }) {
                      channel_list
                      connection_list
                      index_status
                      packet_list
                      transfer_submission
                      transfer_list
                  }
                  rpcs {
                      type
                      url
                  }
                  explorers {
                      address_url
                      block_url
                      description
                      display_name
                      home_url
                      name
                      tx_url
                  }
              }
          }
      `),
    variables: { environment },
    refetchInterval: "1 hour",
    writeData: data => {
      chains.data = pipe(
        data,
        Option.map((d) => d.v2_chains),
        Option.map(Array.map((x) =>
          new Chain({
            ...x,
            minter_address_display: pipe(
              minterDisplayAddressFromUniversalChainId(x.universal_chain_id),
              Option.map(Struct.get("address")),
              Option.getOrNull,
            ),
          })
        )),
      )
    },
    writeError: error => {
      chains.error = error
    },
  })
})
