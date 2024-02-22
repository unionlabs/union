import { createSchema } from "@ponder/core";

export default createSchema((p) => ({
  SentEvent: p.createTable({
    id: p.hex(),
    sender: p.hex(),
    receiver: p.string(),
    denom: p.string(),
    token: p.hex(),
    amount: p.bigint(),
    timestamp: p.bigint(),
  }),

  ReceivedEvent: p.createTable({
    id: p.hex(),
    sender: p.hex(),
    receiver: p.string(),
    denom: p.string(),
    token: p.hex(),
    amount: p.bigint(),
    timestamp: p.bigint(),
  }),

  RefundedEvent: p.createTable({
    id: p.hex(),
    sender: p.hex(),
    receiver: p.string(),
    denom: p.string(),
    token: p.hex(),
    amount: p.bigint(),
    timestamp: p.bigint(),
  }),

  DenomCreatedEvent: p.createTable({
    id: p.hex(),
    denom: p.string(),
    token: p.hex(),
    timestamp: p.bigint(),
  }),
}));
