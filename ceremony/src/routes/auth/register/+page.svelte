<script lang="ts">
import { supabase } from "$lib/supabase/client.ts"
import H1 from "$lib/components/typography/H1.svelte"
import Text from "$lib/components/typography/Text.svelte"
import Button from "$lib/components/Button.svelte"
import Link from "$lib/components/typography/Link.svelte"
import Spinner from "$lib/components/Spinner.svelte"
import { page } from "$app/stores"
import { auth } from "$lib/utils/auth.ts"

let loading = false

async function register() {
  loading = true
  await auth(`${$page.url.origin}/app`)
}
</script>

<div class="p-8  bg-gradient-to-t from-transparent via-black/50 to-transparent backdrop-blur w-full flex items-center flex-col">
  <H1 class="mb-4">Register</H1>
  <Button onclick={register}>
    <span>Register with GitHub</span>
    {#if loading}
      <Spinner class="size-4"/>
    {/if}
  </Button>
  <Text class="mt-4 text-sm text-neutral-600">
    Already have an account? <Link href="/auth/login">Login here</Link>.
  </Text>
</div>
