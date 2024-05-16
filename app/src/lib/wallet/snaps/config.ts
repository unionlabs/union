

interface SnapMethodParams {
  snapId: string
}

export async function checkSnapAvailability({ snapId }: SnapMethodParams) {
  if (!window?.ethereum) return
  let result = await window.ethereum.request({ method: "wallet_getSnaps" })
  if (!result || typeof result !== "object") return
  return Object.hasOwn(result, snapId)
}

export const installSnap = async ({
  snapId,
  snapVersion
}: SnapMethodParams & { snapVersion: string }) =>
  await window?.ethereum?.request({
    method: "wallet_requestSnaps",
    params: {
      [snapId]: { version: snapVersion }
    }
  })

// Initialize with default chain
export const initializeSnap = async ({ snapId }: SnapMethodParams) =>
  await window?.ethereum?.request({
    method: "wallet_invokeSnap",
    params: {
      snapId,
      request: { method: "initialize" }
    }
  })

export const checkSnapInitialized = async ({ snapId }: SnapMethodParams) =>
  await window?.ethereum?.request({
    method: "wallet_invokeSnap",
    params: {
      snapId,
      request: { method: "initialized" }
    }
  })
