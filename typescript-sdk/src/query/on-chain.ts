
import { Comet38Client } from "@cosmjs/tendermint-rpc"

type rpcUrlArgument = { rpcUrl: string }
export type RpcQueryPath = "height" | "block" | "transaction" | "net_info" | "health"

const queryHeaders = new Headers({
  Accept: "application/json",
  "Content-Type": "application/json",
  "User-Agent": "typescript-sdk"
})

export async function getCosmosSdkChainHeight({ rpcUrl }: { rpcUrl: string }) {
  const response = await fetch(`${rpcUrl}/header`)
  const json = (await response.json()) as { result: { header: { height: string } } }
  return Number.parseInt(json.result.header.height)
}

export async function getCosmosSdkChainTransactionReceipt(params: {
  hash: string
  rpcUrl: string
}) {
  const client = await Comet38Client.connect(params.rpcUrl)
  return await client.txSearch({
    query: `tx.hash='${params.hash}'`
  })
}

export async function getCosmosSdkAccountTransactions({
  address,
  rpcUrl
}: { address: string } & rpcUrlArgument) {
  const client = await Comet38Client.connect(rpcUrl)
  const [sent, received] = await Promise.all([
    client.txSearch({
      query: `transfer.sender='${address}'`
    }),
    client.txSearch({
      query: `transfer.recipient='${address}'`
    })
  ])
  return { sent, received }
}
