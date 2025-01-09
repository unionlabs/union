import { derived, type Readable, writable } from "svelte/store"
import type { UserAddresses } from "$lib/types.ts"
import { userAddrCosmos } from "$lib/wallet/cosmos"
import { userAddrEvm } from "$lib/wallet/evm"
import { userAddressAptos } from "$lib/wallet/aptos"
import type { QueryObserverResult } from "@tanstack/query-core"
import type { BalanceResult } from "$lib/queries/balance"

export let userAddress: Readable<UserAddresses> = derived(
  [userAddrCosmos, userAddrEvm, userAddressAptos],
  ([$userAddrCosmos, $userAddrEvm, $userAddressAptos]) => ({
    evm: $userAddrEvm,
    cosmos: $userAddrCosmos,
    aptos: $userAddressAptos
  })
)

export const balanceStore = writable<Array<QueryObserverResult<Array<BalanceResult>, Error>>>()
