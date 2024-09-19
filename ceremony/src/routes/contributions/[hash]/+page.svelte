<script lang="ts">
import { page } from "$app/stores"
import { getUserContribution } from "$lib/supabase"
import Spinner from "$lib/components/Spinner.svelte"
import H2 from "$lib/components/typography/H2.svelte"
import Button from "$lib/components/Button.svelte"
import { toast } from "svelte-sonner"

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
</script>

<div class="p-8">
  {#await getUserContribution($page.params.hash)}
    <Spinner class="size-5 text-union-accent-500"/>
  {:then contribution}
    {#if contribution}
      <div class="flex flex-col items-start gap-1 px-3 py-2">
        <div>
          <H2>Contributor: <span class="!text-union-accent-500">{contribution.user_name}</span></H2>
        </div>

        <div class="flex flex-col gap-4">
          <div>
            <H2 class="mb-2">Public key</H2>
            <pre class="text-white whitespace-pre-wrap bg-neutral-800 p-4 mb-4">{decodeHexString(contribution.public_key)}</pre>
            <Button onclick={() => copyToClipboard(decodeHexString(contribution.public_key), "public key")}>Copy Public
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