import { fetcher } from './utilities'

export async function getUnoFromFaucet({ address }: { address: string }) {
  console.log('getUnoFromFaucet', { address }, address.length)
  const response = await fetcher<
    | { data: { union: { send: undefined } } }
    | {
        errors: Array<{
          message: string
          extensions: { path: string; code: string }
        }>
      }
  >('https://noble-pika-27.hasura.app/v1/graphql', {
    method: 'POST',
    headers: { 'Content-Type': 'application/json' },
    body: JSON.stringify({
      query: /* GraphQL */ `
        mutation GetUno($address: Address!) {
          union {
            send(input: { toAddress: $address })
          }
        }
      `,
      variables: { address },
      operationName: 'GetUno'
    })
  })

  if ('errors' in response) {
    const [error] = response.errors
    console.error(error)
    throw new Error(error?.message)
  }

  console.log(JSON.stringify(response.data, undefined, 2))
  return response.data
}
