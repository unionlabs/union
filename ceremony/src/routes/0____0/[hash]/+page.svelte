<script lang="ts">
import { page } from "$app/stores"
import TerminalWindow from "$lib/components/TerminalApp/TerminalWindow.svelte"
import { getState } from "$lib/state/index.svelte.ts"
import { onMount } from "svelte"
import { toast } from "svelte-sonner"
import { getUserContribution } from "$lib/supabase"
import Print from "$lib/components/TerminalApp/Print.svelte"

const { terminal } = getState()

let hash = $derived($page.params.hash)

onMount(() => {
  terminal.setHash(hash)
  terminal.setTab(4)
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

async function copyToClipboard(text: string, label: string) {
  try {
    await navigator.clipboard.writeText(text)
    toast.success(`Copied ${label}!`)
  } catch (err) {
    console.error("Failed to copy text: ", err)
    toast.error(`Failed to copy ${label} to clipboard.`)
  }
}

const imagePath = "https://ceremony.union.build/images/ceremony.png"
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

<TerminalWindow>
  {#await getUserContribution(hash)}
    <Print>Loading</Print>
  {:then contribution}
    {#if contribution}
      <Print>{hash}</Print>
      <Print class="mb-2">Public key</Print>
      <pre class="text-white whitespace-pre-wrap">{decodeHexString(contribution.public_key)}</pre>
      <Print class="mb-2">Signature</Print>
      <pre class="text-white whitespace-pre-wrap">{decodeHexString(contribution.signature)}</pre>
      <button class="block" onclick={() => copyToClipboard(decodeHexString(contribution.public_key), "public key")}>&gt
        copy public
        key
      </button>
      <button class="block" onclick={() => copyToClipboard(decodeHexString(contribution.signature), "signature")}>&gt
        copy signature
      </button>
    {/if}
  {/await}
</TerminalWindow>

<style>
    pre {
        white-space: pre-wrap;
        word-break: break-word;
        overflow-wrap: break-word;
    }
</style>
