export async function getUnoFromFaucet(address: string) {
  const response = await fetch("https://graphql.union.build/v1/graphql", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      query: /* GraphQL */ `
        mutation GetUno($address: Address!) {
          union { send(input: { toAddress: $address }) }
        }
      `,
      variables: { address },
      operationName: "GetUno"
    })
  })

  if (!response.ok) return { error: await response.text(), status: response.status }

  const responseJson = (await response.json()) as {
    data?: { union: { send: string } }
    errors?: Array<{ message: string }>
  }
  if ("errors" in responseJson) {
    const [error] = responseJson.errors || []
    console.error(error)
    return { error: error.message, status: response.status }
  }

  return { data: responseJson.data, status: response.status }
}
