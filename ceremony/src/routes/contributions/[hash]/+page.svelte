<script lang="ts">
import { getUserContribution } from "$lib/supabase"
import Spinner from "$lib/components/Spinner.svelte"
import H2 from "$lib/components/typography/H2.svelte"
import Button from "$lib/components/Button.svelte"
import { toast } from "svelte-sonner"
import { page } from "$app/stores"

let hash = $derived($page.params.hash)

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

const imagePath = "/images/ceremony.png"
let imageUrl = $derived(new URL(imagePath, $page.url.origin).href)
</script>

<svelte:head>
  <title>Union Ceremony</title>
  <meta name="description" content=""/>

  <meta property="og:title" content="Union Ceremony "/>
  <meta property="og:description" content=""/>
  <meta property="og:type" content="website"/>
  <meta property="og:url" content="https://ceremony.union.build"/>
  <meta property="og:site_name" content="Union Ceremony"/>
  <meta property="og:locale" content="en_US"/>
  <meta property="og:image" content={imageUrl}/>
  <meta property="og:image:secure_url" content={imageUrl}/>
  <meta property="og:image:type" content="image/png"/>
  <meta property="og:image:width" content="1200"/>
  <meta property="og:image:height" content="675"/>
  <meta property="og:image:alt" content="Union Ceremony event banner"/>

  <meta name="twitter:title" content=""/>
  <meta name="twitter:description" content=""/>
  <meta name="twitter:card" content="summary_large_image"/>
  <meta name="twitter:site" content="@union_build"/>
  <meta name="twitter:creator" content="@union_build"/>
  <meta name="twitter:image" content={imageUrl}/>
  <meta name="twitter:image:alt" content="Union Ceremony event banner"/>
</svelte:head>

<div class="w-full flex justify-center mt-[80px] pb-16">
  {#await getUserContribution(hash)}
    <Spinner class="size-5 text-union-accent-500"/>
  {:then contribution}
    {#if contribution}
      <div class="flex flex-col items-start gap-1 py-2 px-4">
        <div>
          <H2>Contributor: <span class="!text-union-accent-500">{contribution.user_name}</span></H2>
        </div>

        <div class="flex flex-col gap-4">
          <div>
            <H2 class="mb-2">Public key</H2>
            <pre class="text-white whitespace-pre-wrap bg-neutral-800 p-4 mb-4">{decodeHexString(contribution.public_key)}</pre>
            <Button onclick={() => copyToClipboard(decodeHexString(contribution.public_key), "public key")}>Copy
              Public
              key
            </Button>
          </div>

          <div>
            <H2 class="mb-2">Signature</H2>
            <pre class="text-white whitespace-pre-wrap bg-neutral-800 p-4 mb-4">{decodeHexString(contribution.signature)}</pre>
            <Button onclick={() => copyToClipboard(decodeHexString(contribution.signature), "signature")}>Copy
              Signature
            </Button>
          </div>
        </div>
      </div>
    {/if}
  {/await}
</div>

<style>
    pre {
        white-space: pre-wrap;
        word-break: break-word;
        overflow-wrap: break-word;
    }
</style>