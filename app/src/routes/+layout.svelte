<script lang="ts">
  import '$lib/polyfill.ts'
  import '$styles/index.css'
  // import '@rainbow-me/rainbowkit/styles.css'
  // import {
  //   getDefaultConfig,
  //   RainbowKitProvider,
  //   darkTheme,
  //   lightTheme,
  // } from '@rainbow-me/rainbowkit'
  import { WagmiProvider } from 'wagmi'

  import { onMount } from 'svelte'
  import { page } from '$app/stores'
  import { ModeWatcher } from 'mode-watcher'
  import { browser } from '$app/environment'
  import { shortcut } from '@svelte-put/shortcut'
  import Footer from '$lib/components/footer.svelte'
  import { Toaster } from '$lib/components/ui/sonner'
  import { crtEffectEnabled } from '$lib/stores/user.ts'
  import { notifyManager } from '@tanstack/svelte-query'
  import DevTools from '$lib/components/dev-tools.svelte'
  import { createQueryClient } from '$lib/query-client.ts'
  import Header from '$lib/components/header/header.svelte'
  import LoadingBar from '$lib/components/loading-bar.svelte'
  import { updateTheme } from '$lib/utilities/update-theme.ts'
  import { SvelteQueryDevtools } from '@tanstack/svelte-query-devtools'
  import { checkWebGLSupport, deviceWidth } from '$lib/utilities/device.ts'
  import { disablePinchToZoom } from '$lib/utilities/disable-pinch-to-zoom.ts'
  import { config, chains } from '$lib/wallet/evm/config.ts'
  import { sepolia, arbitrumSepolia } from 'wagmi/chains'
  import { QueryClientProvider, QueryClient } from '@tanstack/react-query'
  import { ConnectKitProvider, getDefaultConfig } from 'connectkit'

  const { queryClient, localStoragePersister, PersistQueryClientProvider } = createQueryClient()
  if (browser) notifyManager.setScheduler(window.requestAnimationFrame)

  onMount(() => {
    checkWebGLSupport()
    disablePinchToZoom()
  })

  $: updateTheme({ path: $page.url.pathname, activeTheme: 'dark' })

  // const defaultConfig = getDefaultConfig({
  //   appName: 'Union App',
  //   projectId: '49fe74ca5ded7142adefc69a7788d14a',
  //   chains: [sepolia, arbitrumSepolia],
  // })

  const client = new QueryClient()
</script>

<svelte:head>
  <title>Union App Beta</title>
  <meta name="description" content="Union Web App" />
</svelte:head>

<svelte:window
  bind:innerWidth={$deviceWidth}
  use:shortcut={{
    trigger: [
      // easily hide tanstack devtools with ctrl + h
      {
        key: 'h',
        modifier: ['ctrl'],
        callback: () => {
          console.info('Hiding tanstack devtools')
          const tanstackDevtoolsElement = document.querySelector('div.tsqd-transitions-container')
          if (!tanstackDevtoolsElement) return
          tanstackDevtoolsElement.classList.toggle('hidden')
        },
      },
    ],
  }}
/>

<LoadingBar />

<PersistQueryClientProvider
  client={queryClient}
  persistOptions={{ persister: localStoragePersister }}
>
  <react:WagmiProvider {config}>
    <react:QueryClientProvider {client}>
      <!-- <react:RainbowKitProvider
      
        theme={lightTheme({
          fontStack: 'system',
          borderRadius: 'none',
          accentColor: '#A0ECFD',
          accentColorForeground: '#0D0D0D',
        })}
      > -->
      <react:ConnectKitProvider
        theme="default"
        options={{
          disclaimer: `foo bar`,
          embedGoogleFonts: true,
          truncateLongENSAddress: true,
        }}
        customTheme={// {
        //   '--ck-font-family': 'JetBrains Mono, monospace',
        //   '--ck-border-radius': 0,
        //   '--ck-connectbutton-border-radius': 0,
        //   '--ck-connectbutton-color': '#000000',
        //   '--ck-connectbutton-background': '#A0ECFD',
        //   '--ck-accent-color-foreground': '#0D0D0D',
        //   '--ck-connectbutton-hover-background': '#B4F0FD',
        // }
        {
          '--ck-font-family': 'JetBrains Mono, monospace',
          '--ck-font-weight': '600',
          '--ck-border-radius': '0px',
          '--ck-overlay-backdrop-filter': 'blur(8px)',
          '--ck-modal-heading-font-weight': '500',
          '--ck-qr-border-radius': '16px',
          '--ck-connectbutton-font-size': '15px',
          '--ck-connectbutton-color': '#000000',
          '--ck-connectbutton-background': '#a0ecfd',
          '--ck-connectbutton-background-secondary': '#FFFFFF',
          '--ck-connectbutton-border-radius': '0px',
          '--ck-connectbutton-box-shadow': '0 0 0 0 #ffffff',
          '--ck-connectbutton-hover-color': '#00000',
          '--ck-connectbutton-font-weight': '900',
          '--ck-connectbutton-hover-background': '#b4f1ff',
          '--ck-connectbutton-hover-box-shadow': '0 0 0 0 #ffffff',
          '--ck-connectbutton-active-color': '',
          '--ck-connectbutton-active-background': '#b4f1ff',
          '--ck-connectbutton-active-box-shadow': '0 0 0 0 #ffffff',
          '--ck-connectbutton-balance-color': '#373737',
          '--ck-connectbutton-balance-background': '#fff',
          '--ck-connectbutton-balance-box-shadow': 'inset 0 0 0 1px #F6F7F9',
          '--ck-connectbutton-balance-hover-background': '#F6F7F9',
          '--ck-connectbutton-balance-hover-box-shadow': 'inset 0 0 0 1px #F0F2F5',
          '--ck-connectbutton-balance-active-background': '#F0F2F5',
          '--ck-connectbutton-balance-active-box-shadow': 'inset 0 0 0 1px #EAECF1',
          '--ck-primary-button-font-weight': '600',
          '--ck-primary-button-border-radius': '0px',
          '--ck-primary-button-color': '#000000',
          '--ck-primary-button-background': '#caf5ff',
          '--ck-primary-button-box-shadow': '0 0 0 0 #ffffff',
          '--ck-primary-button-hover-color': '#a0ecfd',
          '--ck-primary-button-hover-background': '#ffffff',
          '--ck-primary-button-hover-box-shadow': '0 0 0 0 #ffffff',
          '--ck-primary-button-active-color': '#373737',
          '--ck-primary-button-active-background': '#a0ecfd',
          '--ck-primary-button-active-box-shadow': '0 0 0 0 #ffffff',
          '--ck-secondary-button-font-weight': '600',
          '--ck-secondary-button-border-radius': '0px',
          '--ck-secondary-button-color': '#000000',
          '--ck-secondary-button-background': '#a0ecfd',
          '--ck-secondary-button-box-shadow': '0 0 0 0 #ffffff',
          '--ck-secondary-button-hover-color': '#ffffff',
          '--ck-secondary-button-hover-background': '#a0ecfd',
          '--ck-secondary-button-hover-box-shadow': '0 0 0 0 #ffffff',
          '--ck-secondary-button-active-color': '#373737',
          '--ck-secondary-button-active-background': '#ffffff',
          '--ck-secondary-button-active-box-shadow': '0 0 0 0 #ffffff',
          '--ck-tertiary-button-font-weight': '500',
          '--ck-tertiary-button-border-radius': '16px',
          '--ck-tertiary-button-color': '#373737',
          '--ck-tertiary-button-background': '#F6F7F9',
          '--ck-tertiary-button-box-shadow': '0 0 0 0 #ffffff',
          '--ck-tertiary-button-hover-color': '#373737',
          '--ck-tertiary-button-hover-background': '#F6F7F9',
          '--ck-tertiary-button-hover-box-shadow': '0 0 0 0 #ffffff',
          '--ck-tertiary-button-active-color': '#373737',
          '--ck-tertiary-button-active-background': '#F6F7F9',
          '--ck-tertiary-button-active-box-shadow': '0 0 0 0 #ffffff',
          '--ck-modal-box-shadow': '0px 2px 4px 0px #00000005',
          '--ck-overlay-background': '#44444408',
          '--ck-body-color': '#000000',
          '--ck-body-color-muted': '#999999',
          '--ck-body-color-muted-hover': '#111111',
          '--ck-body-background': '#ffffff',
          '--ck-body-background-transparent': 'rgba(255,255,255,0)',
          '--ck-body-background-secondary': '#ffffff',
          '--ck-body-background-secondary-hover-background': '#e0e4eb',
          '--ck-body-background-secondary-hover-outline': '#4282FF',
          '--ck-body-background-tertiary': '#ffffff',
          '--ck-body-action-color': '#999999',
          '--ck-body-divider': '#f7f6f8',
          '--ck-body-color-danger': '#FF4E4E',
          '--ck-body-color-valid': '#32D74B',
          '--ck-siwe-border': '#F0F0F0',
          '--ck-body-disclaimer-background': '#f6f7f9',
          '--ck-body-disclaimer-color': '#AAAAAB',
          '--ck-body-disclaimer-link-color': '#838485',
          '--ck-body-disclaimer-link-hover-color': '#000000',
          '--ck-tooltip-background': '#ffffff',
          '--ck-tooltip-background-secondary': '#ffffff',
          '--ck-tooltip-color': '#999999',
          '--ck-tooltip-shadow': '0px 2px 10px 0 #00000014',
          '--ck-dropdown-button-color': '#999999',
          '--ck-dropdown-button-box-shadow':
            '0 0 0 1px rgba(0,0,0,0.01), 0px 0px 7px rgba(0, 0, 0, 0.05)',
          '--ck-dropdown-button-background': '#fff',
          '--ck-dropdown-button-hover-color': '#8B8B8B',
          '--ck-dropdown-button-hover-background': '#F5F7F9',
          '--ck-qr-dot-color': '#000000',
          '--ck-qr-background': '#ffffff',
          '--ck-qr-border-color': '#f7f6f8',
          '--ck-focus-color': '#1A88F8',
          '--ck-spinner-color': '#1A88F8',
          '--ck-copytoclipboard-stroke': '#CCCCCC',
          '--ck-recent-badge-color': '#777',
          '--ck-recent-badge-background': '#F6F7F9',
          '--ck-recent-badge-border-radius': '32px',
        }}
      >
        <ModeWatcher defaultMode="system" />
        <Toaster position="bottom-right" expand />

        <Header />
        <div class="flex flex-1 overflow-y-auto bg-background">
          <slot />
        </div>
        <Footer />
      </react:ConnectKitProvider>
      <!-- </react:RainbowKitProvider> -->
      <SvelteQueryDevtools
        position="bottom"
        client={queryClient}
        initialIsOpen={false}
        buttonPosition="bottom-right"
      />
      <DevTools />
      <!-- will be enabled once powered by index status !-->
      <!-- <OnlineStatus /> !-->
    </react:QueryClientProvider>
  </react:WagmiProvider>
</PersistQueryClientProvider>

<div
  class:crt={$crtEffectEnabled}
  class="absolute top-0 w-dvw h-dvh z-50 pointer-events-none"
></div>

<style>
  :global([data-close-button]) {
    background-color: hsl(var(--card) / var(--tw-bg-opacity));
  }
  :global(.grecaptcha-badge) {
    visibility: hidden;
    position: fixed;
    width: 0;
    height: 0;
  }
</style>
