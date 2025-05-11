import { Data, Effect, Option, Match } from "effect"
import { constVoid } from "effect/Function"

export type AddCosmosWalletState = Data.TaggedEnum<{
  Idle: {}
  Connecting: { 
    walletType: 'keplr' | 'leap'
  }
  SwitchChain: { 
    walletType: 'keplr' | 'leap'
    chainId: string 
  }
  Signing: { 
    walletType: 'keplr' | 'leap'
    address: string
    chainId: string
  }
  Verifying: {
    walletType: 'keplr' | 'leap'
    address: string
    chainId: string
    signature: string
    message: string
  }
  Adding: {
    walletType: 'keplr' | 'leap'
    address: string
    chainId: string
    signature: string
    message: string
  }
  Success: { address: string }
  Error: { 
    message: string
    cause?: unknown 
  }
}>

export const AddCosmosWalletState = Data.taggedEnum<AddCosmosWalletState>()
const {
  Idle,
  Connecting,
  SwitchChain,
  Signing,
  Verifying,
  Adding,
  Success,
  Error
} = AddCosmosWalletState

export type StateResult = {
  nextState: Option.Option<AddCosmosWalletState>
  message: string
  error: Option.Option<Error>
}

const fail = (msg: string, err?: Error): StateResult => ({
  nextState: Option.none(),
  message: msg,
  error: err ? Option.some(err) : Option.none(),
})

const ok = (state: AddCosmosWalletState, msg: string): StateResult => ({
  nextState: Option.some(state),
  message: msg,
  error: Option.none(),
})

const complete = (address: string): StateResult => ({
  nextState: Option.some(Success({ address })),
  message: "Wallet added successfully",
  error: Option.none(),
})

export const addCosmosWalletState = (state: AddCosmosWalletState) => {
  return AddCosmosWalletState.$match(state, {
    Idle: () => {
      console.log("Idle state")
      return Effect.succeed(ok(Connecting({ walletType: 'keplr' }), "Connecting to wallet..."))
    },
    Connecting: ({ walletType }) => {
      console.log("Connecting state", { walletType })
      return Effect.succeed(ok(SwitchChain({ 
        walletType, 
        chainId: "cosmoshub-4" 
      }), "Switching chain..."))
    },
    SwitchChain: ({ walletType, chainId }) => {
      console.log("SwitchChain state", { walletType, chainId })
      return Effect.succeed(ok(Signing({ 
        walletType, 
        address: "cosmos...", 
        chainId 
      }), "Signing message..."))
    },
    Signing: ({ walletType, address, chainId }) => {
      console.log("Signing state", { walletType, address, chainId })
      return Effect.succeed(ok(Verifying({ 
        walletType,
        address, 
        chainId, 
        signature: "0x...", 
        message: "..." 
      }), "Verifying signature..."))
    },
    Verifying: ({ walletType, address, chainId, signature, message }) => {
      console.log("Verifying state", { walletType, address, chainId, signature, message })
      return Effect.succeed(ok(Adding({ 
        walletType,
        address, 
        chainId, 
        signature, 
        message 
      }), "Adding wallet..."))
    },
    Adding: ({ walletType, address, chainId, signature, message }) => {
      console.log("Adding state", { walletType, address, chainId, signature, message })
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
