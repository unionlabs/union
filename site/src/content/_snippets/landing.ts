interface Snippet {
  lang: string
  code: string
  title: string
}

export const codeSnippets = [
  {
    lang: "solidity",
    title: "swap.sol",
    code: `
\`\`\`solidity showLineNumbers wrap /IBCAppBase/#accent /ibcHandler/#accent
contract DexPortal is IBCAppBase {
  function swap(
    address base,
    uint256 amount,
    uint256 minReturn,
    uint64 timeout
  ) public {
    Proxy quote = swapProxies[base];
    IERC20(base).transferFrom(
      _msgSender(),
      address(this),
      amount
    );
    ibcHandler.send(
      quote.port,
      quote.channel,
      timeout,
      abi.encode(
        Swap({
          origin: _msgSender(),
          amount: amount,
          minReturn: minReturn
        })
      )
    );
  }
}
\`\`\`
`
  },
  {
    lang: "rust",
    title: "swap.rs",
    code: `
\`\`\`rust showLineNumbers /IbcMsg/#accent
fn swap(
    deps: Deps,
    env: Env,
    info: MessageInfo,
    base: String,
    amount: Uint256,
    min_return: Uint256,
    timeout: u64,
) -> Result<Response, Error> {
    let proxy = SWAP_PROXIES.load(&deps.storage, base)?;

    Ok(Response::add_message(wasm_execute(
        base,
        &cw20::msg::TransferFrom {
            owner: info.sender,
            recipient: env.contract.address,
            amount,
        },
        vec![],
    ))
    .add_message(IbcMsg::SendPacket {
        channel_id: proxy.channel,
        data: Swap {
            origin: info.sender,
            amount,
            min_return,
        },
        timeout: IbcTimeout::with_block(timeout),
    }))
}
`
  }
] satisfies Array<Snippet>
