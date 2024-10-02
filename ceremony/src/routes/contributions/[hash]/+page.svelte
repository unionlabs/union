<script lang="ts">
import { page } from "$app/stores"
import { getUserContribution } from "$lib/supabase"
import Print from "$lib/components/Terminal/Print.svelte"
import { sleep } from "$lib/utils/utils.ts"
import Buttons from "$lib/components/Terminal/Install/Buttons.svelte"
import { onMount } from "svelte"
import { getState } from "$lib/state/index.svelte.ts"

let hash = $derived($page.params.hash)
let prints = $state<Array<string>>([])

const { terminal } = getState()
let contribution = $state()

onMount(async () => {
  terminal.clearHistory()
  contribution = await getUserContribution(hash)
  terminal.setTab(4)
  terminal.setHash(hash)
})

function hexToUint8Array(hexString: string) {
  return new Uint8Array(hexString.match(/.{1,2}/g)?.map(byte => Number.parseInt(byte, 16)) || [])
}

function uint8ArrayToUtf8(bytes: Uint8Array) {
  return new TextDecoder().decode(bytes)
}

function decodeHexString(hexString: string) {
  return uint8ArrayToUtf8(hexToUint8Array(hexString))
}

async function copyToClipboard(text: string, type: string) {
  prints.push(`Copying ${type}..`)
  await sleep(1000)
  await navigator.clipboard.writeText(text)
  prints.push(`Successfully copied ${type}`)
}

function trigger(value: "key" | "signature") {
  if (value === "key") {
    copyToClipboard(decodeHexString(contribution.public_key), "public key")
  } else if (value === "signature") {
    copyToClipboard(decodeHexString(contribution.signature), "signature")
  }
}
</script>

{#if contribution}
  <pre class="text-white whitespace-pre-wrap text-sm sm:text-base">{decodeHexString(contribution?.public_key)}</pre>
  <pre class="text-white whitespace-pre-wrap text-sm sm:text-base">{decodeHexString(contribution?.signature)}</pre>
  {#each prints as print}
    <Print>{print}</Print>
  {/each}
  <Print><br></Print>
  <Buttons
          data={[{text: "Copy public key", action: 'key'}, {text: "Copy signature", action: 'signature'}]}
          trigger={(value: 'key' | 'signature') => trigger(value)}
  />
{:else}
  <Print>Loading</Print>
{/if}

<style>
    pre {
        white-space: pre-wrap;
        word-break: break-word;
        overflow-wrap: break-word;
    }
</style>
