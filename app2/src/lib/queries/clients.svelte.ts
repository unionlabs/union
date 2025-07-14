import { clientsStore } from "$lib/stores/clients.svelte"
import { createQueryGraphql } from "$lib/utils/queries"
import { ClientId, Height, TransactionHash, UniversalChainId } from "@unionlabs/sdk/schema"
import { Option, Schema as S } from "effect"
import { graphql } from "gql.tada"

// Define the schema for client status
const ClientStatus = S.Struct({
  height: S.OptionFromNullOr(Height),
  timestamp: S.OptionFromNullOr(S.String),
  transaction_hash: S.OptionFromNullOr(TransactionHash),
  counterparty_height: S.OptionFromNullOr(Height),
})

const ChainStatus = S.Struct({
  height: S.OptionFromNullOr(Height),
  timestamp: S.OptionFromNullOr(S.String),
  tip_age_seconds: S.OptionFromNullOr(S.Number),
  status: S.OptionFromNullOr(S.String),
})

const Chain = S.Struct({
  status: S.OptionFromNullOr(ChainStatus),
})

const Client = S.Struct({
  client_id: ClientId,
  counterparty_universal_chain_id: UniversalChainId,
  universal_chain_id: UniversalChainId,
  chain: S.OptionFromNullOr(Chain),
  counterparty_chain: S.OptionFromNullOr(Chain),
  status: S.OptionFromNullOr(ClientStatus),
})

const Clients = S.Array(Client)

export const clientsQuery = () =>
  createQueryGraphql({
    schema: S.Struct({ v2_clients: Clients }),
    document: graphql(`
      query ClientStatus @cached(ttl: 60) {
        v2_clients {
          client_id
          counterparty_universal_chain_id
          universal_chain_id
          chain {
            status {
              height
              timestamp
              tip_age_seconds
              status
            }
          }
          counterparty_chain {
            status {
              height
              timestamp
              tip_age_seconds
              status
            }
          }
          status {
            height
            timestamp
            transaction_hash
            counterparty_height
          }
        }
      }
    `),
    variables: {},
    refetchInterval: "1 minute",
    writeData: data => {
      clientsStore.data = data.pipe(Option.map(d => d.v2_clients))
    },
    writeError: error => {
      clientsStore.error = error
    },
  })

export type Client = typeof Client.Type
export type Clients = typeof Clients.Type
