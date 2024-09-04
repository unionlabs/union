<script lang="ts">
  import {toast} from "svelte-sonner";

  let command = "docker run -p 4919:4919 -it haitlahcen/union-mpc-cli:latest"
  const copy = () => {
    navigator.clipboard.writeText(command)
    toast.success("Copied to clipboard", {position: "bottom-right"})
  }
</script>

<div class="max-w-7xl sm:px-6 lg:px-8">
  <h1 class="text-lg text-center font-bold mb-4" >INSTALL CLIENT</h1>
  <code class="text-sm sm:text-base inline-flex text-left items-center space-x-4 bg-black text-white p-4 pl-6 font-mono">
    <span class="flex gap-4">
        <span class="shrink-0 text-accent-500">
            $
        </span>

        <span class="flex-1">
            <span>{command}</span>
        </span>
    </span>

    <button on:click={copy} class="p-1 text-neutral-500 hover:bg-accent-500 hover:text-black">
      <svg class="shrink-0 h-5 w-5" xmlns="http://www.w3.org/2000/svg"
           viewBox="0 0 20 20" fill="currentColor" aria-hidden="true">
        <path d="M8 2a1 1 0 000 2h2a1 1 0 100-2H8z"></path>
        <path
                d="M3 5a2 2 0 012-2 3 3 0 003 3h2a3 3 0 003-3 2 2 0 012 2v6h-4.586l1.293-1.293a1 1 0 00-1.414-1.414l-3 3a1 1 0 000 1.414l3 3a1 1 0 001.414-1.414L10.414 13H15v3a2 2 0 01-2 2H5a2 2 0 01-2-2V5zM15 11h2a1 1 0 110 2h-2v-2z">
        </path>
      </svg>
    </button>
  </code>
</div>

