import {Data, Effect, type Exit, Option} from "effect";
import type {Transfer} from "$lib/components/Transfer/transfer.svelte.ts";
import {checkBalance} from "$lib/components/Transfer/state/filling/check-balance.ts";
import {createOrdersBatch} from "$lib/components/Transfer/state/filling/create-orders.ts";
import {checkAllowances} from "$lib/components/Transfer/state/filling/check-allowance.ts";

export type StateResult = {
  nextState: CreateTransferState | null;
  message?: string;
};

export type EffectToExit<T> = T extends Effect.Effect<infer A, infer E, any>
  ? Exit.Exit<A, E>
  : never;

export type CreateTransferState = Data.TaggedEnum<{
  Filling: {};
  CreateIntents: {};
  CheckBalance: {};
  CheckAllowance: {};
  CreateOrders: {
    allowances: Array<{ token: string; allowance: bigint }>;
  };
  CreateSteps: {
    allowances: Array<{ token: string; allowance: bigint }>;
    orders: Array<unknown>;
  };
}>;

export const CreateTransferState = Data.taggedEnum<CreateTransferState>();

const {
  CreateIntents,
  CheckBalance,
  CheckAllowance,
  CreateOrders,
  CreateSteps
} = CreateTransferState;

export const createTransferState = (
  cts: CreateTransferState,
  transfer: Transfer
): Effect.Effect<StateResult, unknown, never> => {
  if (
    !Option.isSome(transfer.sourceChain) ||
    !Option.isSome(transfer.destinationChain) ||
    !Option.isSome(transfer.baseToken) ||
    !Option.isSome(transfer.derivedSender) ||
    !Option.isSome(transfer.parsedAmount) ||
    !Option.isSome(transfer.ucs03address) ||
    !Option.isSome(transfer.channel) ||
    !Option.isSome(transfer.intents)
  ) {
    console.log("[CTS] Missing arguments");
    return Effect.succeed({
      nextState: null,
      message: "Missing arguments"
    });
  }

  // Destructure the needed values
  const channel = transfer.channel.value;
  const ucs03address = transfer.ucs03address.value;
  const source = transfer.sourceChain.value;
  const destination = transfer.destinationChain.value;
  const token = transfer.baseToken.value;
  const sender = transfer.derivedSender.value;
  const amount = transfer.parsedAmount.value;
  const intents = transfer.intents.value;

  if (amount === "0" || amount === "" || BigInt(amount) === BigInt(0)) {
    return Effect.succeed({
      nextState: null,
      message: "Please enter a non-zero amount"
    });
  }

  return CreateTransferState.$match(cts, {
    Filling: () =>
      Effect.succeed({
        nextState: CreateIntents(),
        message: "Creating intents..."
      }),
    CreateIntents: () =>
      Effect.succeed({
        nextState: CheckBalance(),
        message: "Checking balance..."
      }),
    CheckBalance: () =>
      Effect.gen(function* () {
        const hasEnoughBalance = yield* checkBalance(source, sender, token, amount);
        return hasEnoughBalance
          ? { nextState: CheckAllowance(), message: "Checking allowance..." }
          : { nextState: null, message: "Insufficient funds" };
      }),
    CheckAllowance: () =>
      Effect.gen(function* ($) {
        // Attempt to retrieve allowances
        const allowancesOpt = yield* $(checkAllowances(source, intents, sender, ucs03address));

        if (Option.isNone(allowancesOpt)) {
          return {
            nextState: null,
            message: "CheckAllowance not supported or no tokens to approve"
          };
        }
        const allowances = allowancesOpt.value;
        return {
          nextState: CreateOrders({ allowances }),
          message: "Allowances found, creating orders next..."
        };
      }),
    CreateOrders: ({ allowances }) =>
      Effect.gen(function* () {
        // Now we can pass in the chain details & create the batch
        const batchOpt = yield* createOrdersBatch(
          source,
          destination,
          channel,
          ucs03address,
          intents
        );

        if (Option.isNone(batchOpt)) {
          return {
            nextState: null,
            message: "Could not create orders"
          };
        }

        const batch = batchOpt.value;
        console.log("Successfully created batch:", batch);
        return {
          nextState: CreateSteps({
            allowances,
            orders: batch
          }),
          message: "Orders created successfully"
        };
      }),
    CreateSteps: ({ allowances, orders }) =>
      Effect.succeed({
        nextState: null,
        message: `Transfer process complete (or ready) – allowances: ${JSON.stringify(
          allowances
        )}, orders: ${JSON.stringify(orders)}`
      })
  });
};
