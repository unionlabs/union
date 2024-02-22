import { ponder } from "@/generated";

ponder.on("UCS01_RELAY:Sent", async ({ event, context }) => {
  await context.db.Sent.create({
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
