<script lang="ts">
import { toggleMode } from "mode-watcher"
import Sun from "virtual:icons/lucide/sun"
import Moon from "virtual:icons/lucide/moon"
import { Button } from "$lib/components/ui/button/index.ts"
</script>

<Button on:click={toggleMode} variant="outline" size="icon" class='outline-none border-none hover:bg-transparent'>
  <Sun
    class="size-[1.3rem] rotate-0 scale-100 transition-all dark:-rotate-90 dark:scale-0"
  />
  <Moon
    class="absolute size-[1.3rem] rotate-90 scale-0 transition-all dark:rotate-0 dark:scale-100"
  />
  <span class="sr-only">Toggle theme</span>
</Button>
