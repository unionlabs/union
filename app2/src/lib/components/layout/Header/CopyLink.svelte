<!-- this file is AI generated, feel free to ignore the quality !-->
<script lang="ts">
import Button from "$lib/components/ui/Button.svelte"

let copied = $state(false)

function copyToClipboard() {
  const currentUrl = window.location.href
  const modifiedUrl = getModifiedUrl(currentUrl, "app.union.build")

  navigator.clipboard
    .writeText(modifiedUrl)
    .then(() => {
      copied = true
      setTimeout(() => {
        copied = false
      }, 2000)
    })
    .catch(err => {
      console.error("Failed to copy: ", err)
    })
}

function getModifiedUrl(currentUrl: string, host: string): string {
  try {
    const url = new URL(currentUrl)
    url.host = host
    url.port = "" // Remove any port number
    return url.toString()
  } catch (e) {
    console.error("Error modifying URL:", e)
    return currentUrl
  }
}
</script>

<Button
  variant="icon"
  onclick={copyToClipboard}
  title="Copy link"
>
  {#if copied}
    <svg
      xmlns="http://www.w3.org/2000/svg"
      class="h-5 w-5 text-green-500"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <polyline points="20 6 9 17 4 12"></polyline>
    </svg>
  {:else}
    <svg
      xmlns="http://www.w3.org/2000/svg"
      class="h-5 w-5"
      viewBox="0 0 24 24"
      fill="none"
      stroke="currentColor"
      stroke-width="2"
      stroke-linecap="round"
      stroke-linejoin="round"
    >
      <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path>
      <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path>
    </svg>
  {/if}
</Button>
