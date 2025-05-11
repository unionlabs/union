import { Data, Effect, Option, Match } from "effect"
import { constVoid } from "effect/Function"
import { switchChain } from "$lib/services/transfer-ucs03-evm"
import { getLastConnectedWalletId } from "$lib/wallet/evm/config.svelte.ts"
import { writeContract, waitForTransactionReceipt } from "@unionlabs/sdk/evm"
import { ViemPublicClient } from "@unionlabs/sdk/evm"
import type { Chain, PublicClient, WalletClient, Hash } from "viem"

export type AddEvmWalletState = Data.TaggedEnum<{
  Idle: {}
  Connecting: {}
  SwitchChain: { 
    chainId: number
  }
  Signing: { 
    address: string 
    chainId: number
  }
  Verifying: { 
    address: string
    chainId: number
    signature: string
    message: string
  }
  Adding: {
    address: string
    chainId: number
    signature: string
    message: string
  }
  Success: { address: string }
  Error: { 
    message: string
    cause?: unknown 
  }
}>

export const AddEvmWalletState = Data.taggedEnum<AddEvmWalletState>()
const {
  Idle,
  Connecting,
  SwitchChain,
  Signing,
  Verifying,
  Adding,
  Success,
  Error
} = AddEvmWalletState

export type StateResult = {
  nextState: Option.Option<AddEvmWalletState>
  message: string
  error: Option.Option<Error>
}

const fail = (msg: string, err?: Error): StateResult => ({
  nextState: Option.none(),
  message: msg,
  error: err ? Option.some(err) : Option.none(),
})

const ok = (state: AddEvmWalletState, msg: string): StateResult => ({
  nextState: Option.some(state),
  message: msg,
  error: Option.none(),
})

const complete = (address: string): StateResult => ({
  nextState: Option.some(Success({ address })),
  message: "Wallet added successfully",
  error: Option.none(),
})

export const addEvmWalletState = (
  state: AddEvmWalletState,
  chain: Chain,
  publicClient: PublicClient,
  walletClient: WalletClient
) => {
  return AddEvmWalletState.$match(state, {
    Idle: () => {
      console.log("Idle state")
      return Effect.succeed(ok(Connecting(), "Connecting to wallet..."))
    },
    Connecting: () => {
      console.log("Connecting state")
      return Effect.succeed(ok(SwitchChain({ chainId: chain.id }), "Switching chain..."))
    },
    SwitchChain: ({ chainId }) => {
      console.log("SwitchChain state", { chainId })
      const isSafeWallet = getLastConnectedWalletId() === "safe"
      
      return Effect.flatMap(
        Effect.tryPromise(() => 
          isSafeWallet 
            ? Promise.resolve() 
            : Effect.runPromise(switchChain(chain))
        ),
        () => Effect.succeed(ok(Signing({ 
          address: walletClient.account.address, 
          chainId 
        }), "Signing message..."))
      )
    },
    Signing: ({ address, chainId }) => {
      console.log("Signing state", { address, chainId })
      // Here we would implement the actual signing logic
      const message = "Sign this message to verify wallet ownership"
      return Effect.succeed(ok(Verifying({ 
        address, 
        chainId, 
        signature: "0x...", // This would be the actual signature
        message 
      }), "Verifying signature..."))
    },
    Verifying: ({ address, chainId, signature, message }) => {
      console.log("Verifying state", { address, chainId, signature, message })
      // Here we would implement the actual verification logic
      return Effect.succeed(ok(Adding({ 
        address, 
        chainId, 
        signature, 
        message 
      }), "Adding wallet..."))
    },
    Adding: ({ address, chainId, signature, message }) => {
      console.log("Adding state", { address, chainId, signature, message })
      // Here we would implement the actual wallet addition logic
      return Effect.succeed(complete(address))
    },
    Success: ({ address }) => {
      console.log("Success state", { address })
      return Effect.succeed(complete(address))
    },
    Error: ({ message, cause }) => {
      console.log("Error state", { message, cause })
      return Effect.succeed(fail(message, cause as Error))
    }
  })
}
