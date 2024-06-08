import { createMutation, useQueryClient } from "@tanstack/svelte-query"
import { faucetUnoMutation } from "$lib/graphql/documents/faucet.ts"
import { URLS } from "$lib/constants"

export async function getUnoFromFaucet(address: string) {
  const response = await fetch("https://graphql.union.build/v1/graphql", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({
      query: faucetUnoMutation
      variables: { address },
      operationName: "FaucetUnoMutation"
    })
  })

  if (!response.ok) {
    const errorText = await response.text()
    console.error("Fetch error:", errorText)
    return { error: errorText, status: response.status }
  }

  const responseJson = (await response.json()) as {
    data?: { faucet: { send: string } }
    errors?: Array<{ message: string }>
  }

  if (responseJson.errors && responseJson.errors.length > 0) {
    const errorMessage = responseJson.errors.map(e => e.message).join("; ")
    console.error("GraphQL error:", errorMessage)
    return { error: errorMessage, status: response.status }
  }

  return { data: responseJson.data, status: response.status }
}
