export const BASE_URL = "https://app.union.build"

import { isValidBech32Address, isValidEvmAddress } from "@union/client"
import { page } from "$app/stores"
import Badge from "$lib/components/ui/badge/badge.svelte"

let searchInput = ""
let commandDialogOpen = false

/**
 * union14qemq0vw6y3gc3u3e0aty2e764u4gs5lnxk4rv
 * 0x8478B37E983F520dBCB5d7D3aAD8276B82631aBd
 * union1m87a5scxnnk83wfwapxlufzm58qe2v65985exff70z95a2yr86yq7hl08h
 */

function validAddress(address: string) {
  return isValidBech32Address(address) || isValidEvmAddress(address)
}

console.info(validAddress("stride1n4ccqcxw0k8q6mt72ytghvc57xfwqns6ll6k4k"))

console.info: () => $data.

)
