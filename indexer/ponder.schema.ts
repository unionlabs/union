import { createSchema } from "@ponder/core"

export default createSchema(p => ({
  TransferEvent: p.createTable({
    sourceChainId: p.string(), // '11155111' or 'union-testnet-6'
    targetChainId: p.string(), // '11155111' or 'union-testnet-6'
    event: p.string(), // 'UCS01_RELAY:Sent', 'UCS01_RELAY:Received', or 'UNO_ERC20:Transfer'
    id: p.string(),
    sender: p.string(),
    receiver: p.string(),
    denom: p.string().optional(),
    token: p.hex(),
    amount: p.bigint(),
    timestamp: p.bigint()
  }),

  RefundedEvent: p.createTable({
    id: p.string(),
    sender: p.hex(),
    receiver: p.string(),
    denom: p.string(),
    token: p.hex(),
    amount: p.bigint(),
    timestamp: p.bigint()
  }),

  DenomCreatedEvent: p.createTable({
    id: p.string(),
    denom: p.string(),
    token: p.hex(),
    timestamp: p.bigint()
  }),

  ApprovalEvent: p.createTable({
    sourceChainId: p.string(), // '11155111'
    id: p.string(),
    owner: p.hex(),
    spender: p.hex(),
    amount: p.bigint(),
    timestamp: p.bigint()
  })
}))
