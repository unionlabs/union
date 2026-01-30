<script lang="ts">
import { dev } from "$app/environment"
import { goto } from "$app/navigation"
import { page } from "$app/stores"
import { type ChainConfig, CHAINS, type SocialLink } from "$lib/chains/config"
import SearchCommand from "$lib/components/search-command.svelte"
import * as DropdownMenu from "$lib/components/ui/dropdown-menu/index.js"
import * as Sidebar from "$lib/components/ui/sidebar/index.js"
import { cache } from "$lib/snippet-cache/promise.svelte"
import { addressFormat } from "$lib/stores/address-format.svelte"
import { chainStore } from "$lib/stores/chain.svelte"
import ActivityIcon from "@lucide/svelte/icons/activity"
import ArrowLeftRightIcon from "@lucide/svelte/icons/arrow-left-right"
import BoxIcon from "@lucide/svelte/icons/box"
import CheckIcon from "@lucide/svelte/icons/check"
import ChevronsUpDownIcon from "@lucide/svelte/icons/chevrons-up-down"
import GithubIcon from "@lucide/svelte/icons/github"
import GlobeIcon from "@lucide/svelte/icons/globe"
import LayoutDashboardIcon from "@lucide/svelte/icons/layout-dashboard"
import LinkIcon from "@lucide/svelte/icons/link"
import MessageCircleIcon from "@lucide/svelte/icons/message-circle"
import SendIcon from "@lucide/svelte/icons/send"
import SettingsIcon from "@lucide/svelte/icons/settings"
import ShieldIcon from "@lucide/svelte/icons/shield"
import TwitterIcon from "@lucide/svelte/icons/twitter"
import VoteIcon from "@lucide/svelte/icons/vote"
import WalletIcon from "@lucide/svelte/icons/wallet"
import type { Component, ComponentProps } from "svelte"

const socialIcons: Record<SocialLink["type"], Component> = {
  twitter: TwitterIcon,
  discord: MessageCircleIcon,
  github: GithubIcon,
  telegram: SendIcon,
  website: GlobeIcon,
}

// Base nav items (will be prefixed with chainId)
const navItems = [
  { title: "Dashboard", path: "", icon: LayoutDashboardIcon },
  { title: "Blocks", path: "/blocks", icon: BoxIcon },
  { title: "Transactions", path: "/transactions", icon: ArrowLeftRightIcon },
  { title: "Validators", path: "/validators", icon: ShieldIcon },
  { title: "Accounts", path: "/account", icon: WalletIcon },
  { title: "Governance", path: "/governance", icon: VoteIcon },
  { title: "Parameters", path: "/parameters", icon: SettingsIcon },
  { title: "IBC", path: "/ibc", icon: LinkIcon },
]

// Current chain from store
const currentChain = $derived(chainStore.config)

// Generate full URL with chain prefix (using universal_chain_id)
function getUrl(path: string): string {
  return `/${currentChain.universal_chain_id}${path}`
}

// Check if nav item is active
function isActive(path: string, pathname: string): boolean {
  const fullPath = getUrl(path)
  if (path === "") {
    return pathname === fullPath
  }
  return pathname.startsWith(fullPath)
}

// Get all chains as array
const chains = Object.values(CHAINS)

// Navigate to a different chain, preserving the current page path
function selectChain(chain: ChainConfig) {
  if (chain.universal_chain_id === chainStore.id) {
    return
  }

  // Get current path segment (e.g., /blocks, /transactions/xyz)
  const currentPath = $page.url.pathname
  const pathParts = currentPath.split("/")
  // Remove the chain_id part (second segment after leading /)
  // Universal chain IDs contain a dot, so we need to handle that
  const pagePath = pathParts.length > 2 ? "/" + pathParts.slice(2).join("/") : ""

  // Clear cache first, then navigate
  // The layout's $effect will update chainStore when URL changes
  cache.clear()
  goto(`/${chain.universal_chain_id}${pagePath}`, { invalidateAll: true })
}

let { ...restProps }: ComponentProps<typeof Sidebar.Root> = $props()
</script>

<Sidebar.Root
  collapsible="icon"
  {...restProps}
  class="border-r border-border bg-background"
>
  <Sidebar.Header class="p-2">
    <DropdownMenu.Root>
      <DropdownMenu.Trigger class="w-full">
        <div class="flex items-center gap-2 px-2 py-1.5 hover:bg-muted transition-colors group">
          <div class="flex size-6 items-center justify-center bg-foreground text-background text-xs font-bold font-mono shrink-0">
            {currentChain.pretty_name.charAt(0)}
          </div>
          <div class="flex-1 text-left group-data-[collapsible=icon]:hidden">
            <div class="text-sm font-medium truncate">{currentChain.pretty_name}</div>
            <div class="text-[10px] font-mono text-muted-foreground truncate">
              {currentChain.chain_id}
            </div>
          </div>
          <ChevronsUpDownIcon
            class="h-4 w-4 text-muted-foreground shrink-0 group-data-[collapsible=icon]:hidden"
          />
        </div>
      </DropdownMenu.Trigger>
      <DropdownMenu.Content
        class="w-[--radix-dropdown-menu-trigger-width] min-w-56"
        align="start"
      >
        <DropdownMenu.Label
          class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground"
        >
          Switch Chain
        </DropdownMenu.Label>
        <DropdownMenu.Separator />
        {#each chains as chain}
          <DropdownMenu.Item
            class="flex items-center gap-2 cursor-pointer"
            onclick={() => selectChain(chain)}
          >
            <div class="flex size-5 items-center justify-center bg-foreground text-background text-[10px] font-bold font-mono">
              {chain.pretty_name.charAt(0)}
            </div>
            <div class="flex-1">
              <div class="text-sm">{chain.pretty_name}</div>
              <div class="text-[10px] font-mono text-muted-foreground">{chain.chain_id}</div>
            </div>
            {#if chain.universal_chain_id === currentChain.universal_chain_id}
              <CheckIcon class="h-4 w-4" />
            {/if}
          </DropdownMenu.Item>
        {/each}
        <DropdownMenu.Separator />
        <DropdownMenu.Item
          class="text-xs text-muted-foreground cursor-default"
          disabled
        >
          More chains coming soon
        </DropdownMenu.Item>
      </DropdownMenu.Content>
    </DropdownMenu.Root>
  </Sidebar.Header>

  <Sidebar.Content class="py-2">
    <!-- Search -->
    <div class="px-2 pb-2">
      <SearchCommand />
    </div>

    <Sidebar.Group>
      <Sidebar.GroupLabel
        class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground px-3 py-2"
      >
        Explorer
      </Sidebar.GroupLabel>
      <Sidebar.GroupContent>
        <Sidebar.Menu>
          {#each navItems as item}
            {@const active = isActive(item.path, $page.url.pathname)}
            {@const url = getUrl(item.path)}
            <Sidebar.MenuItem>
              <Sidebar.MenuButton
                tooltipContent={item.title}
                isActive={active}
                class="mx-1"
              >
                {#snippet child({ props })}
                  <a
                    href={url}
                    {...props}
                    class="flex items-center gap-3 px-2 py-1.5 text-sm transition-colors {active ? 'bg-foreground text-background' : 'text-muted-foreground hover:text-foreground hover:bg-muted'}"
                  >
                    <item.icon class="h-4 w-4" />
                    <span class="font-medium">{item.title}</span>
                  </a>
                {/snippet}
              </Sidebar.MenuButton>
            </Sidebar.MenuItem>
          {/each}
        </Sidebar.Menu>
      </Sidebar.GroupContent>
    </Sidebar.Group>

    <!-- Dev Tools (only in dev mode) -->
    {#if dev}
      <Sidebar.Group>
        <Sidebar.GroupLabel
          class="text-[10px] font-mono uppercase tracking-wider text-muted-foreground px-3 py-2"
        >
          Dev Tools
        </Sidebar.GroupLabel>
        <Sidebar.GroupContent>
          <Sidebar.Menu>
            <Sidebar.MenuItem>
              <Sidebar.MenuButton
                tooltipContent="Indexer Health"
                isActive={$page.url.pathname === "/health"}
                class="mx-1"
              >
                {#snippet child({ props })}
                  <a
                    href="/health"
                    {...props}
                    class="flex items-center gap-3 px-2 py-1.5 text-sm transition-colors {$page.url.pathname === '/health' ? 'bg-foreground text-background' : 'text-muted-foreground hover:text-foreground hover:bg-muted'}"
                  >
                    <ActivityIcon class="h-4 w-4" />
                    <span class="font-medium">Indexer Health</span>
                  </a>
                {/snippet}
              </Sidebar.MenuButton>
            </Sidebar.MenuItem>
          </Sidebar.Menu>
        </Sidebar.GroupContent>
      </Sidebar.Group>
    {/if}
  </Sidebar.Content>

  <Sidebar.Footer class="mt-auto p-3 space-y-3">
    <!-- Address Format Toggle -->
    <div class="flex items-center justify-center group-data-[collapsible=icon]:hidden">
      <button
        onclick={() => addressFormat.toggle()}
        class="flex items-center gap-2 px-2 py-1 text-[10px] font-mono uppercase tracking-wider text-muted-foreground hover:text-foreground transition-colors"
        title="Toggle address format"
      >
        <span class={addressFormat.value === "hex" ? "text-foreground" : ""}>HEX</span>
        <span class="text-muted-foreground/50">/</span>
        <span class={addressFormat.value === "base64" ? "text-foreground" : ""}>B64</span>
      </button>
    </div>

    {#if currentChain.socials?.length}
      <div class="flex items-center justify-center gap-1 group-data-[collapsible=icon]:flex-col">
        {#each currentChain.socials as social}
          {@const Icon = socialIcons[social.type]}
          <a
            href={social.url}
            target="_blank"
            rel="noopener noreferrer"
            class="p-2 text-muted-foreground hover:text-foreground hover:bg-muted transition-colors"
            title={social.type.charAt(0).toUpperCase() + social.type.slice(1)}
          >
            <Icon class="h-4 w-4" />
          </a>
        {/each}
      </div>
    {/if}
  </Sidebar.Footer>
</Sidebar.Root>
