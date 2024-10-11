/*
Package forkchoice implements the service to support fork choice for the Ethereum beacon chain. This contains the
necessary components to track latest validators votes, and balances. Then a store object to be used
to calculate head. High level fork choice summary:
https://notes.ethereum.org/@vbuterin/rkhCgQteN?type=view#LMD-GHOST-fork-choice-rule
*/
package forkchoice
