<script lang="ts">
import Sections from "$lib/components/ui/Sections.svelte"
import DashboardErrorContainer from "$lib/dashboard/components/DashboardErrorContainer.svelte"
import type { Snippet } from "svelte"

interface Props {
  children: Snippet
}

let { children }: Props = $props()
</script>

<Sections>
  <DashboardErrorContainer />
  {@render children()}
</Sections>
