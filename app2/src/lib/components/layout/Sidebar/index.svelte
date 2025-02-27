<script>
import ChainList from "$lib/components/ChainList.svelte"
import Sections from "$lib/components/ui/Sections.svelte"
import { wallets } from "$lib/stores/wallets.svelte"
</script>

<Sections>
  <h1 class="font-extrabold text-3xl uppercase">Union</h1>
  <section>
    <h2 class="font-bold text-xl">Transfer</h2>
    <ul>
      <li>Transfer</li>
      <li>Swap</li>
      <li>Stake</li>
    </ul>
  </section>
  <section>
    <h2 class="font-bold text-xl">Explorer</h2>
    <ul>
      <li><a href="/" class="underline">Home</a></li>
      <li><a href="/explorer/transfers" class="underline">Transfers</a></li>
      <!--<li><a href="/explorer/packets" class="underline">Packets</a></li>!-->
      <!--<li><a href="/explorer/connections" class="underline">Connections</a></li>!-->
      <!--<li><a href="/explorer/channels" class="underline">Channels</a></li>!-->
    </ul>
  </section>

  <section>
  {wallets.evmAddress}
  </section>
</Sections>
