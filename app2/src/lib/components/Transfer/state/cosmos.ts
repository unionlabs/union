import { Data, Effect, type Exit } from "effect"
import { switchChain } from "$lib/services/transfer-ucs03-cosmos"
import { executeContract } from "@unionlabs/sdk/cosmos"
import type { Chain } from "$lib/schema/chain.ts"
import type { SigningCosmWasmClient } from "@cosmjs/cosmwasm-stargate"

export type EffectToExit<T> = T extends Effect.Effect<infer A, infer E, any>
  ? Exit.Exit<A, E>
  : never

export type TransactionSubmissionCosmos = Data.TaggedEnum<{
  Filling: {}
  SwitchChainInProgress: {}
  SwitchChainComplete: { exit: EffectToExit<ReturnType<typeof switchChain>> }
  ExecuteContractInProgress: {}
  ExecuteContractComplete: { exit: EffectToExit<ReturnType<typeof executeContract>> }
}>

export const TransactionSubmissionCosmos = Data.taggedEnum<TransactionSubmissionCosmos>()
const {
  SwitchChainInProgress,
  SwitchChainComplete,
  ExecuteContractInProgress,
  ExecuteContractComplete
} = TransactionSubmissionCosmos

export const nextStateCosmos = async (
  ts: TransactionSubmissionCosmos,
  chain: Chain,
  signingClient: SigningCosmWasmClient,
  senderAddress: string,
  contractAddress: string,
  msg: Record<string, unknown>,
  funds?: ReadonlyArray<{ denom: string; amount: string }>
): Promise<TransactionSubmissionCosmos> =>
  TransactionSubmissionCosmos.$match(ts, {
    Filling: () => {
      console.log('[Filling] → Moving to SwitchChainInProgress');
      return SwitchChainInProgress();
    },
    SwitchChainInProgress: async () => {
      console.log('[SwitchChainInProgress] Starting chain switch to:', chain.universal_chain_id);
      try {
        const switchResult = await Effect.runPromiseExit(switchChain(chain));
        console.log('[SwitchChainInProgress] Chain switch result:', switchResult);
        return SwitchChainComplete({
          exit: switchResult
        });
      } catch (error) {
        console.error('[SwitchChainInProgress] Error during chain switch:', error);
        return SwitchChainComplete({
          exit: Effect.failCause(Cause.fail(error))
        });
      }
    },
    SwitchChainComplete: ({ exit }) => {
      if (exit._tag === "Failure") {
        console.error('[SwitchChainComplete] Chain switch failed with error:', exit.cause);
        console.log('[SwitchChainComplete] → Retrying SwitchChainInProgress');
        return SwitchChainInProgress();
      } else {
        console.log('[SwitchChainComplete] Chain switch successful. → Moving to ExecuteContractInProgress');
        return ExecuteContractInProgress();
      }
    },
    ExecuteContractInProgress: async () => {
      console.log('[ExecuteContractInProgress] Starting contract execution:', {
        contractAddress,
        msgType: Object.keys(msg)[0],
        senderAddress,
        fundsProvided: funds ? funds.map(f => `${f.amount} ${f.denom}`).join(', ') : 'none'
      });

      try {
        const executeResult = await Effect.runPromiseExit(
          executeContract(signingClient, senderAddress, contractAddress, msg, funds)
        );
        console.log('[ExecuteContractInProgress] Contract execution result:', executeResult);
        return ExecuteContractComplete({
          exit: executeResult
        });
      } catch (error) {
        console.error('[ExecuteContractInProgress] Error during contract execution:', error);
        return ExecuteContractComplete({
          exit: Effect.failCause(Cause.fail(error))
        });
      }
    },
    ExecuteContractComplete: ({ exit }) => {
      if (exit._tag === "Failure") {
        console.error('[ExecuteContractComplete] Contract execution failed with error:', exit.cause);
        console.log('[ExecuteContractComplete] → Retrying ExecuteContractInProgress');
        return ExecuteContractInProgress();
      } else {
        console.log('[ExecuteContractComplete] Contract execution successful. Transaction complete!');
        return ts;
      }
    }
  })

export const hasFailedExit = (state: TransactionSubmissionCosmos) =>
  "exit" in state && state.exit._tag === "Failure"

export const isComplete = (state: TransactionSubmissionCosmos) =>
  state._tag === "ExecuteContractComplete" && state.exit._tag === "Success"
