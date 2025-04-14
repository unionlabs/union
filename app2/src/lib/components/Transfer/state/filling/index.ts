import {Data, Effect, Option} from "effect"
import {type Transfer, type TransferIntents} from "$lib/components/Transfer/transfer.svelte.ts"
import {checkBalanceForIntents} from "$lib/components/Transfer/state/filling/check-balance.ts"
import {createOrdersBatch} from "$lib/components/Transfer/state/filling/create-orders.ts"
import {
  checkAllowances,
  type ApprovalStep
} from "$lib/components/Transfer/state/filling/check-allowance.ts"
import {
  FillingError,
  InsufficientFundsError,
  MissingTransferFieldsError,
  OrderCreationError,
  type TransferFlowError
} from "$lib/components/Transfer/state/errors.ts"
import type {Instruction} from "@unionlabs/sdk/ucs03/instruction"
import {wallets} from "$lib/stores/wallets.svelte.ts"

export type StateResult = {
  nextState: Option.Option<CreateTransferState>
  message: string
  orders: Option.Option<Array<Instruction>>
  allowances: Option.Option<Array<ApprovalStep>>
  error: Option.Option<TransferFlowError>
}

export type CreateTransferState = Data.TaggedEnum<{
  Filling: {}
  Validation: {}
  CreateIntents: {}
  CheckBalance: { intents: TransferIntents }
  CheckAllowance: {}
  CreateOrders: { allowances: Array<ApprovalStep> }
  CreateSteps: { allowances: Array<ApprovalStep>; orders: Array<Instruction> }
}>

export const CreateTransferState = Data.taggedEnum<CreateTransferState>()
const {Validation, CreateIntents, CheckBalance, CheckAllowance, CreateOrders, CreateSteps} =
  CreateTransferState

export type FillingState = Data.TaggedEnum<{
  WalletMissing: {}
  SourceChainMissing: {}
  ChainWalletMissing: {}
  BaseTokenMissing: {}
  DestinationMissing: {}
  InvalidAmount: {}
  ReceiverMissing: {}
  Ready: {}
}>

export const FillingState = Data.taggedEnum<FillingState>()

export const getFillingState = (transfer: Transfer): FillingState => {
  if (!wallets.hasAnyWallet()) {
    return FillingState.WalletMissing()
  }

  return Option.match(transfer.sourceChain, {
    onNone: () => {
      return FillingState.SourceChainMissing()
    },
    onSome: sourceChain => {
      const sourceWallet = wallets.getAddressForChain(sourceChain)
      if (Option.isNone(sourceWallet)) {
        return FillingState.ChainWalletMissing()
      }

      if (Option.isNone(transfer.baseToken)) {
        return FillingState.BaseTokenMissing()
      }

      if (Option.isNone(transfer.destinationChain)) {
        return FillingState.DestinationMissing()
      }

      if (!transfer.raw.amount) {
        return FillingState.InvalidAmount()
      }

      const parsedAmount = Number.parseFloat(transfer.raw.amount)
      if (Number.isNaN(parsedAmount) || parsedAmount <= 0) {
        return FillingState.InvalidAmount()
      }

      if (Option.isSome(transfer.destinationChain) && Option.isNone(transfer.derivedReceiver)) {
        return FillingState.ReceiverMissing()
      }

      return FillingState.Ready()
    }
  })
}


const fail = (msg: string, err?: TransferFlowError): StateResult => ({
  nextState: Option.none<CreateTransferState>(),
  message: msg,
  orders: Option.none<Array<Instruction>>(),
  allowances: Option.none<Array<ApprovalStep>>(),
  error: err ? Option.some<TransferFlowError>(err) : Option.none<TransferFlowError>()
})

const ok = (state: CreateTransferState, msg: string): StateResult => ({
  nextState: Option.some<CreateTransferState>(state),
  message: msg,
  orders: Option.none<Array<Instruction>>(),
  allowances: Option.none<Array<ApprovalStep>>(),
  error: Option.none<TransferFlowError>()
})

const complete = (
  msg: string,
  orders: Array<Instruction>,
  allowances: Array<ApprovalStep>
): StateResult => ({
  nextState: Option.none<CreateTransferState>(),
  message: msg,
  orders: Option.some<Array<Instruction>>(orders),
  allowances: Option.some<Array<ApprovalStep>>(allowances),
  error: Option.none<TransferFlowError>()
})

export const createTransferState = (cts: CreateTransferState, transfer: Transfer) => {
  if (
    Option.isNone(transfer.sourceChain) ||
    Option.isNone(transfer.destinationChain) ||
    Option.isNone(transfer.baseToken) ||
    Option.isNone(transfer.derivedSender) ||
    Option.isNone(transfer.parsedAmount) ||
    Option.isNone(transfer.ucs03address) ||
    Option.isNone(transfer.channel) ||
    Option.isNone(transfer.intents)
  ) {
    const missingFields: Array<string> = []

    if (Option.isNone(transfer.sourceChain)) missingFields.push("sourceChain")
    if (Option.isNone(transfer.destinationChain)) missingFields.push("destinationChain")
    if (Option.isNone(transfer.baseToken)) missingFields.push("baseToken")
    if (Option.isNone(transfer.derivedSender)) missingFields.push("derivedSender")
    if (Option.isNone(transfer.parsedAmount)) missingFields.push("parsedAmount")
    if (Option.isNone(transfer.ucs03address)) missingFields.push("ucs03address")
    if (Option.isNone(transfer.channel)) missingFields.push("channel")
    if (Option.isNone(transfer.intents)) missingFields.push("intents")

    console.log("filling: Missing required fields", missingFields)

    return Effect.succeed(
      fail(
        "Missing arguments",
        new MissingTransferFieldsError({ fields: missingFields })
      )
    )
  }


  const source = transfer.sourceChain.value
  const destination = transfer.destinationChain.value
  const channel = transfer.channel.value
  const ucs03address = transfer.ucs03address.value
  const sender = transfer.derivedSender.value
  const intents = transfer.intents.value

  return CreateTransferState.$match(cts, {
    Filling: () => {
      const state = getFillingState(transfer)

      return FillingState.$match(state, {
        WalletMissing: () =>
          Effect.succeed(
            fail(
              "Connect wallet",
              new FillingError({ cause: "WalletMissing" })
            )
          ),

        SourceChainMissing: () =>
          Effect.succeed(
            fail(
              "Select from chain",
              new FillingError({ cause: "SourceChainMissing" })
            )
          ),

        ChainWalletMissing: ({ chain }) =>
          Effect.succeed(
            fail(
              `Connect wallet for ${chain.name}`,
              new FillingError({ cause: "ChainWalletMissing", details: { chain } })
            )
          ),

        BaseTokenMissing: () =>
          Effect.succeed(
            fail(
              "Select asset",
              new FillingError({ cause: "BaseTokenMissing" })
            )
          ),

        DestinationMissing: () =>
          Effect.succeed(
            fail(
              "Select destination chain",
              new FillingError({ cause: "DestinationMissing" })
            )
          ),

        InvalidAmount: () =>
          Effect.succeed(
            fail(
              "Invalid amount",
              new FillingError({ cause: "InvalidAmount", details: { amount: transfer.raw.amount } })
            )
          ),

        ReceiverMissing: () =>
          Effect.succeed(
            fail(
              "Enter receiver address",
              new FillingError({ cause: "ReceiverMissing" })
            )
          ),

        ValidationFailed: () =>
          Effect.succeed(
            fail(
              "Validation failed",
              new FillingError({ cause: "ValidationFailed", details: transfer.validation })
            )
          ),

        Ready: () =>
          Effect.succeed(ok(CreateIntents(), "All fields ready, continuing..."))
      })
    },

    Validation: () => {
      if (transfer.validation._tag !== "Success") {
        return Effect.succeed(fail("Validation failed"))
      }
      return Effect.succeed(ok(CreateIntents(), "Validation passed"))
    },

    CreateIntents: () => Effect.succeed(ok(CheckBalance({intents}), "Checking balance")),

    CheckBalance: ({intents}) =>
      checkBalanceForIntents(source, intents).pipe(
        Effect.flatMap(hasEnough =>
          hasEnough
            ? Effect.succeed(ok(CheckAllowance(), "Checking allowance..."))
            : Effect.succeed(
              fail(
                "Insufficient funds",
                new InsufficientFundsError({cause: "Insufficient funds"})
              )
            )
        ),
        Effect.catchAll(error => Effect.succeed(fail("Balance check failed", error)))
      ),

    CheckAllowance: () =>
      checkAllowances(
        source,
        intents,
        sender,
        ucs03address
      ).pipe(
        Effect.map(allowancesOpt => {
          const allowances = Option.getOrElse(allowancesOpt, () => [])
          return ok(CreateOrders({allowances}), "Creating orders...")
        }),
        Effect.catchAll(error => Effect.succeed(fail("Allowance check failed", error)))
      ),

    CreateOrders: ({allowances}) =>
      createOrdersBatch(
        source,
        destination,
        channel,
        ucs03address,
        intents
      ).pipe(
        Effect.flatMap(batchOpt => {
          if (Option.isNone(batchOpt)) {
            return Effect.succeed(
              fail("Could not create orders", new OrderCreationError({details: "No batch returned"}))
            )
          }

          const batch = batchOpt.value
          return Effect.succeed(
            ok(CreateSteps({allowances, orders: [...batch.operand]}), "Building final steps...")
          )
        }),
        Effect.catchAll(error => Effect.succeed(fail("Order creation failed", error)))
      ),

    CreateSteps: ({allowances, orders}) =>
      Effect.succeed(complete("Transfer ready", orders, allowances))
  })
}
