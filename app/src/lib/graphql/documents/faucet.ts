export const faucetUnoMutation = /* GraphQL */ `
  mutation FaucetUnoMutation($address: Address!, $captchaToken: String!) {
    faucet {
      send(input: {toAddress: $address, captchaToken: $captchaToken})
    }
  }
`
