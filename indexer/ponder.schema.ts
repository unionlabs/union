import { createSchema } from "@ponder/core";

export default createSchema((p) => ({
  /**
   * UCS01_RELAY
   */
  SentEvent: p.createTable({
    id: p.string(),
    transactionHash: p.hex(),
    sender: p.hex(),
    receiver: p.string(),
    denom: p.string(),
    token: p.hex(),
    amount: p.bigint(),
    timestamp: p.bigint(),
  }),

  ReceivedEvent: p.createTable({
    id: p.string(),
    transactionHash: p.hex(),
    sender: p.string(),
    receiver: p.hex(),
    denom: p.string(),
    token: p.hex(),
    amount: p.bigint(),
    timestamp: p.bigint(),
  }),

  RefundedEvent: p.createTable({
    id: p.string(),
    transactionHash: p.hex(),
    sender: p.hex(),
    receiver: p.string(),
    denom: p.string(),
    token: p.hex(),
    amount: p.bigint(),
    timestamp: p.bigint(),
  }),

  DenomCreatedEvent: p.createTable({
    id: p.string(),
    transactionHash: p.hex(),
    denom: p.string(),
    token: p.hex(),
    timestamp: p.bigint(),
  }),

  /**
   * UNO_ERC20
   */
  TransferEvent: p.createTable({
    id: p.string(),
    transactionHash: p.hex(),
    sender: p.hex(),
    receiver: p.hex(),
    amount: p.bigint(),
    timestamp: p.bigint(),
  }),

  ApprovalEvent: p.createTable({
    id: p.string(),
    transactionHash: p.hex(),
    owner: p.hex(),
    spender: p.hex(),
    amount: p.bigint(),
    timestamp: p.bigint(),
  }),
}));
