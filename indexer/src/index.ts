import { ponder } from "@/generated";
import { evmDecodeUnionAddress } from "./utilities/codec.ts";

/**
 * UCS01_RELAY
 */
ponder.on("UCS01_RELAY:Sent", async ({ event, context }) => {
  await context.db.SentEvent.create({
    id: `${event.block.hash}-${event.transaction.hash}-${event.log.logIndex}`,
    data: {
      sender: event.args.sender,
      receiver: evmDecodeUnionAddress(event.args.receiver),
      denom: event.args.denom,
      token: event.args.token,
      amount: event.args.amount,
      timestamp: event.block.timestamp,
    },
  });
});

ponder.on("UCS01_RELAY:Received", async ({ event, context }) => {
  await context.db.ReceivedEvent.create({
    id: `${event.block.hash}-${event.transaction.hash}-${event.log.logIndex}`,
    data: {
      sender: evmDecodeUnionAddress(event.args.sender),
      receiver: event.args.receiver,
      denom: event.args.denom,
      token: event.args.token,
      amount: event.args.amount,
      timestamp: event.block.timestamp,
    },
  });
});

ponder.on("UCS01_RELAY:Refunded", async ({ event, context }) => {
  await context.db.RefundedEvent.create({
    id: `${event.block.hash}-${event.transaction.hash}-${event.log.logIndex}`,
    data: {
      sender: event.args.sender,
      receiver: event.args.receiver,
      denom: event.args.denom,
      token: event.args.token,
      amount: event.args.amount,
      timestamp: event.block.timestamp,
    },
  });
});

ponder.on("UCS01_RELAY:DenomCreated", async ({ event, context }) => {
  await context.db.DenomCreatedEvent.create({
    id: `${event.block.hash}-${event.transaction.hash}-${event.log.logIndex}`,
    data: {
      denom: event.args.denom,
      token: event.args.token,
      timestamp: event.block.timestamp,
    },
  });
});

/**
 * UNO_ERC20
 */
ponder.on("UNO_ERC20:Approval", async ({ event, context }) => {
  await context.db.ApprovalEvent.create({
    id: `${event.block.hash}-${event.transaction.hash}-${event.log.logIndex}`,
    data: {
      owner: event.args.owner,
      spender: event.args.spender,
      amount: event.args.value,
      timestamp: event.block.timestamp,
    },
  });
});

ponder.on("UNO_ERC20:Transfer", async ({ event, context }) => {
  await context.db.TransferEvent.create({
    id: `${event.block.hash}-${event.transaction.hash}-${event.log.logIndex}`,
    data: {
      sender: event.args.from,
      receiver: event.args.to,
      amount: event.args.value,
      timestamp: event.block.timestamp,
    },
  });
});
