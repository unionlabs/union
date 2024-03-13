import { fetcher } from "$/lib/utilities"

const transfersQuery = /* GraphQL */ `
  query TransfersQuery($address: String!) {
    transferEvents(
      orderBy: "timestamp"
      orderDirection: "desc"
      where: { OR: [{ sender: $address }, { receiver: $address }] }
    ) {
      items {
        timestamp
        id
        event
        sourceChainId
        targetChainId
        sender
        receiver
        token
        amount
      }
    }
  }
`

type ChainId = "11155111" | "union-testnet-6"

export interface TransferEvent {
  sourceChainId: ChainId
  targetChainId: ChainId
  event:
    | "UCS01_RELAY:Sent" // Union to Sepolia
    | "UCS01_RELAY:Received" // Sepolia to Union
    | "UNO_ERC20:Transfer" // Sepolia to Sepolia
  id: string
  sender: string
  receiver: string
  timestamp: string
  amount: string
  token: string
}

export async function fetchUserTransfers({
  address
}: {
  address: string
}): Promise<TransferEvent[]> {
  const response = await fetcher<{
    data: {
      transferEvents: { items: Array<TransferEvent> }
    }
  }>("https://union.up.railway.app", {
    method: "POST",
    body: JSON.stringify({
      query: transfersQuery,
      variables: { address: address },
      operationName: "TransfersQuery"
    })
  })
  return response.data.transferEvents.items
}
