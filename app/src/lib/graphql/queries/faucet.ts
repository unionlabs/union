import { graphql } from "gql.tada"

export const faucetUnoMutation2 = graphql(/* GraphQL */ `
  mutation FaucetUnoMutation2($address: String!, $captchaToken: String!) {
    faucet2 {
      send(toAddress: $address, captchaToken: $captchaToken)
    }
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
