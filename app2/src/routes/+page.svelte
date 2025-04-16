<script>
import { onMount } from "svelte"
import { goto } from "$app/navigation"
import { MAX_MOBILE_SIZE } from "$lib/constants"
import { vw } from "$lib/utils"

const isMobile = vw() < MAX_MOBILE_SIZE

onMount(() => {
  if (isMobile) {
  } else {
    goto("/transfer", { replaceState: true })
  }
})
</script>
