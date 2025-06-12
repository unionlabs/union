<script lang="ts">
import { goto } from "$app/navigation"
import { MAX_MOBILE_SIZE } from "$lib/constants"
import { vw } from "$lib/utils"
import { onMount } from "svelte"

const isMobile = vw() < MAX_MOBILE_SIZE

onMount(() => {
  if (isMobile) {
  } else {
    goto("/transfer", { replaceState: true })
  }
})
</script>
