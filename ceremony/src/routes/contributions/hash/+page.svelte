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
