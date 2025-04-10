import { graphql } from "gql.tada"
import { createQueryGraphql } from "$lib/utils/queries"
import { Schema } from "effect"
import { URLS } from "$lib/constants"

export const faucetUnoMutationDocument = graphql(`
  mutation UnoFaucetMutation(
    $chainId: String!,
    $denom: String!,
    $address: String!,
    $captchaToken: String!
  ) {
    drip_drop {
      send(
        chainId: $chainId,
        denom: $denom,
        address: $address,
        captchaToken: $captchaToken
      )
    }
  }
`)

export const faucetUnoMutation = ({
  chainId,
  denom,
  address,
  captchaToken
}: {
  chainId: string
  denom: string
  address: string
  captchaToken: string
}) =>
  createQueryGraphql({
    schema: Schema.Struct({ send: Schema.String }),
    document: faucetUnoMutationDocument,
    variables: { chainId, denom, address, captchaToken },
    url: URLS().GRAPHQL
  })
