<script lang="ts">
import Sections from "$lib/components/ui/Sections.svelte"
import AccountInfo from "$lib/dashboard/components/AccountInfo.svelte"
import DashboardErrorContainer from "$lib/dashboard/components/DashboardErrorContainer.svelte"
import LastSyncLabel from "$lib/dashboard/components/LastSyncLabel.svelte"
import NewUser from "$lib/dashboard/components/NewUser.svelte"
import type { Snippet } from "svelte"

interface Props {
  children: Snippet
}

let { children }: Props = $props()
</script>

<Sections>
  <LastSyncLabel />
  <DashboardErrorContainer />
  <NewUser />
  {@render children()}
</Sections>
<AccountInfo />
