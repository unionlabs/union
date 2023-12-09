/*
Package genutil contains a variety of genesis utility functionality
for usage within a blockchain application. Namely:
  - Genesis transactions related (gentx)
  - Commands for collection and creation of gentxs
  - `InitChain` processing of gentxs
  - Genesis file validation
  - Genesis file migration
  - CometBFT related initialization (Translation of an app genesis to a CometBFT genesis)
*/
package genutil
