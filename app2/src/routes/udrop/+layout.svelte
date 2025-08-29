<script lang="ts">
import { goto } from "$app/navigation"
import { page } from "$app/state"
import Sections from "$lib/components/ui/Sections.svelte"
import CodeModal from "$lib/dashboard/components/CodeModal.svelte"
import DashboardErrorContainer from "$lib/dashboard/components/DashboardErrorContainer.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { Option } from "effect"
import { onMount } from "svelte"
import type { Snippet } from "svelte"

interface Props {
  children: Snippet
}

let { children }: Props = $props()
let showCodeModal = $state(false)
let referralCode = $state("")

// Watch for code parameter in URL
$effect(() => {
  const code = page.url.searchParams.get("code")
  if (code && code.trim() && Option.isSome(dashboard.session)) {
    referralCode = code.trim()
    showCodeModal = true
  } else {
    showCodeModal = false
    referralCode = ""
    // Clean URL if no session or no code
    if (code && Option.isNone(dashboard.session)) {
      const url = new URL(page.url)
      url.searchParams.delete("code")
      window.history.replaceState({}, "", url.toString())
    }
  }
})

function handleCloseCodeModal() {
  showCodeModal = false
  // Remove code parameter from URL without navigation
  const url = new URL(page.url)
  url.searchParams.delete("code")
  window.history.replaceState({}, "", url.toString())
}

onMount(() => {
  if (Option.isNone(dashboard.session)) {
    goto("/udrop/check")
  }
})
</script>

<Sections>
  <DashboardErrorContainer />
  {@render children()}
</Sections>

<!-- Code Modal -->
<CodeModal
  isOpen={showCodeModal}
  onClose={handleCloseCodeModal}
  code={referralCode}
/>
