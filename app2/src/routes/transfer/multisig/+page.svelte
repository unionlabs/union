<script lang="ts">
import Sections from "$lib/components/ui/Sections.svelte"
import VideoTutorial from "$lib/components/ui/VideoTutorial.svelte"
import { balancesStore } from "$lib/stores/balances.svelte.ts"
import { tokensStore } from "$lib/stores/tokens.svelte.ts"
import Transfer from "$lib/transfer/index.svelte"
import { transferData } from "$lib/transfer/shared/data/transfer-data.svelte.ts"
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

    balancesStore.fetchBalances(sourceChain, address, denoms)
    lastFetchKey = fetchKey
  }
})
</script>

<Sections>
  <Transfer mode="multisig" />

  <!-- Video Tutorial Popup -->
  <VideoTutorial
    title="Multisig Transfer Tutorial"
    description="Learn how to create and execute multisig transfers with Robert's comprehensive guide."
    videoUrl="https://www.youtube.com/watch?v=ajd3wHlyDYQ"
    class="fixed bottom-6 right-6 z-50 max-w-80"
  />
</Sections>
