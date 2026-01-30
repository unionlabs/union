<script lang="ts">
import "../app.css"
import { page } from "$app/stores"
import ExplorerHeader from "$lib/components/explorer-header.svelte"
import ExplorerSidebar from "$lib/components/explorer-sidebar.svelte"
import * as Sidebar from "$lib/components/ui/sidebar/index.js"
import { ModeWatcher } from "mode-watcher"

const { children } = $props()

const getPageTitle = (pathname: string) => {
  if (pathname === "/") {
    return "Overview"
  }
  if (pathname.startsWith("/blocks")) {
    return "Blocks"
  }
  if (pathname.startsWith("/transactions")) {
    return "Transactions"
  }
  if (pathname.startsWith("/validators")) {
    return "Validators"
  }
  if (pathname.startsWith("/governance")) {
    return "Governance"
  }
  if (pathname.startsWith("/parameters")) {
    return "Parameters"
  }
  if (pathname.startsWith("/ibc")) {
    return "IBC"
  }
  if (pathname.startsWith("/account")) {
    return "Account"
  }
  if (pathname.startsWith("/search")) {
    return "Search"
  }
  return "Explorer"
}
</script>

<ModeWatcher defaultMode="dark" />

<Sidebar.Provider style="--header-height: 3rem;">
  <ExplorerSidebar />
  <Sidebar.Inset>
    <ExplorerHeader title={getPageTitle($page.url.pathname)} />
    <main class="flex-1 p-4 lg:p-6">
      {@render children()}
    </main>
  </Sidebar.Inset>
</Sidebar.Provider>
