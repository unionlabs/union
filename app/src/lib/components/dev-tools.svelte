<script lang="ts">
import { toast } from "svelte-sonner"
import { toggleMode } from "mode-watcher"
import { shortcut } from "@svelte-put/shortcut"
import { useQueryClient } from "@tanstack/svelte-query"

const queryClient = useQueryClient()
</script>

<svelte:window
  use:shortcut={{
    trigger: [
      /**
       * @note - only works in development mode
       * @dangerous - clears all cache and local storage
       */
      {
        key: "d",
        modifier: ["ctrl"],
        callback: _shortCutEventDetail => {
          if (import.meta.env.MODE !== "development") return
          console.info("Clearing cache and local storage")
          toast.warning("Clearing cache and local storage")
          queryClient.clear()
          queryClient.resetQueries()
          queryClient.removeQueries()
          queryClient.cancelQueries()
          queryClient.invalidateQueries()
          queryClient.unmount()
          if (typeof window === "undefined") return
          window.localStorage.clear()
          window.sessionStorage.clear()

          toast.success("Cache and local storage cleared")
        }
      },
      /**
       * `ctrl + t` toggles between light and dark mode
       */
      {
        key: "t",
        modifier: ["ctrl"],
        callback: _shortCutEventDetail => toggleMode()
      }
    ]
  }}
/>

{#if import.meta.env.MODE === "development"}
  <slot />
{/if}
