cw20IncreaseAllowance: async ({
                                contractAddress,
                                amount,
                                spender,
                                account = parameters.account,
                                gasPrice = parameters.gasPrice
                              }) => {
  const rpcUrl = parameters.transport({}).value?.url;
  if (!rpcUrl) return err(new Error("No cosmos RPC URL found"));
  if (!account) return err(new Error("No cosmos signer found"));
  if (!gasPrice) return err(new Error("No gas price found"));
  return await cosmwasmTransfer({
    account,
    rpcUrl,
    gasPrice,
    instructions: [
      {
        contractAddress,
        msg: { increase_allowance: { spender, amount: amount.toString() } }
      }
    ]
  });
},