<script lang="ts">
import H1 from "$lib/components/typography/H1.svelte"
import Text from "$lib/components/typography/Text.svelte"
import Button from "$lib/components/Button.svelte"
import Link from "$lib/components/typography/Link.svelte"
import Spinner from "$lib/components/Spinner.svelte"
import { page } from "$app/stores"
import { auth } from "$lib/utils/auth.ts"

let loading = false

async function login() {
  loading = true
  await auth(`${$page.url.origin}/app`)
}
</script>

<div class="p-8 border border-neutral-500 bg-background-light">
  <H1 class="mb-4">Login</H1>
  <Button onclick={login}>
    <span>Sign in with GitHub</span>
    {#if loading}
      <Spinner class="size-4"/>
    {/if}
  </Button>
  <Text class="mt-4 text-sm text-neutral-600">
    Don't have an account? <Link href="/auth/register">Register here</Link>.
  </Text>
</div>

