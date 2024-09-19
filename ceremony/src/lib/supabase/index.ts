import { user } from "$lib/stores/user.svelte.ts"
import {
  getContribution,
  getContributor,
  getQueueCount,
  getSubmittedContribution,
  getUserQueuePosition,
  queryAllowance,
  queryContributions,
  queryUserContribution
} from "$lib/supabase/queries.ts"
import { supabase } from "$lib/supabase/client.ts"
import type { AllowanceState, ContributionState } from "$lib/stores/state.svelte.ts"

export const callJoinQueue = async (code: string | null): Promise<boolean> => {
  const userId = user.session?.user.id
  if (!userId) {
    throw new Error("User is not logged in")
  }

  try {
    const { error } = await supabase.rpc("join_queue", { code_id: code })

    if (error) {
      console.error("Error joining queue:", error)
      return false
    }

    return true
  } catch (err) {
    console.error("Unexpected error:", err)
    return false
  }
}

export const checkIfOpen = async (): Promise<boolean> => {
  const { data, error } = await supabase.rpc("open_to_public")
  console.log("isOpen:", data)
  return data
}

export const getUserQueueInfo = async () => {
  const userId = user.session?.user.id
  if (!userId) {
    throw new Error("User is not logged in")
  }

  const { data, error } = await getUserQueuePosition(userId)
  const { count, error: countError } = await getQueueCount()

  if (error) {
    console.error("Error getting user queue position:", error)
    return { error }
  }

  if (!data) {
    return {
      inQueue: false,
      message: "User not found in the queue"
    }
  }

  return {
    inQueue: true,
    count: count,
    ...data
  }
}

export const getContributionState = async (): Promise<ContributionState> => {
  const userId = user.session?.user.id
  if (!userId) {
    throw new Error("User ID is required")
  }

  try {
    const [contributor, submittedContribution, verifiedContribution] = await Promise.all([
      getContributor(userId),
      getSubmittedContribution(userId),
      getContribution(userId)
    ])

    const isContributor = !!contributor?.data
    const hasSubmitted = !!submittedContribution?.data
    const hasVerified = !!verifiedContribution?.data

    let status: ContributionState

    if (isContributor && !hasSubmitted && !hasVerified) {
      status = "contribute"
    } else if (isContributor && hasSubmitted && !hasVerified) {
      status = "verifying"
    } else if (hasVerified) {
      status = "contributed"
    } else {
      status = "notContributed"
    }

    return status
  } catch (error) {
    console.log("Error checking contribution status:", error)
    throw new Error("Failed to check contribution status")
  }
}

export const getAllowanceState = async (userId: string | undefined): Promise<AllowanceState> => {
  if (!userId) {
    console.log("Need to be logged in to get allowance state")
    return undefined
  }

  const { data, error } = await queryAllowance()
  if (error || !data) return undefined

  if (data.has_redeemed) return "hasRedeemed"
  if (data.in_queue) return "inQueue"
  if (data.in_waitlist) return "inWaitlist"

  return "join"
}

export const getContributions = async () => {
  const { data, error } = await queryContributions()
  if (error || !data) return undefined

  console.log(data)

  return data
}

export const getUserContribution = async (hash: string) => {
  const { data, error } = await queryUserContribution(hash)
  if (error || !data)
    return {
      id: "5515a415-a96e-4ec6-9cca-05d7cac203f0",
      user_name: "Lukas",
      avatar_url: "https://avatars.githubusercontent.com/u/36674091?v=4",
      seq: 12,
      payload_id: "d6533ce0-17e3-48d1-b738-f915d009d5c9",
      public_key:
        "2d2d2d2d2d424547494e20504750205055424c4943204b455920424c4f434b2d2d2d2d2d0a0a786a4d455a7572793268594a4b7759424241486152773842415164416b544d3875744d6b6562554d653173666a5a4c5877367054446d4d7457665259526d4c560a3849504f50464c4e466d78316132467a4c6d3535596d56795a30426e625746706243356a6232334367775151466767414b77495a415155435a7572327a5149620a4167494c4351495641414557466945452b486957756b356a46307a412b6c477158726b2f61467831412f554143676b5158726b2f61467831412f58597867442b0a4a4170655675726872596676615659754170545550354e68666238732b616a796353517967365953417863412f32313163634a555477632f5251525a62582b750a62344b6f4e496465786f346a6b5262306b59716c2b3441480a3d3171504d0a2d2d2d2d2d454e4420504750205055424c4943204b455920424c4f434b2d2d2d2d2d0a",
      signature:
        "2d2d2d2d2d424547494e20504750205349474e4544204d4553534147452d2d2d2d2d0a486173683a205348413235360a0a305f5f5f5f5f5f30202d2030303030303030302d303030302d303030302d303030302d303030303030303030303030202d2064363533336365302d313765332d343864312d623733382d663931356430303964356339202d20383562323331323233363138633665343830373963393365356335623439373366383030623733323530313337326564333231383139316464356434366138300a2d2d2d2d2d424547494e20504750205349474e41545552452d2d2d2d2d0a0a776e5545415259494142305749515434654a6136546d4d58544d4436556170657554396f58485544395155435a7572327a51414b435242657554396f584855440a39555737415143654943472f76686534483674556775476d416135683248444d4252356a776b6c59715647596e2f5375346744384474536f2f634f77426768360a485657726f393842526e324d317952704d684279745839513847767876676b3d0a3d4d4235480a2d2d2d2d2d454e4420504750205349474e41545552452d2d2d2d2d0a",
      public_key_hash: "174874cebccadc3e169c8826b00171268200050e428427d0757937d2cf407d36"
    }

  return data
}
