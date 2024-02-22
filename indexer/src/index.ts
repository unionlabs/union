import { ponder } from "@/generated";
import { evmDecodeUnionAddress } from "./utilities/codec.ts";

ponder.on("UCS01_RELAY:Sent", async ({ event, context }) => {
  await context.db.SentEvent.create({
    id: event.transaction.hash,
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
    id: event.transaction.hash,
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

ponder.on("UCS01_RELAY:Refunded", async ({ event, context }) => {
  await context.db.RefundedEvent.create({
    id: event.transaction.hash,
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
    id: event.transaction.hash,
    data: {
      denom: event.args.denom,
      token: event.args.token,
      timestamp: event.block.timestamp,
    },
  });
});
