<script lang="ts">
import Sections from "$lib/components/ui/Sections.svelte"
import { balancesStore } from "$lib/stores/balances.svelte"
import { tokensStore } from "$lib/stores/tokens.svelte"
import Transfer from "$lib/transfer/index.svelte"
import { transferData } from "$lib/transfer/shared/data/transfer-data.svelte"
import { Ucs05 } from "@unionlabs/sdk"
import { Option } from "effect"

$effect(() => {
  if (Option.isSome(transferData.sourceChain)) {
    tokensStore.fetchTokens(transferData.sourceChain.value.universal_chain_id)
  }
})

let lastFetchKey = $state("")

$effect(() => {
  if (Option.isNone(transferData.sourceChain)) {
    return
  }

  const sourceChain = transferData.sourceChain.value

  const addressOption = transferData.derivedSender
  if (Option.isNone(addressOption)) {
    return
  }

  const address = addressOption.value

  const tokensOption = tokensStore.data.get(sourceChain.universal_chain_id) ?? Option.none()
  if (Option.isNone(tokensOption)) {
    return
  }

  const fetchKey = `${sourceChain.universal_chain_id}:${address}`

  if (fetchKey !== lastFetchKey) {
    const tokens = tokensOption.value
    const denoms = tokens.map(token => token.denom)

    // TODO: replace with SDK
    balancesStore.fetchBalances(sourceChain, Ucs05.anyDisplayToCanonical(address), denoms)
    lastFetchKey = fetchKey
  }
})
</script>

<Sections>
  <Transfer mode="normal" />
</Sections>
