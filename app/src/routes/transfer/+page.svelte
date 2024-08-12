<script lang="ts">
<<<<<<< HEAD
import { get, derived, writable, type Writable, type Readable } from "svelte/store"
import { custom, erc20Abi, parseUnits, getAddress, formatUnits, type Address } from "viem"
=======
>>>>>>> 343d7e6b2 (chore: progress)
import {
  switchChain,
  writeContract,
  simulateContract,
  getConnectorClient,
  waitForTransactionReceipt
} from "@wagmi/core"
import {
  cosmosHttp,
  createPfmMemo,
  bytesToBech32Address,
  createCosmosSdkClient,
  type TransactionResponse,
  type TransferAssetsParameters
} from "@union/client"
import { onMount } from "svelte"
import { page } from "$app/stores"
import { toast } from "svelte-sonner"
import { goto } from "$app/navigation"
import { useMachine } from "@xstate/svelte"
import { ucs01abi } from "$lib/abi/ucs-01.ts"
import { cn } from "$lib/utilities/shadcn.ts"
import { cosmosStore } from "$lib/wallet/cosmos"
import Chevron from "./(components)/chevron.svelte"
<<<<<<< HEAD
=======
import Transfer from "./(components)/transfer.svelte"
>>>>>>> 343d7e6b2 (chore: progress)
import { raise, sleep } from "$lib/utilities/index.ts"
import type { OfflineSigner } from "@leapwallet/types"
import { userBalancesQuery } from "$lib/queries/balance"
import { transferStateMachine } from "./state-machine.ts"
import * as Card from "$lib/components/ui/card/index.ts"
import type { Chain, UserAddresses } from "$lib/types.ts"
import { Input } from "$lib/components/ui/input/index.ts"
import { userAddrOnChain } from "$lib/utilities/address.ts"
import ChainsGate from "$lib/components/chains-gate.svelte"
<<<<<<< HEAD
import TransferForm from "./(components)/transfer-form.svelte"
</script>

<ChainsGate let:chains>
  <div class="w-full flex flex-col items-center">
    <TransferForm {chains}/>
  </div>
=======
import { createBrowserInspector } from "@statelyai/inspect"
import WalletGate from "$lib/components/wallet-gate.svelte"
import { Button } from "$lib/components/ui/button/index.ts"
import { getSupportedAsset } from "$lib/utilities/helpers.ts"
import ArrowLeftRight from "virtual:icons/lucide/arrow-left-right"
import { getCosmosChainInfo } from "$lib/wallet/cosmos/chain-info.ts"
import { sepoliaStore, wagmiConfig, evmConnect } from "$lib/wallet/evm"
import { submittedTransfers } from "$lib/stores/submitted-transfers.ts"
import CardSectionHeading from "./(components)/card-section-heading.svelte"
import { sepolia, berachainTestnetbArtio, arbitrumSepolia } from "viem/chains"
import { get, derived, writable, type Writable, type Readable } from "svelte/store"
import { custom, erc20Abi, parseUnits, getAddress, formatUnits, type Address } from "viem"
import ChainButton from "./(components)/chain-button.svelte"

// const { inspect, ...inspector } = createBrowserInspector({
//   autoStart: import.meta.env.DEV && import.meta.env.DEBUG_XSTATE === "true"
// })

// const { snapshot, send } = useMachine(transferStateMachine, {
//   inspect,
//   input: {
//     cosmosStore: $cosmosStore,
//     sepoliaStore: $sepoliaStore
//   }
// })

// $: console.info($snapshot.context)
</script>

<ChainsGate let:chains let:rawChains>
  <WalletGate let:connected let:userAddr>
    <div class="w-full flex flex-col items-center">
      <Transfer chains={rawChains} {connected} userAddresses={userAddr} />
    </div>
  </WalletGate>
>>>>>>> 343d7e6b2 (chore: progress)
</ChainsGate>
