type rpcUrlArgument = { rpcUrl: string }
export type RpcQueryPath = "height" | "block" | "transaction" | "net_info" | "health"

const queryHeaders = new Headers({
  Accept: "application/json",
  "User-Agent": "typescript-sdk",
  "Content-Type": "application/json"
})

/**
 * get the current block height
 * @example
 * ```ts
 * const height = await getCosmosHeight({
 *   rpcUrl: "https://rpc.testnet-8.union.build",
 * })
 * ```
 */
export async function getCosmosHeight({ rpcUrl }: { rpcUrl: string }) {
  const response = await fetch(`${rpcUrl}/header`)
  const json = (await response.json()) as { result: { header: { height: string } } }
  return Number.parseInt(json.result.header.height)
}

/**
 * get the transaction receipt for a given transaction hash
 * @example
 * ```ts
 * const receipt = await getCosmosTransactionReceipt({
 *   rpcUrl: "https://rpc.testnet-8.union.build",
 *   hash: "A6E276CE66CDB35C0CAAC49EC9AAB3CB2CF8A34C807A4C729EA385E64C88D69B",
 * })
 * ```
 */
export async function getCosmosTransactionReceipt(params: {
  hash: string
  rpcUrl: string
}) {
  const url = `${params.rpcUrl}/tx_search?query="tx.hash='${params.hash}'"`
  const response = await fetch(url, { headers: queryHeaders })
  return await response.json()
}

/**
 * get the transactions sent and received by an address
 * @example
 * ```ts
 * const transactions = await getCosmosAccountTransactions({
 *   address: "union1qp0wtsfltjk9rnvyu3fkdv0s0skp4y5y3py96f",
 *   rpcUrl: "https://rpc.testnet-8.union.build",
 * })
 * ```
 */
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
