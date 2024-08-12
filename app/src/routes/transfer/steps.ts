let stepperSteps: any = derived([fromChain, transferState], ([$fromChain, $transferState]) => {
  if ($transferState.kind === "PRE_TRANSFER") return [] // don't generate steps before transfer is ready
  if ($fromChain?.rpc_type === "evm") {
    // TODO: Refactor this by implementing Ord for transferState
    return [
      // Do not uncomment
      stateToStatus(
        $transferState,
        "SWITCHING_TO_CHAIN",
        `Switch to ${$fromChain.display_name}`,
        `Switched to ${$fromChain.display_name}`,
        ts => ({
          status: "ERROR",
          title: `Error switching to ${$fromChain.display_name}`,
          description: `There was an issue switching to ${$fromChain.display_name} to your wallet. ${ts.warning}`
        }),
        () => ({
          status: "WARNING",
          title: `Could not automatically switch chain.`,
          description: `Please make sure your wallet is connected to  ${$fromChain.display_name}`
        }),
        () => ({
          status: "IN_PROGRESS",
          title: `Switching to ${$fromChain.display_name}`,
          description: `Click 'Approve' in wallet.`
        })
      ),
      stateToStatus(
        $transferState,
        "APPROVING_ASSET",
        "Approve ERC20",
        "Approved ERC20",
        ts => ({
          status: "ERROR",
          title: `Error approving ERC20`,
          description: `${ts.error}`
        }),
        () => ({}),
        () => ({
          status: "IN_PROGRESS",
          title: "Approving ERC20",
          description: "Click 'Next' and 'Approve' in wallet."
        })
      ),
      stateToStatus(
        $transferState,
        "AWAITING_APPROVAL_RECEIPT",
        "Wait for approval receipt",
        "Received approval receipt",
        ts => ({
          status: "ERROR",
          title: `Error waiting for approval receipt`,
          description: `${ts.error}`
        }),
        () => ({}),
        () => ({
          status: "IN_PROGRESS",
          title: "Awaiting approval receipt",
          description: `Waiting on ${$fromChain.display_name}`
        })
      ),
      stateToStatus(
        $transferState,
        "SIMULATING_TRANSFER",
        "Simulate transfer",
        "Simulated transfer",
        ts => ({
          status: "ERROR",
          title: `Error simulating transfer on ${$fromChain.display_name}`,
          // @ts-expect-error
          description: `${ts.error}`
        }),
        () => ({
          status: "WARNING",
          title: `Failed to simulate transfer`,
          description: `You can still attempt to make this transfer in your wallet`
        }),
        () => ({
          status: "IN_PROGRESS",
          title: "Simulating transfer",
          description: `Waiting on ${$fromChain.display_name}`
        })
      ),
      stateToStatus(
        $transferState,
        "CONFIRMING_TRANSFER",
        "Confirm transfer",
        "Confirmed transfer",
        ts => ({
          status: "ERROR",
          title: "Error confirming transfer",
          description: `${ts.error}`
        }),
        () => ({}),
        () => ({
          status: "IN_PROGRESS",
          title: "Confirming your transfer",
          description: `Click 'Confirm' in your wallet`
        })
      ),
      stateToStatus(
        $transferState,
        "AWAITING_TRANSFER_RECEIPT",
        "Wait for transfer receipt",
        "Confirmed transfer",
        ts => ({
          status: "ERROR",
          title: "Error while waiting on transfer receipt",
          description: `tx hash: ${ts.transferHash}, error: ${ts.error}`
        }),
        () => ({}),
        () => ({
          status: "IN_PROGRESS",
          title: "Awaiting transfer receipt",
          description: `Waiting on ${$fromChain.display_name}`
        })
      ),
      stateToStatus(
        $transferState,
        "TRANSFERRING",
        "Transfer assets",
        "Transferred assets",
        () => ({}),
        () => ({}),
        () => ({
          status: "IN_PROGRESS",
          title: "Transferring assets",
          description: `Successfully initiated transfer`
        })
      )
    ]
  }
  if ($fromChain?.rpc_type === "cosmos") {
    return [
      stateToStatus(
        $transferState,
        "SWITCHING_TO_CHAIN",
        `Switch to ${$fromChain.display_name}`,
        `Switched to ${$fromChain.display_name}`,
        ts => ({
          status: "ERROR",
          title: `Error switching to ${$fromChain.display_name}`,
          description: `There was an issue switching to ${$fromChain.display_name} to your wallet. ${ts.warning}`
        }),
        () => ({
          status: "WARNING",
          title: `Could not automatically switch chain.`,
          description: `Please make sure your wallet is connected to  ${$fromChain.display_name}`
        }),
        () => ({
          status: "IN_PROGRESS",
          title: `Switching to ${$fromChain.display_name}`,
          description: `Click 'Approve' in wallet.`
        })
      ),
      stateToStatus(
        $transferState,
        "CONFIRMING_TRANSFER",
        "Confirm transfer",
        "Confirmed transfer",
        ts => ({
          status: "ERROR",
          title: "Error confirming transfer",
          description: `${ts.error}`
        }),
        () => ({}),
        () => ({
          status: "IN_PROGRESS",
          title: "Confirming your transfer",
          description: `Click 'Approve' in your wallet`
        })
      ),
      stateToStatus(
        $transferState,
        "TRANSFERRING",
        "Transfer assets",
        "Transferred assets",
        () => ({}),
        () => ({}),
        () => ({
          status: "IN_PROGRESS",
          title: "Transferring assets",
          description: `Successfully initiated transfer`
        })
      )
    ]
  }
  raise("trying to make stepper for unsupported chain")
})
