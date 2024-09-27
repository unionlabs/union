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

const imagePath = "https://ceremony.union.build/images/ceremony.png"

function trigger(value: "key" | "signature") {
  if (value === "key") {
    copyToClipboard(decodeHexString(contribution.public_key), "public key")
  } else if (value === "signature") {
    copyToClipboard(decodeHexString(contribution.signature), "signature")
  }
}
</script>

<svelte:head>
  <title>Union Ceremony</title>
  <meta name="description"
        content="Ceremony to generate trustworthy cryptographic keys for securing the Union zero-knowledge system."/>

  <meta property="og:title" content="Union Ceremony "/>
  <meta property="og:description"
        content="Ceremony to generate trustworthy cryptographic keys for securing the Union zero-knowledge system."/>
  <meta property="og:type" content="website"/>
  <meta property="og:url" content="https://ceremony.union.build"/>
  <meta property="og:site_name" content="Union Ceremony"/>
  <meta property="og:locale" content="en_US"/>
  <meta property="og:image" content={imagePath}/>
  <meta property="og:image:secure_url" content={imagePath}/>
  <meta property="og:image:type" content="image/png"/>
  <meta property="og:image:width" content="1200"/>
  <meta property="og:image:height" content="675"/>
  <meta property="og:image:alt" content="Union Ceremony event banner"/>

  <meta name="twitter:title"
        content="Ceremony to generate trustworthy cryptographic keys for securing the Union zero-knowledge system."/>
  <meta name="twitter:description"
        content="Ceremony to generate trustworthy cryptographic keys for securing the Union zero-knowledge system."/>
  <meta name="twitter:card" content="summary_large_image"/>
  <meta name="twitter:site" content="@union_build"/>
  <meta name="twitter:creator" content="@union_build"/>
  <meta name="twitter:image" content={imagePath}/>
  <meta name="twitter:image:alt" content="Union Ceremony event banner"/>
</svelte:head>


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
