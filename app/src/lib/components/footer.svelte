<script lang="ts">
import { navigating } from "$app/stores"
import { cn } from "$lib/utilities/shadcn.ts"
import HomeIcon from "virtual:icons/lucide/home"
import MenuIcon from "virtual:icons/lucide/menu"
import WalletMinimalIcon from "virtual:icons/lucide/wallet"
import { Button } from "$lib/components/ui/button/index.ts"
import ArrowDownUpIcon from "virtual:icons/lucide/arrow-up-down"
import { Navigation } from "$lib/components/navigation/index.ts"

const onWalletClick = () => document.querySelector("button[data-dialog-trigger]")?.click()

let navigationDrawerOpen = false
$: if ($navigating) navigationDrawerOpen = false

const navigationIconStyle = "size-9 min-w-6 dark:hover:text-white text-zinc-accent"
</script>

<footer
  class={cn(
    'overflow-hidden fixed left-0 bottom-0 right-0 w-screen h-16 py-2 ',
    'grid lg:hidden grid-cols-4 gap-y-2 place-content-center divide-x-[1px] divide-[#fafafa25]',
    'border-t-[1px] border-solid border-[#fafafa25] border-opacity-90 backdrop-blur-md',
    // styling children
    '*:my-auto *:self-center *:h-16 *:w-full *:rounded-none *:border-solid *:border-t-0',
  )}
>
  <Button
    href="/"
    size="lg"
    name="home"
    variant="link"
    aria-label="Home page link"
    class="hover:bg-transparent hover:bg-muted"
  >
    <HomeIcon class={navigationIconStyle} />
  </Button>
  <Button
    size="icon"
    name="send"
    href="/send"
    variant="link"
    aria-label="send page link"
    class="hover:bg-transparent hover:bg-muted"
  >
    <ArrowDownUpIcon class={navigationIconStyle} />
  </Button>
  <Button
    size="icon"
    name="wallet"
    type="button"
    variant="ghost"
    on:click={() => onWalletClick()}
    class="hover:bg-transparent hover:bg-muted"
  >
    <WalletMinimalIcon class={navigationIconStyle} />
  </Button>
  <Button
    size="icon"
    name="menu"
    type="button"
    variant="ghost"
    on:click={() => (navigationDrawerOpen = !navigationDrawerOpen)}
    class="hover:bg-transparent hover:bg-muted"
  >
    <MenuIcon class={navigationIconStyle} />
    <Navigation {navigationDrawerOpen} />
  </Button>
</footer>
