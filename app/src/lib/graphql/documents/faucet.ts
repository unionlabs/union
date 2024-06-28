export const faucetUnoMutation = /* GraphQL */ `
  mutation FaucetUnoMutation($address: Address!, $captchaToken: String!) {
    faucet {
      send(input: {toAddress: $address, captchaToken: $captchaToken})
    }
  }
`
export const faucetUnoMutation2 = /* GraphQL */ `
  mutation FaucetUnoMutation2($address: String!, $captchaToken: String!) {
    faucet2 {
      send(toAddress: $address, captchaToken: $captchaToken)
    }
  }
`
