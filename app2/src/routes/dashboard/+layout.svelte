<script lang="ts">
import Sections from "$lib/components/ui/Sections.svelte"
import AccountInfo from "$lib/dashboard/components/AccountInfo.svelte"
import DashboardErrorContainer from "$lib/dashboard/components/DashboardErrorContainer.svelte"
import LastSyncLabel from "$lib/dashboard/components/LastSyncLabel.svelte"
import NewUser from "$lib/dashboard/components/NewUser.svelte"
import { dashboard } from "$lib/dashboard/stores/user.svelte"
import { Option } from "effect"
import type { Snippet } from "svelte"

interface Props {
  children: Snippet
  data: any
}

let { children, data }: Props = $props()

// Authentication is now handled by the load function
// The page will only render if authenticated
const isAuthenticated = $derived(
  data?.session || data?.isAuthenticated || Option.isSome(dashboard.session),
)
</script>

<Sections>
  <LastSyncLabel />
  <DashboardErrorContainer />
  <NewUser />
  {@render children()}
</Sections>
<AccountInfo />
