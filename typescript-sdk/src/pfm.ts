export const createPfmMemo = ({
  port,
  channel,
  receiver
}: {
  port: string
  channel: string
  receiver: string
}) =>
  JSON.stringify({
    forward: {
      port,
      channel,
      receiver: receiver.startsWith("0x") ? receiver.slice(2) : receiver
    }
  })
