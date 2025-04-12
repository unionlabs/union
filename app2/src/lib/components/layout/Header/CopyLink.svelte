<script lang="ts">
let isOpen = false

function toggleDropdown() {
  isOpen = !isOpen
}

function closeDropdown() {
  isOpen = false
}

function copyToClipboard(host: string) {
  const currentUrl = window.location.href
  const modifiedUrl = getModifiedUrl(currentUrl, host)

  navigator.clipboard
    .writeText(modifiedUrl)
    .then(() => {
      closeDropdown()
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

<div class="relative">
  <!-- svelte-ignore a11y_consider_explicit_label -->
  <button 
    class="flex cursor-pointer items-center gap-1 text-sm rounded hover:bg-zinc-800 transition-colors"
    onclick={toggleDropdown}
  >
    <svg xmlns="http://www.w3.org/2000/svg" class="h-4 w-4" viewBox="0 0 24 24" fill="none" stroke="currentColor" stroke-width="2" stroke-linecap="round" stroke-linejoin="round">
      <path d="M10 13a5 5 0 0 0 7.54.54l3-3a5 5 0 0 0-7.07-7.07l-1.72 1.71"></path>
      <path d="M14 11a5 5 0 0 0-7.54-.54l-3 3a5 5 0 0 0 7.07 7.07l1.71-1.71"></path>
    </svg>
  </button>
  
  {#if isOpen}
    <div class="absolute right-0 mt-1 w-48 bg-zinc-900 border border-zinc-800 rounded-md shadow-lg z-20">
      <div class="py-1">
        <button 
          class="cursor-pointer px-4 py-2 w-full text-left text-sm hover:bg-zinc-800 transition-colors"
          onclick={() => copyToClipboard('app2.union.build')}
        >
          Production
        </button>
        <button 
          class="w-full text-left px-4 py-2 cursor-pointer text-sm hover:bg-zinc-800 transition-colors"
          onclick={() => copyToClipboard('staging.app2.union.build')}
        >
          Staging
        </button>
      </div>
    </div>
  {/if}
</div>
