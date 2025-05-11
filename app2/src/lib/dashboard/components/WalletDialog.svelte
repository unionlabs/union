<script lang="ts">
  import { Siwe } from 'ox';
  import { signMessage } from '@wagmi/core';

  import { getCosmosChainInfo } from "$lib/services/cosmos/chain-info";
  import { dashboard } from '../stores/user.svelte';
  import { Effect, pipe } from 'effect';
  import { extractErrorDetails } from '@unionlabs/sdk/utils';
  import { isValidBech32Address } from '$lib/utils/format';

  const allegianceMessage = "I'm signing this message to prove account ownership and to pledge allegiance to zkgm.";

  let selectedChains = $state<Array<string>>([]);
  let isConnecting = $state(false);

  async function connectEvmWallet() {
    isConnecting = true;
    
    try {
      const connectResult = await wagmiConnect({
        walletId: 'injected',
      });

      if (!connectResult?.chainId) {
        console.error('Failed to connect wallet');
        return;
      }

      const [address] = connectResult.accounts;
      if (!address) {
        console.error('No wallet address found');
        return;
      }

      // Check if wallet already exists
      const existingWallets = Option.getOrNull(dashboard.wallets);
      if (existingWallets?.some(wallet => wallet.address.toLowerCase() === address.toLowerCase())) {
        console.warn('Wallet already connected and verified');
        await wagmiDisconnect();
        return;
      }

      const siweMessage = Siwe.createMessage({
        address,
        version: '1',
        chainId: 11155111,
        nonce: Siwe.generateNonce(),
        domain: 'dashboard.union.build',
        uri: 'https://dashboard.union.build/wallet',
      });

      const signature = await signMessage(wagmiConfig, {
        account: address,
        message: siweMessage,
      });

      await Effect.runPromise(
        pipe(
          dashboard.wallets?.addWallet(address, `evm:${connectResult.chainId}`),
          Effect.catchAll((error) => {
            console.error('Failed to add wallet:', extractErrorDetails(error));
            return Effect.void;
          })
        )
      );

    } catch (error) {
      console.error('Wallet connection error:', error);
    } finally {
      await wagmiDisconnect();
      isConnecting = false;
    }
  }

  async function connectCosmosWallet(walletType: 'keplr' | 'leap') {
    try {
      const wallet = window[walletType];
      if (!wallet) {
        console.error(`${walletType} not found`);
        return;
      }

      const chainId = 'union-testnet-10';
      const chainInfo = await Effect.runPromise(
        getCosmosChainInfo({ chain_id: chainId } as Chain, walletType)
      );

      await wallet.experimentalSuggestChain(chainInfo);
      await wallet.enable([chainId]);

      const offlineSigner = wallet.getOfflineSigner(chainId);
      const accounts = await offlineSigner.getAccounts();
      const address = accounts[0].address;

      if (!isValidBech32Address(address)) {
        console.error('Invalid address');
        return;
      }

      const message = `Sign this message to verify your wallet ownership. Nonce: ${Date.now()}`;
      const signature = await offlineSigner.signAmino(address, {
        chain_id: chainId,
        account_number: '0',
        sequence: '0',
        fee: {
          amount: [],
          gas: '0',
        },
        msgs: [
          {
            type: 'sign/MsgSignData',
            value: {
              signer: address,
              data: message,
            },
          },
        ],
        memo: '',
      });

      await Effect.runPromise(
        pipe(
          dashboard.wallets?.addWallet({
            address,
            chain_id: chainId,
            wallet_type: walletType,
            signature: signature.signature,
            message,
          }),
          Effect.catchAll((error) => {
            console.error('Failed to add wallet:', error);
            return Effect.void;
          })
        )
      );
    } catch (error) {
      console.error('Failed to connect wallet:', error);
    }
  }
</script>

<dialog id="connect-dialog" class="backdrop:bg-black/50 backdrop:backdrop-blur-sm">
  <div class="bg-zinc-900 border border-zinc-800 rounded-lg p-6 max-w-md w-full">
    <h2 class="text-xl font-medium text-zinc-200 mb-4">Connect Wallet</h2>
    
    <div class="flex flex-col gap-4">
      <!-- EVM Wallet -->
      <button
        class="flex items-center justify-between p-4 bg-zinc-800/50 hover:bg-zinc-800 rounded-lg transition-colors"
        onclick={connectEvmWallet}
        disabled={isConnecting}
      >
        <div class="flex items-center gap-3">
          <img src="/wallets/metamask.svg" alt="MetaMask" class="w-8 h-8" />
          <div class="text-left">
            <div class="text-sm font-medium text-zinc-200">MetaMask</div>
            <div class="text-xs text-zinc-500">Connect your EVM wallet</div>
          </div>
        </div>
        <div class="text-zinc-500">→</div>
      </button>

      <!-- Keplr Wallet -->
      <button
        class="flex items-center justify-between p-4 bg-zinc-800/50 hover:bg-zinc-800 rounded-lg transition-colors"
        onclick={() => connectCosmosWallet('keplr')}
        disabled={isConnecting}
      >
        <div class="flex items-center gap-3">
          <img src="/wallets/keplr.svg" alt="Keplr" class="w-8 h-8" />
          <div class="text-left">
            <div class="text-sm font-medium text-zinc-200">Keplr</div>
            <div class="text-xs text-zinc-500">Connect your Cosmos wallet</div>
          </div>
        </div>
        <div class="text-zinc-500">→</div>
      </button>

      <!-- Leap Wallet -->
      <button
        class="flex items-center justify-between p-4 bg-zinc-800/50 hover:bg-zinc-800 rounded-lg transition-colors"
        onclick={() => connectCosmosWallet('leap')}
        disabled={isConnecting}
      >
        <div class="flex items-center gap-3">
          <img src="/wallets/leap.svg" alt="Leap" class="w-8 h-8" />
          <div class="text-left">
            <div class="text-sm font-medium text-zinc-200">Leap</div>
            <div class="text-xs text-zinc-500">Connect your Cosmos wallet</div>
          </div>
        </div>
        <div class="text-zinc-500">→</div>
      </button>
    </div>
  </div>
</dialog> 