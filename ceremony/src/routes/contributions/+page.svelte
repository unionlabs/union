<script lang="ts">
import { getContributions, getUserContribution } from "$lib/supabase"
import Spinner from "$lib/components/Spinner.svelte"
import Text from "$lib/components/typography/Text.svelte"
import Blink from "$lib/components/Blink.svelte"
import H4 from "$lib/components/typography/H4.svelte"
import { page } from "$app/stores"
import H2 from "$lib/components/typography/H2.svelte"
import Button from "$lib/components/Button.svelte"

let hash = $derived($page.url.searchParams.get("hash"))

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

let intervalId: NodeJS.Timeout | number
let contributions = $state()

async function loadContributions() {
  contributions = await getContributions()
}

$effect(() => {
  loadContributions()
  intervalId = setInterval(loadContributions, 1000 * 5)

  return () => {
    if (intervalId) clearInterval(intervalId)
  }
})
</script>

{#if hash}
  {#await getUserContribution(hash)}
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
  {:else}
  {#if contributions}
    <div class="flex flex-col-reverse items-center h-svh overflow-y-auto pb-24 pt-36 w-full">
      <div class="w-full h-48 bg-gradient-to-b via-black from-black to-transparent absolute top-0"></div>
      <div class="flex flex-col items-center max-w-sm">
        {#each contributions as contribution, index }
          {#if index !== 0}
            <div class="h-4 w-[2px] bg-white"></div>
          {/if}
          <a href="/contributions?hash={contribution.public_key_hash}"
             class="text-white flex gap-1 items-center border-white border px-3 py-2 w-full">
            <img class="size-7" src={contribution.avatar_url} alt="">
            <Text class="uppercase max-w-48 truncate">{contribution.user_name}</Text>
          </a>
        {/each}
        <div class="h-4 w-[2px] bg-white"></div>
        <div class="text-white flex gap-2 items-center border-white border px-3 py-2 mb-16">
          <Spinner class="size-7 text-union-accent-500"/>
          <span>Next contribution...</span>
        </div>
        <H4>
          <Blink/>
        </H4>
      </div>
    </div>
  {:else}
    <Spinner class="size-5 text-union-accent-500"/>
  {/if}
{/if}
