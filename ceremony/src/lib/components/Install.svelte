<script lang="ts">
  import { toast } from "svelte-sonner"
  import H1 from "$lib/components/typography/H1.svelte"
  import Button from "$lib/components/Button.svelte"

  let command = "docker run -p 4919:4919 -it haitlahcen/union-mpc-cli:latest"
  const copy = () => {
    navigator.clipboard.writeText(command)
    toast.success("Copied to clipboard", { position: "bottom-right" })
  }
</script>

<div class="p-8  bg-gradient-to-t from-transparent via-black/50 to-transparent backdrop-blur w-full flex items-center flex-col">
  <H1 class="text-center mb-4" >Install client</H1>
  <code class="text-sm sm:text-base inline-flex text-left items-center space-x-4 bg-white text-black p-4 pl-6 font-mono">
    <span class="flex gap-4">
        <span class="shrink-0">
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
</div>

