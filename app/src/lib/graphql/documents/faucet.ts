export const faucetUnoMutation = `
mutation FaucetUnoMutation($address: Address!) {
  faucet {
    send(input: {toAddress: $address})
  }
}`
