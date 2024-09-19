<script lang="ts">
import { toast } from "svelte-sonner"
import H1 from "$lib/components/typography/H1.svelte"
import Button from "$lib/components/Button.svelte"

let open = $state(false)

let command = "docker run -p 4919:4919 -it haitlahcen/union-mpc-cli:latest"

const copy = () => {
  navigator.clipboard.writeText(command)
  toast.success("Copied to clipboard", { position: "bottom-right" })
}
</script>

{#if open}
  <section class="fixed inset-0 h-full w-full bg-black flex justify-center z-50">
    <div class="max-w-4xl p-8">
      <H1>How to</H1>
      <code class="text-sm sm:text-base inline-flex text-left items-center space-x-4 bg-black text-white p-4 pl-6 font-mono">
          <span class="flex gap-4">
              <span class="shrink-0 text-union-accent-500">
                  $
              </span>

              <span class="flex-1">
                  <span>{command}</span>
              </span>
          </span>

        <Button onclick={copy} class="!bg-transparent text-neutral-500">
          <svg class="shrink-0 h-5 w-5" xmlns="http://www.w3.org/2000/svg"
               viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
            <path d="M8 2a1 1 0 000 2h2a1 1 0 100-2H8z"></path>
            <path
                    d="M3 5a2 2 0 012-2 3 3 0 003 3h2a3 3 0 003-3 2 2 0 012 2v6h-4.586l1.293-1.293a1 1 0 00-1.414-1.414l-3 3a1 1 0 000 1.414l3 3a1 1 0 001.414-1.414L10.414 13H15v3a2 2 0 01-2 2H5a2 2 0 01-2-2V5zM15 11h2a1 1 0 110 2h-2v-2z">
            </path>
          </svg>
        </Button>
      </code>
      <Button onclick={() => open = !open}>Close</Button>
    </div>
  </section>
{/if}

<Button onclick={() => open = !open}>Install Client</Button>


