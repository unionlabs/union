import {
  CosmosSwitchChainError,
  CosmosWalletNotConnectedError,
  CosmosWalletNotOnWindowError,
  CosmWasmError,
  GasPriceError,
  GetChainInfoError,
  NoCosmosChainInfoError,
  OfflineSignerError,
  switchChain,
} from "$lib/services/transfer-ucs03-cosmos"
import type { EffectToExit, HasKey } from "$lib/types"
import type { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate"
import { executeContract, ExecuteContractError } from "@unionlabs/sdk/cosmos"
import type { Chain } from "@unionlabs/sdk/schema"
import { Data, Effect, Exit, Match, pipe, Predicate, Schedule } from "effect"

export type TransactionState = Data.TaggedEnum<{
  Filling: {}
  SwitchChainInProgress: {}
  SwitchChainComplete: { exit: Effect.Effect.Success<ReturnType<typeof switchChain>> }
  WriteContractInProgress: { signingClient: SigningCosmWasmClient }
  WriteContractComplete: {
    signingClient: SigningCosmWasmClient
    exit: Effect.Effect.Success<ReturnType<typeof executeContract>>
  }
}>
type ExitStates = HasKey<TransactionState, "exit">

export const TransactionState = Data.taggedEnum<TransactionState>()
export const {
  SwitchChainInProgress,
  SwitchChainComplete,
  WriteContractInProgress,
  WriteContractComplete,
  $is: is,
} = TransactionState

export const nextState = (
  ts: TransactionState,
  chain: Chain,
  senderAddress: string,
  contractAddress: string,
  msg: Record<string, unknown>,
  funds?: ReadonlyArray<{ denom: string; amount: string }>,
): Effect.Effect<
  TransactionState,
  | CosmWasmError
  | CosmosWalletNotConnectedError
  | CosmosWalletNotOnWindowError
  | ExecuteContractError
  | GasPriceError
  | GetChainInfoError
  | NoCosmosChainInfoError
  | OfflineSignerError
  | CosmosSwitchChainError,
  never
> =>
  TransactionState.$match(ts, {
    Filling: () => Effect.succeed(SwitchChainInProgress()),

    SwitchChainInProgress: () =>
      pipe(
        switchChain(chain),
        Effect.map((exit) => SwitchChainComplete({ exit })),
      ),

    SwitchChainComplete: ({ exit }) =>
      Effect.succeed(WriteContractInProgress({ signingClient: exit.signingClient })),

    WriteContractInProgress: ({ signingClient }) =>
      Effect.gen(function*() {
        const retryableExecute = yield* executeContract(
          signingClient,
          senderAddress,
          contractAddress,
          msg,
          funds,
        ).pipe(
          // TODO: replace with retry-after header policy
          Effect.retry({
            while: error => error.message.includes("429"),
            schedule: Schedule.fibonacci("1 second"),
          }),
        )

        return WriteContractComplete({
          signingClient,
          exit: retryableExecute,
        })
      }),

    WriteContractComplete: ({ signingClient, exit }) => Effect.succeed(ts),
  })

export const toCtaText = (orElse: string) =>
  pipe(
    Match.type<TransactionState>(),
    Match.tags({
      WriteContractInProgress: () => "Confirming Transaction..." as const,
      SwitchChainInProgress: () => "Switching Chain..." as const,
    }),
    Match.orElse(() => orElse),
  )
