# Using the smartcard wallet

## Requirements

  * A USB smartcard reader
  * A keycard that supports the status app
  * PCSCD version 4.3 running on your system **Only version 4.3 is currently supported**

## Preparing the smartcard

  **WARNING: FOLLOWING THESE INSTRUCTIONS WILL DESTROY THE MASTER KEY ON YOUR CARD. ONLY PROCEED IF NO FUNDS ARE ASSOCIATED WITH THESE ACCOUNTS**

  You can use status' [keycard-cli](https://github.com/status-im/keycard-cli) and you should get _at least_ version 2.1.1 of their [smartcard application](https://github.com/status-im/status-keycard/releases/download/2.2.1/keycard_v2.2.1.cap)

  You also need to make sure that the PCSC daemon is running on your system.

  Then, you can install the application to the card by typing:

  ```
  keycard install -a keycard_v2.2.1.cap && keycard init
  ```

  At the end of this process, you will be provided with a PIN, a PUK and a pairing password. Write them down, you'll need them shortly.

  Start `geth` with the `console` command. You will notice the following warning:

  ```
  WARN [04-09|16:58:38.898] Failed to open wallet                    url=keycard://044def09                          err="smartcard: pairing password needed"
  ```

  Write down the URL (`keycard://044def09` in this example). Then ask `geth` to open the wallet:

  ```
  > personal.openWallet("keycard://044def09", "pairing password")
  ```

  The pairing password has been generated during the card initialization process.
  
  The process needs to be repeated once more with the PIN:

  ```
  > personal.openWallet("keycard://044def09", "PIN number")
  ```
  
  If everything goes well, you should see your new account when typing `personal` on the console:

  ```
  > personal
  WARN [04-09|17:02:07.330] Smartcard wallet account derivation failed url=keycard://044def09 err="Unexpected response status Cla=0x80, Ins=0xd1, Sw=0x6985"
  {
    listAccounts: [],
    listWallets: [{
        status: "Empty, waiting for initialization",
        url: "keycard://044def09"
    }],
    ...
  }
  ```

  So the communication with the card is working, but there is no key associated with this wallet. Let's create it:

  ```
  > personal.initializeWallet("keycard://044def09")
  "tilt ... impact"
  ```

  You should get a list of words, this is your seed so write them down. Your wallet should now be initialized:

  ```
  > personal.listWallets
  [{
    accounts: [{
        address: "0x678b7cd55c61917defb23546a41803c5bfefbc7a",
        url: "keycard://044d/m/44'/60'/0'/0/0"
    }],
    status: "Online",
    url: "keycard://044def09"
  }]
  ```

  You're all set!

## Usage

  1. Start `geth` with the `console` command
  2. Check the card's URL by checking `personal.listWallets`:

```
  listWallets: [{
      status: "Online, can derive public keys",
      url: "keycard://a4d73015"
  }]
```

  3. Open the wallet, you will be prompted for your pairing password, then PIN:

```
personal.openWallet("keycard://a4d73015")
```

  4. Check that creation was successful by typing e.g. `personal`. Then use it like a regular wallet.

## Known issues

  * Starting geth with a valid card seems to make firefox crash.
  * PCSC version 4.4 should work, but is currently untested
