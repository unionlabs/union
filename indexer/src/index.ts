import { ponder } from "@/generated"
import { evmDecodeUnionAddress } from "./utilities/codec.ts"

/**
 * UCS01_RELAY
 */
ponder.on("UCS01_RELAY:Sent", async ({ event, context }) => {
  await context.db.TransferEvent.create({
    id: `${event.block.hash}-${event.transaction.hash}-${event.log.logIndex}`,
    data: {
      sourceChainId: "11155111",
      targetChainId: "union-testnet-6",
      event: "UCS01_RELAY:Sent",
      sender: event.args.sender,
      receiver: evmDecodeUnionAddress(event.args.receiver),
      denom: event.args.denom,
      token: event.args.token,
      amount: event.args.amount,
      timestamp: event.block.timestamp
    }
  })
})

ponder.on("UCS01_RELAY:Received", async ({ event, context }) => {
  await context.db.TransferEvent.create({
    id: `${event.block.hash}-${event.transaction.hash}-${event.log.logIndex}`,
    data: {
      sourceChainId: "union-testnet-6",
      targetChainId: "11155111",
      event: "UCS01_RELAY:Received",
      sender: evmDecodeUnionAddress(event.args.sender),
      receiver: event.args.receiver,
      denom: event.args.denom,
      token: event.args.token,
      amount: event.args.amount,
      timestamp: event.block.timestamp
    }
  })
})

ponder.on("UCS01_RELAY:Refunded", async ({ event, context }) => {
  await context.db.RefundedEvent.create({
    id: `${event.block.hash}-${event.transaction.hash}-${event.log.logIndex}`,
    data: {
      sender: event.args.sender,
      receiver: event.args.receiver,
      denom: event.args.denom,
      token: event.args.token,
      amount: event.args.amount,
      timestamp: event.block.timestamp
    }
  })
})

ponder.on("UCS01_RELAY:DenomCreated", async ({ event, context }) => {
  await context.db.DenomCreatedEvent.create({
    id: `${event.block.hash}-${event.transaction.hash}-${event.log.logIndex}`,
    data: {
      denom: event.args.denom,
      token: event.args.token,
      timestamp: event.block.timestamp
    }
  })
})

/**
 * UNO_ERC20
 */
ponder.on("UNO_ERC20:Approval", async ({ event, context }) => {
  await context.db.ApprovalEvent.create({
    id: `${event.block.hash}-${event.transaction.hash}-${event.log.logIndex}`,
    data: {
      sourceChainId: "11155111",
      owner: event.args.owner,
      spender: event.args.spender,
      amount: event.args.value,
      timestamp: event.block.timestamp
    }
  })
})

ponder.on("UNO_ERC20:Transfer", async ({ event, context }) => {
  await context.db.TransferEvent.create({
    id: `${event.block.hash}-${event.transaction.hash}-${event.log.logIndex}`,
    data: {
      sourceChainId: "11155111",
      targetChainId: "11155111",
      event: "UNO_ERC20:Transfer",
      sender: event.args.from,
      token: context.contracts.UNO_ERC20.address,
      receiver: event.args.to,
      amount: event.args.value,
      timestamp: event.block.timestamp
    }
  })
})
