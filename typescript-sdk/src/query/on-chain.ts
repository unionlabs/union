type rpcUrlArgument = { rpcUrl: string }
export type RpcQueryPath = "height" | "block" | "transaction" | "net_info" | "health"

const queryHeaders = new Headers({
  Accept: "application/json",
  "User-Agent": "typescript-sdk",
  "Content-Type": "application/json"
})

export async function getCosmosHeight({ rpcUrl }: { rpcUrl: string }) {
  const response = await fetch(`${rpcUrl}/header`)
  const json = (await response.json()) as { result: { header: { height: string } } }
  return Number.parseInt(json.result.header.height)
}

export async function getCosmosTransactionReceipt(params: {
  hash: string
  rpcUrl: string
}) {
  const url = `${params.rpcUrl}/tx_search?query="tx.hash='${params.hash}'"`
  const response = await fetch(url, { headers: queryHeaders })
  return await response.json()
}

export async function getCosmosAccountTransactions({
  address,
  rpcUrl
}: { address: string } & rpcUrlArgument) {
  const senderUrl = `${rpcUrl}/tx_search?query="transfer.sender='${address}'"`
  const recipientUrl = `${rpcUrl}/tx_search?query="transfer.recipient='${address}'"`
  const [sent, received] = await Promise.all([
    fetch(senderUrl, { headers: queryHeaders })
      .then(_ => _.json())
      .catch(),
    fetch(recipientUrl, { headers: queryHeaders })
      .then(_ => _.json())
      .catch()
  ])

  return {
    sent,
    received,
    // @ts-expect-error
    total: Number.parseInt(sent.result.total_count) + Number.parseInt(received.result.total_count)
  }
}
