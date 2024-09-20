<script lang="ts">
import Text from "$lib/components/typography/Text.svelte"
import H1 from "$lib/components/typography/H1.svelte"
import type { ContributorState } from "$lib/stores/state.svelte.ts"
import Button from "$lib/components/Button.svelte"
import { toast } from "svelte-sonner"
import { isSafari } from "$lib/utils/utils.ts"

type Props = {
  contributor: ContributorState
}
let { contributor }: Props = $props()

let command =
  "mkdir -p ceremony && docker pull ghcr.io/unionlabs/union/mpc-client:latest && docker run -v $(pwd)/ceremony:/ceremony -w /ceremony -p 4919:4919 --rm -it ghcr.io/unionlabs/union/mpc-client:latest"

const copy = () => {
  navigator.clipboard.writeText(command)
  toast.success("Copied to clipboard", { position: "bottom-right" })
}
</script>

{#if contributor}
  <div class="flex flex-col items-center text-center mb-4">

    <H1 class="mb-4">Run the MPC client</H1>
    <Text>
      You must have docker installed in order to contribute. <br> On linux, install docker through your package manager, and skip to step 5.
      <br>
      On macOS, we highly recommend
      <a href="https://orbstack.dev/"
         class="underline underline-offset-4 decoration-union-accent-500"
         target="_blank">OrbStack</a>
      because <span class="text-rose-400">Docker Desktop is too slow.</span>
      <br>
      <strong>
        If you use Docker Desktop it is extremely likely that you will <span class="text-rose-400">lose your contribution slot.</span>
      </strong>
    </Text>
    <Text class="mt-4 !text-union-accent-500">
      <a href="https://orbstack.dev/">1. Install OrbStack</a>
    </Text>
    <Text class="mt-4 !text-union-accent-500">
      2. Open OrbStack from the Applications/ folder
    </Text>
    <Text class="mt-4 !text-union-accent-500">
      3. Click allow on the OrbStack popups
    </Text>
    <Text class="mt-4 !text-union-accent-500">
      4. Open Terminal from the Applications/Utilities/ folder
    </Text>
    <Text class="mt-4 !text-union-accent-500">
      5. Paste the following command in Terminal to start the MPC client:
    </Text>
    <div class="max-w-4xl p-8">
      <button onclick={copy}>
        <code
              class="cursor-pointer text-sm sm:text-base inline-flex text-left items-center space-x-4 bg-black text-white p-4 pl-6 font-mono border-white border">
          <span class="flex gap-4">
              <span class="shrink-0 text-union-accent-500 select-none">
                  $
              </span>
              <span class="flex-1">
                  <span>{command}</span>
              </span>
          </span>
          <svg class="shrink-0 h-5 w-5" xmlns="http://www.w3.org/2000/svg"
               viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
            <path d="M8 2a1 1 0 000 2h2a1 1 0 100-2H8z"></path>
            <path
                    d="M3 5a2 2 0 012-2 3 3 0 003 3h2a3 3 0 003-3 2 2 0 012 2v6h-4.586l1.293-1.293a1 1 0 00-1.414-1.414l-3 3a1 1 0 000 1.414l3 3a1 1 0 001.414-1.414L10.414 13H15v3a2 2 0 01-2 2H5a2 2 0 01-2-2V5zM15 11h2a1 1 0 110 2h-2v-2z">
            </path>
          </svg>

        </code>
      </button>
    </div>
    <Text>
      Once the MPC client is running you can return to this page.
    </Text>
    <Text class="mb-4">
      If the MPC client is running but you still see this page, ensure that you are using either Chrome, FireFox or Brave.
      <br>
      For Brave, disable the shields in the address bar.
    </Text>
    {#if isSafari()}
      <div class="border border-rose-400 bg-rose-400/10 text-rose-400 py-2 px-3">
        Safari is not supported
      </div>
    {/if}
  </div>
{/if}
