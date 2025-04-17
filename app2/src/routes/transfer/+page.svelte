<script lang="ts">
import Sections from "$lib/components/ui/Sections.svelte"
import Transfer from "$lib/transfer/index.svelte"
import { Option } from "effect"
import { transferData } from "$lib/transfer/shared/data/transfer-data.svelte.ts"
import { tokensStore } from "$lib/stores/tokens.svelte.ts"
import { balancesStore } from "$lib/stores/balances.svelte.ts"

$effect(() => {
  if (Option.isSome(transferData.sourceChain)) {
    tokensStore.fetchTokens(transferData.sourceChain.value.universal_chain_id)
  }
})

let lastFetchKey = $state("")

$effect(() => {
  if (Option.isNone(transferData.sourceChain)) return

  const sourceChain = transferData.sourceChain.value

  const addressOption = transferData.derivedSender
  if (Option.isNone(addressOption)) return

  const address = addressOption.value

  const tokensOption = tokensStore.data.get(sourceChain.universal_chain_id) ?? Option.none()
  if (Option.isNone(tokensOption)) return

  const fetchKey = `${sourceChain.universal_chain_id}:${address}`

  if (fetchKey !== lastFetchKey) {
    const tokens = tokensOption.value
    const denoms = tokens.map(token => token.denom)

    balancesStore.fetchBalances(sourceChain, address, denoms)
    lastFetchKey = fetchKey
  }
})
</script>

<Sections>
  <Transfer mode="normal" />
</Sections>
