import { createSchema } from "@ponder/core";

export default createSchema((p) => ({
  Sent: p.createTable({
    id: p.hex(),
    sender: p.hex(),
    receiver: p.string(),
    denom: p.string(),
    token: p.hex(),
    amount: p.bigint(),
    timestamp: p.bigint(),
  }),

  Received: p.createTable({
    id: p.hex(),
    sender: p.hex(),
    receiver: p.string(),
    denom: p.string(),
    token: p.hex(),
    amount: p.bigint(),
    timestamp: p.bigint(),
  }),

  Refunded: p.createTable({
    id: p.hex(),
    sender: p.hex(),
    receiver: p.string(),
    denom: p.string(),
    token: p.hex(),
    amount: p.bigint(),
    timestamp: p.bigint(),
  }),

  DenomCreated: p.createTable({
    id: p.hex(),
    denom: p.string(),
    token: p.hex(),
    timestamp: p.bigint(),
  }),
}));
