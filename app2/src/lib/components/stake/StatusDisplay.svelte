<script lang="ts">
import { Match, Option as O, Data } from "effect"
import { wallets as WalletStore } from "$lib/stores/wallets.svelte"
import { getLastConnectedWalletId } from "$lib/wallet/evm/config.svelte"

type BondState = Data.TaggedEnum<{
  Ready: {}
  SwitchingChain: {}
  CheckingAllowance: {}
  ApprovingAllowance: {}
  AllowanceSubmitted: { txHash: string }
  WaitingForAllowanceConfirmation: { txHash: string }
  AllowanceApproved: {}
  CreatingTokenOrder: {}
  PreparingBondTransaction: {}
  ConfirmingBond: {}
  BondSubmitted: { txHash: string }
  WaitingForConfirmation: { txHash: string }
  WaitingForIndexer: { txHash: string }
  Success: { txHash: string }
  Error: { message: string }
}>

type UnbondState = Data.TaggedEnum<{
  Ready: {}
  SwitchingChain: {}
  CheckingAllowance: {}
  ApprovingAllowance: {}
  AllowanceSubmitted: { txHash: string }
  WaitingForAllowanceConfirmation: { txHash: string }
  AllowanceApproved: {}
  CreatingTokenOrder: {}
  PreparingUnbondTransaction: {}
  ConfirmingUnbond: {}
  UnbondSubmitted: { txHash: string }
  WaitingForConfirmation: { txHash: string }
  WaitingForIndexer: { txHash: string }
  Success: { txHash: string }
  Error: { message: string }
}>

type WithdrawalState = Data.TaggedEnum<{
  Ready: {}
  Loading: {}
  Success: {}
  Error: { message: string }
}>

type DustWithdrawState = Data.TaggedEnum<{
  Ready: {}
  SwitchingChain: {}
  CheckingAllowance: {}
  ApprovingAllowance: {}
  AllowanceSubmitted: { txHash: string }
  WaitingForAllowanceConfirmation: { txHash: string }
  AllowanceApproved: {}
  PreparingTransaction: {}
  ConfirmingWithdrawal: {}
  WithdrawalSubmitted: { txHash: string }
  WaitingForConfirmation: { txHash: string }
  WaitingForIndexer: { txHash: string }
  Success: { txHash: string }
  Error: { message: string }
}>

type QuickWithdrawState = Data.TaggedEnum<{
  Ready: {}
  SwitchingChain: {}
  CheckingAllowance: {}
  ApprovingAllowance: {}
  AllowanceSubmitted: { txHash: string }
  WaitingForAllowanceConfirmation: { txHash: string }
  AllowanceApproved: {}
  ConfirmingWithdraw: {}
  WithdrawSubmitted: { txHash: string }
  WaitingForConfirmation: { txHash: string }
  Success: { txHash: string; receivedAmount: bigint }
  Error: { message: string }
}>

interface Props {
  state: BondState | UnbondState | WithdrawalState | DustWithdrawState | QuickWithdrawState
  type: "bond" | "unbond" | "withdrawal" | "dust" | "quick-withdraw"
  inputAmount?: string
  class?: string
  size?: "default" | "compact"
}

let {
  state,
  type,
  inputAmount = "",
  class: className = "",
  size = "default",
}: Props = $props()

const isReady = $derived(state._tag === "Ready")
const isSuccess = $derived(state._tag === "Success")
const isError = $derived(state._tag === "Error")
const isActive = $derived(!isReady && !isSuccess && !isError)

const isSafeWallet = $derived(getLastConnectedWalletId() === "safe")
const txHash = $derived("txHash" in state ? O.some(state.txHash) : O.none())
const errorMessage = $derived("message" in state ? state.message : "An error occurred")

const getMessage = (type: string, state: any, inputAmount: string) => {
  const baseMessages: Record<string, Record<string, any>> = {
    bond: {
      Ready: {
        title: O.isNone(WalletStore.evmAddress)
          ? "Connect your wallet to start staking"
          : inputAmount
          ? `Ready to stake ${inputAmount} U`
          : "Enter amount to stake U tokens",
        subtitle: O.isNone(WalletStore.evmAddress)
          ? "Connect wallet to see balance and start staking"
          : inputAmount
          ? "Click stake button to begin transaction"
          : "Enter the amount of U tokens you want to stake",
        txHash: O.none()
      },
      SwitchingChain: {
        title: isSafeWallet ? "Preparing Safe Transaction" : "Switching to mainnet",
        subtitle: isSafeWallet ? "Preparing transaction for Safe wallet..." : "Please switch to mainnet in your wallet",
        txHash: O.none()
      },
      CheckingAllowance: {
        title: "Checking Token Allowance",
        subtitle: "Reading current token allowance from blockchain...",
        txHash: O.none()
      },
      ApprovingAllowance: {
        title: `Approving ${inputAmount || "0"} U`,
        subtitle: "Confirm token approval transaction in your wallet",
        txHash: O.none()
      },
      AllowanceSubmitted: {
        title: "Approval submitted",
        subtitle: "Allowance transaction submitted",
        txHash
      },
      WaitingForAllowanceConfirmation: {
        title: "Confirming submission",
        subtitle: "Waiting for allowance confirmation",
        txHash
      },
      AllowanceApproved: {
        title: `Approved ${inputAmount || "0"} U`,
        subtitle: "Token spending approved, proceeding...",
        txHash: O.none()
      },
      CreatingTokenOrder: {
        title: "Creating order",
        subtitle: "Building cross-chain token order...",
        txHash: O.none()
      },
      PreparingBondTransaction: {
        title: "Preparing bond",
        subtitle: "Preparing bond transaction with contracts...",
        txHash: O.none()
      },
      ConfirmingBond: {
        title: "Confirm bond",
        subtitle: "Confirm bond transaction in your wallet",
        txHash: O.none()
      },
      BondSubmitted: {
        title: "Bond successfully submitted",
        subtitle: "Transaction submitted",
        txHash
      },
      WaitingForConfirmation: {
        title: "Confirming submission",
        subtitle: "Waiting for confirmation",
        txHash
      },
      WaitingForIndexer: {
        title: "Indexing submission",
        subtitle: "Transaction confirmed, indexing data...",
        txHash
      },
      Success: {
        title: "Bond submitted",
        subtitle: "Your stake has been successfully submitted!",
        txHash
      },
      Error: {
        title: "Bond Failed",
        subtitle: errorMessage,
        txHash: O.none()
      }
    },
    unbond: {
      Ready: {
        title: O.isNone(WalletStore.evmAddress)
          ? "Connect your wallet to start unstaking"
          : inputAmount
          ? `Ready to unstake ${inputAmount} eU`
          : "Enter amount to unstake eU tokens",
        subtitle: O.isNone(WalletStore.evmAddress)
          ? "Connect wallet to see balance and start unstaking"
          : inputAmount
          ? "Click unstake button to begin transaction (27 day unbond period)"
          : "Enter the amount of eU tokens you want to unstake",
        txHash: O.none()
      },
      SwitchingChain: {
        title: isSafeWallet ? "Preparing Safe Transaction" : "Switching to mainnet",
        subtitle: isSafeWallet ? "Preparing transaction for Safe wallet..." : "Please switch to mainnet in your wallet",
        txHash: O.none()
      },
      CheckingAllowance: {
        title: "Checking Token Allowance",
        subtitle: "Reading current token allowance from blockchain...",
        txHash: O.none()
      },
      ApprovingAllowance: {
        title: `Approving ${inputAmount || "0"} eU`,
        subtitle: "Confirm token approval transaction in your wallet",
        txHash: O.none()
      },
      AllowanceSubmitted: {
        title: "Approval submitted",
        subtitle: "Allowance transaction submitted",
        txHash
      },
      WaitingForAllowanceConfirmation: {
        title: "Confirming submission",
        subtitle: "Waiting for allowance confirmation",
        txHash
      },
      AllowanceApproved: {
        title: `Approved ${inputAmount || "0"} eU`,
        subtitle: "Token spending approved, proceeding...",
        txHash: O.none()
      },
      CreatingTokenOrder: {
        title: "Creating order",
        subtitle: "Building cross-chain token order...",
        txHash: O.none()
      },
      PreparingUnbondTransaction: {
        title: "Preparing unbond",
        subtitle: "Preparing unbond transaction with contracts...",
        txHash: O.none()
      },
      ConfirmingUnbond: {
        title: "Confirm unbond",
        subtitle: "Confirm unbond transaction in your wallet",
        txHash: O.none()
      },
      UnbondSubmitted: {
        title: "Unbond successfully submitted",
        subtitle: "Transaction submitted",
        txHash
      },
      WaitingForConfirmation: {
        title: "Confirming submission",
        subtitle: "Waiting for confirmation",
        txHash
      },
      WaitingForIndexer: {
        title: "Indexing submission",
        subtitle: "Transaction confirmed, indexing data...",
        txHash
      },
      Success: {
        title: "Unbond submitted",
        subtitle: "Your unbond request has been successfully submitted!",
        txHash
      },
      Error: {
        title: "Unbond Failed",
        subtitle: errorMessage,
        txHash: O.none()
      }
    },
    withdrawal: {
      Ready: {
        title: O.isNone(WalletStore.evmAddress)
          ? "Connect your wallet to view withdrawals"
          : "Ready to withdraw",
        subtitle: O.isNone(WalletStore.evmAddress)
          ? "Connect wallet to see your withdrawable tokens"
          : "Withdraw your tokens to your wallet",
        txHash: O.none()
      },
      Loading: {
        title: "Processing withdrawal transaction",
        subtitle: "Please wait while we process your withdrawal...",
        txHash: O.none()
      },
      Success: {
        title: "Withdrawal completed successfully",
        subtitle: "Your tokens have been successfully withdrawn",
        txHash: O.none()
      },
      Error: {
        title: "Withdrawal failed",
        subtitle: errorMessage,
        txHash: O.none()
      }
    },
    dust: {
      Ready: {
        title: O.isNone(WalletStore.evmAddress)
          ? "Connect wallet to recover dust"
          : "Ready to recover dust",
        subtitle: O.isNone(WalletStore.evmAddress)
          ? "Connect wallet to recover your proxy dust"
          : "Recover eU tokens from your proxy contract",
        txHash: O.none()
      },
      SwitchingChain: {
        title: "Switching to Ethereum",
        subtitle: "Please confirm the network switch",
        txHash: O.none()
      },
      CheckingAllowance: {
        title: "Checking eU allowance",
        subtitle: "Verifying token permissions",
        txHash: O.none()
      },
      ApprovingAllowance: {
        title: "Approve eU spending",
        subtitle: "Confirm the approval in your wallet",
        txHash: O.none()
      },
      AllowanceSubmitted: {
        title: "Approval submitted",
        subtitle: "Transaction submitted",
        txHash
      },
      WaitingForAllowanceConfirmation: {
        title: "Confirming approval",
        subtitle: "Waiting for confirmation",
        txHash
      },
      AllowanceApproved: {
        title: "Approval confirmed",
        subtitle: "eU spending approved",
        txHash: O.none()
      },
      PreparingTransaction: {
        title: "Preparing dust recovery",
        subtitle: "Setting up the transaction",
        txHash: O.none()
      },
      ConfirmingWithdrawal: {
        title: "Confirm dust recovery",
        subtitle: "Confirm transaction in your wallet",
        txHash: O.none()
      },
      WithdrawalSubmitted: {
        title: "Dust recovery submitted",
        subtitle: "Transaction submitted",
        txHash
      },
      WaitingForConfirmation: {
        title: "Confirming recovery",
        subtitle: "Waiting for confirmation",
        txHash
      },
      WaitingForIndexer: {
        title: "Indexing recovery",
        subtitle: "Transaction confirmed, indexing data...",
        txHash
      },
      Success: {
        title: "Dust recovered successfully",
        subtitle: "Your eU tokens have been recovered to your wallet!",
        txHash
      },
      Error: {
        title: "Dust recovery failed",
        subtitle: errorMessage,
        txHash: O.none()
      }
    },
    "quick-withdraw": {
      Ready: {
        title: O.isNone(WalletStore.evmAddress)
          ? "Connect wallet for quick withdrawal"
          : inputAmount
          ? `Ready to quick withdraw ${inputAmount} eU`
          : "Enter amount for instant withdrawal",
        subtitle: O.isNone(WalletStore.evmAddress)
          ? "Connect wallet to use quick withdrawal"
          : inputAmount
          ? "Click withdraw button to begin instant conversion"
          : "Quick withdraw converts eU to U instantly",
        txHash: O.none()
      },
      SwitchingChain: {
        title: isSafeWallet ? "Preparing Safe Transaction" : "Switching to mainnet",
        subtitle: isSafeWallet ? "Preparing transaction for Safe wallet..." : "Please switch to mainnet in your wallet",
        txHash: O.none()
      },
      CheckingAllowance: {
        title: "Checking eU Allowance",
        subtitle: "Verifying token permissions for quick withdrawal...",
        txHash: O.none()
      },
      ApprovingAllowance: {
        title: `Approving ${inputAmount || "0"} eU`,
        subtitle: "Confirm token approval transaction in your wallet",
        txHash: O.none()
      },
      AllowanceSubmitted: {
        title: "Approval submitted",
        subtitle: "Allowance transaction submitted",
        txHash
      },
      WaitingForAllowanceConfirmation: {
        title: "Confirming approval",
        subtitle: "Waiting for allowance confirmation",
        txHash
      },
      AllowanceApproved: {
        title: `Approved ${inputAmount || "0"} eU`,
        subtitle: "Token spending approved, proceeding...",
        txHash: O.none()
      },
      ConfirmingWithdraw: {
        title: "Confirm quick withdrawal",
        subtitle: "Confirm withdrawal transaction in your wallet",
        txHash: O.none()
      },
      WithdrawSubmitted: {
        title: "Quick withdrawal submitted",
        subtitle: "Transaction submitted",
        txHash
      },
      WaitingForConfirmation: {
        title: "Processing withdrawal",
        subtitle: "Waiting for confirmation",
        txHash
      },
      Success: {
        title: "Quick withdrawal successful",
        subtitle: "Your U tokens have been sent to your wallet!",
        txHash
      },
      Error: {
        title: "Quick withdrawal failed",
        subtitle: errorMessage,
        txHash: O.none()
      }
    }
  }
  
  const messages = baseMessages[type] || {}
  return messages[state._tag] || { title: "Unknown state", subtitle: "Please refresh the page", txHash: O.none() }
}

const currentMessage = $derived(getMessage(type, state, inputAmount))

const containerPadding = $derived(size === "compact" ? "p-3" : "p-3.5")
const contentGap = $derived(size === "compact" ? "gap-2.5" : "gap-3")
const badgeSize = $derived(size === "compact" ? "size-6" : "size-7")
const titleClass = $derived(size === "compact" ? "text-[12px]" : "text-[13px]")
const subtitleSize = $derived(size === "compact" ? "text-[10px]" : "text-[11px]")
</script>

<div class="rounded-lg bg-zinc-900 border border-zinc-800/50 {containerPadding} {className}">
  <div class="flex items-center {contentGap}">
    <div class="{badgeSize} rounded-md {isError ? 'bg-red-500/10 border border-red-500/20' : isSuccess ? 'bg-accent/10 border border-accent/20' : isReady ? 'bg-zinc-800' : 'bg-accent/10 border border-accent/20'} flex items-center justify-center flex-shrink-0">
      {#if isReady}
        <svg
          class="w-3.5 h-3.5 text-zinc-400"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M13 16h-1v-4h-1m1-4h.01M21 12a9 9 0 11-18 0 9 9 0 0118 0z"
          />
        </svg>
      {:else if isActive}
        <svg class="animate-spin w-3.5 h-3.5 text-accent" xmlns="http://www.w3.org/2000/svg" fill="none" viewBox="0 0 24 24">
          <circle class="opacity-25" cx="12" cy="12" r="10" stroke="currentColor" stroke-width="4"></circle>
          <path class="opacity-75" fill="currentColor" d="M4 12a8 8 0 018-8V0C5.373 0 0 5.373 0 12h4zm2 5.291A7.962 7.962 0 014 12H0c0 3.042 1.135 5.824 3 7.938l3-2.647z"></path>
        </svg>
      {:else if isSuccess}
        <svg
          class="w-3.5 h-3.5 text-accent"
          fill="none"
          stroke="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            stroke-linecap="round"
            stroke-linejoin="round"
            stroke-width="2"
            d="M5 13l4 4L19 7"
          />
        </svg>
      {:else if isError}
        <svg
          class="w-3.5 h-3.5 text-red-400"
          fill="currentColor"
          viewBox="0 0 24 24"
        >
          <path
            d="M12 2L1 21h22L12 2zm0 4l7.53 13H4.47L12 6zm-1 5v4h2v-4h-2zm0 6v2h2v-2h-2z"
          />
        </svg>
      {/if}
    </div>
    <div class="flex-1">
      <div class="{titleClass} font-medium text-white">
        {currentMessage.title}
      </div>
      <div class="{subtitleSize} {isReady ? 'text-zinc-500' : isError ? 'text-red-400/80' : isSuccess ? 'text-accent/80' : 'text-accent/60'} mt-0.5">
        {currentMessage.subtitle}
        {#if O.isSome(currentMessage.txHash)}
          {" "}
          <a
            href="https://etherscan.io/tx/{O.getOrElse(currentMessage.txHash, () => "")}"
            target="_blank"
            rel="noopener noreferrer"
            class="underline hover:no-underline"
          >
            View
          </a>
        {/if}
      </div>
    </div>
  </div>
</div>
