import { createQueryGraphql } from "$lib/utils/queries.ts"
import { Option, Schema } from "effect"
import { graphql } from "gql.tada"
import { transferHashStore } from "$lib/stores/transfer-hash.svelte.ts"
import { PacketHash } from "@unionlabs/sdk/schema"

export const transferPacketHashQuery = ({ submission_tx_hash }: { submission_tx_hash: string }) =>
  createQueryGraphql({
    schema: Schema.Struct({
      v2_transfers: Schema.Array(
        Schema.Struct({
          packet_hash: PacketHash
        })
      )
    }),
    document: graphql(/* GraphQL */ `
      query($submission_tx_hash: String!) {
          v2_transfers(args: {
              p_transaction_hash: $submission_tx_hash
          }) {
              packet_hash
          }
      }
  `),
    variables: {
      submission_tx_hash
    },
    refetchInterval: "1 seconds",
    writeData: data => {
      transferHashStore.data = data.pipe(
        Option.flatMap(result =>
          result.v2_transfers.length > 0
            ? Option.some(result.v2_transfers[0].packet_hash)
            : Option.none()
        )
      )
    },

    writeError: error => {
      transferHashStore.error = error
    }
  })
