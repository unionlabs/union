import { fetcher } from '$/lib/utilities'

const transfersQuery = /* GraphQL */ `
  query TransfersQuery($address: String!) {
    sentEvents(
      orderBy: "timestamp"
      orderDirection: "desc"
      where: { OR: [{ sender: $address }, { receiver: $address }] }
    ) {
      items {
        id
        sender
        receiver
        token
        timestamp
        amount
      }
    }
    transferEvents(
      orderBy: "timestamp"
      orderDirection: "desc"
      where: { OR: [{ sender: $address }, { receiver: $address }] }
    ) {
      items {
        id
        sender
        receiver
        timestamp
        amount
      }
    }
  }
`

export interface TransferEvent {
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
      sentEvents: { items: Array<{ token: string } & TransferEvent> }
      transferEvents: { items: Array<TransferEvent> }
    }
  }>('https://union.up.railway.app', {
    method: 'POST',
    body: JSON.stringify({
      query: transfersQuery,
      variables: { address: address.toLowerCase() },
      operationName: 'TransfersQuery'
    })
  })

  const sepoliaUnionTransfers = response.data.sentEvents.items
  const sepoliaTransfers = response.data.transferEvents.items.map(transfer => ({
    ...transfer,
    token: '0x465F49DE53eDDAc9635C4Cc941F442C12bE351B0'
  }))

  return [...sepoliaUnionTransfers, ...sepoliaTransfers].sort((a, b) => +b.timestamp - +a.timestamp)
}
