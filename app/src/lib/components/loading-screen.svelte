<script>
import { fade } from "svelte/transition"
import Logo from "./logo.svelte"
</script>

<div
  class="w-screen h-screen flex justify-center items-center fixed z-[9999] bg-background"
  out:fade|local
>
<Logo class="size-16"/>
</div>