<script lang="ts">
import Button from "$lib/components/Button.svelte"
import H1 from "$lib/components/typography/H1.svelte"
import Text from "$lib/components/typography/Text.svelte"
import { ContributorState } from "$lib/stores/state.svelte.ts"
import { onMount } from "svelte"
import { user } from "$lib/stores/user.svelte.ts"
import { generateSecret } from "$lib/client"

type Props = {
  contributor: ContributorState
}

let { contributor }: Props = $props()

let download = $state(false)

function handleDownload(event: MouseEvent) {
  event.preventDefault()
  const newUrl = "http://localhost:4919/secret_key"
  window.open(newUrl, "_blank")
  download = true
}

function setDownloadedSecret() {
  localStorage.setItem("downloaded-secret", "true")
  contributor.downloadedSecret = true
}

onMount(async () => {
  const email = user.session?.user.email
  await generateSecret(email)
})
</script>

{#if !download}
  <H1>Generate your PGP secret</H1>
  <Text class="text-center mb-4">
    The MPC client automatically uses this secret to sign your contribution.<br>
    Your secret is locally generated through the MPC client.
  </Text>
  <Button variant="primary" onclick={handleDownload}>Generate secret</Button>
{:else}
  <H1>Store your PGP secret</H1>
  <Text class="text-center mb-4">
    Please store your secret somewhere safe, such as in your password manager.
    <br> Never share your secret.
    <br> This secret key is the only way to prove that you have contributed.
  </Text>
  <div class="flex gap-4">
    <Button variant="primary" onclick={setDownloadedSecret}>I've generated and stored my secret</Button>
    <Button variant="secondary" href="http://localhost:4919/secret_key" target="_blank">Generate again</Button>
  </div>
{/if}