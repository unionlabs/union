import { fetcher } from "./utilities/index.ts"

export async function getUnoFromFaucet({ address }: { address: string }) {
  const response = await fetcher<
    | { data: { union: { send: undefined } } }
    | {
        errors: Array<{
          message: string
          extensions: { path: string; code: string }
        }>
      }
  >("https://graphql.union.build", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      query: /* GraphQL */ `
        mutation GetUno($address: Address!) {
          union {
            send(input: { toAddress: $address })
          }
        }
      `,
      variables: { address },
      operationName: "GetUno"
    })
  })

  if ("errors" in response) {
    const [error] = response.errors
    throw new Error(error?.message)
  }

  return response.data
}
