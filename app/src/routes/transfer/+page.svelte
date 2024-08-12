<script lang="ts">
import { get, derived, writable, type Writable, type Readable } from "svelte/store"
import { custom, erc20Abi, parseUnits, getAddress, formatUnits, type Address } from "viem"
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
import { raise, sleep } from "$lib/utilities/index.ts"
import type { OfflineSigner } from "@leapwallet/types"
import { userBalancesQuery } from "$lib/queries/balance"
import { transferStateMachine } from "./state-machine.ts"
import * as Card from "$lib/components/ui/card/index.ts"
import type { Chain, UserAddresses } from "$lib/types.ts"
import { Input } from "$lib/components/ui/input/index.ts"
import { userAddrOnChain } from "$lib/utilities/address.ts"
import ChainsGate from "$lib/components/chains-gate.svelte"
import TransferForm from "./(components)/transfer-form.svelte"
</script>

<ChainsGate let:chains>
  <div class="w-full flex flex-col items-center">
    <TransferForm {chains}/>
  </div>
</ChainsGate>
