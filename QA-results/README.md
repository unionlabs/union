# QA Sanity Checklist Process for Union Labs

This repository documents the QA sanity check procedures for the Union Labs. The QA process ensures core features such as token transfers, wallet connectivity, and explorer functionalities are verified across all supported blockchain channel pairs (e.g., Ethereum <> Babylon, Babylon <> Bob).

## QA Checklist Document

The main QA checklist is hosted as a Google Spreadsheet:

**[QA Checklist Link](https://docs.google.com/spreadsheets/d/12BEhTyA98ElUdvMEFqK5jNTrEZxIL9tRhgI538dGQWM/edit?gid=1011123960#gid=1011123960)**

### How to Use the Checklist:

1. **Duplicate the Sheet**  
   - Make a copy of the QA checklist.
   - Rename it using the following format:  
     `QA Sanity Check - [Date] - [Version] - [Tester Name]`

2. **Update Channel Pairs**  
   - Ensure **all active interchain pairs** are listed (e.g., Ethereum <> Babylon, Babylon <> Bob).  
   - **Add** any new pairs that are now live and **remove** outdated ones.

3. **Verify All QA Sections**  
   - Follow each checklist item and complete it step-by-step.
   - Log relevant transfer links, verification notes, and observations.
   - If an issue is discovered during testing, report it via the incident response system immediately.

4. **Finalization**  
   - Ensure the top of your duplicated document includes:
     - QA Sanity Check - [Date] - [Version] - [Tester Name]
   - Upload the finalized QA document to the `QA-results` folder of the main repository:
     [Union GitHub QA Folder](https://github.com/unionlabs/union/tree/QA-results)

---

## ✅ QA Sections Overview

The checklist is divided into the following verification areas:

### 1. Transfers Between Chains
- Validate transfers for **ERC20**, **CW20**, and **native tokens** for each channel pair.
- Confirm packet acknowledgments and recipient correctness.

### 2. Encoding & Decoding Verification
- Confirm data in transfer packets (sender, receiver, amount) is correctly encoded and decoded.
- Compare on-chain raw data with UI-represented values.

### 3. MultiSig Transaction Test
- Send a transfer using a multi-signature wallet.
- Validate it appears in both `/explorer/transfers` and `/explorer/find-packet`.
- Confirm full encoding and decoding.

### 4. Wallet Connectivity
- Test major wallet integrations:
  - **Cosmos**: Keplr, Leap  
  - **EVM**: MetaMask, Safe Wallet, WalletConnect, Brave Wallet, Coinbase Wallet

### 5. Application Page Checks
- Review all frontend pages for functionality:
  - Page loads
  - Button states (active/disabled)
  - Auto-filled and editable receiver fields
  - Transfer history
  - Search/filter performance

### 6. Post-Checks & Documentation
- Double-check transfer links are saved in the sheet.
- Ensure the document is stored in the central repository.
- Notify the team of completion and note any issues encountered.

