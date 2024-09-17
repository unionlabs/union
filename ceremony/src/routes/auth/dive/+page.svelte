<script lang="ts">
import H1 from "$lib/components/typography/H1.svelte"
import Button from "$lib/components/Button.svelte"
import Spinner from "$lib/components/Spinner.svelte"
import { page } from "$app/stores"
import { auth } from "$lib/utils/auth.ts"

let loading = false

async function login() {
  loading = true
  await auth(`${$page.url.origin}/`)
}
</script>

<div class="p-8  bg-gradient-to-t from-transparent via-black/50 to-transparent backdrop-blur w-full flex items-center flex-col">
  <Button onclick={login}>
    <span>Dive in using GitHub</span>
    {#if loading}
      <Spinner class="size-4"/>
    {/if}
  </Button>
</div>

