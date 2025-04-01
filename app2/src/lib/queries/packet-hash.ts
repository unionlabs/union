import {fetchDecodeGraphql} from "$lib/utils/queries.ts";
import {Schema} from "effect";
import {graphql} from "gql.tada";

export const transferPacketHashQuery = (
  {submission_tx_hash}: { submission_tx_hash: string }
) =>
fetchDecodeGraphql(
  Schema.Struct({
    v2_transfers: Schema.Array(
      Schema.Struct({
        packet_hash: Schema.String
      })
    )
  }),
  graphql(/* GraphQL */ `
      query($submission_tx_hash: String!) {
          v2_transfers(where: {
              transfer_send_transaction_hash: $submission_tx_hash
          }) {
              packet_hash
          }
      }
  `),
  {
    submission_tx_hash
  }
)