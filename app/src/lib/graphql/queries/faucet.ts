import { graphql } from "gql.tada"

export const faucetUnoMutation2 = graphql(/* GraphQL */ `
  mutation UnoFaucetMutation(
    $chainId: String!,
    $denom: String!,
    $address: String!,
    $captchaToken: String!
  ) {
    send(
      chainId: $chainId,
      denom: $denom,
      address: $address,
      captchaToken: $captchaToken
    )
  }
`)

export const dydxFaucetMutation = graphql(/* GraphQL */ `
  mutation DyDxFaucetMutation($address: String!, $captchaToken: String!) {
    dydx_faucet {
      send(toAddress: $address, captchaToken: $captchaToken)
    }
  }  
`)

export const strideFaucetMutation = graphql(/* GraphQL */ `
  mutation StrideFaucetMutation($address: String!, $captchaToken: String!) {
    stride_faucet {
      send(toAddress: $address, captchaToken: $captchaToken)
    }
  }  
`)
