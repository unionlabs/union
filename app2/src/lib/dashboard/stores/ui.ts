import type {
  AchievementError,
  AuthenticationError,
  CategoryError,
  ChainError,
  DashboardUnknownException,
  EmailLinkError,
  LeaderboardError,
  MissionError,
  ProviderLinkError,
  RewardError,
  SupabaseClientError,
  SupabaseError,
  WalletError,
} from "$lib/dashboard/errors"
import { writable } from "svelte/store"

type DashboardError =
  | AuthenticationError
  | SupabaseClientError
  | ChainError
  | CategoryError
  | SupabaseError
  | DashboardUnknownException
  | AchievementError
  | LeaderboardError
  | MissionError
  | RewardError
  | WalletError
  | ProviderLinkError
  | EmailLinkError

interface UIState {
  error: DashboardError | null
}

const createUIStore = () => {
  const { subscribe, update } = writable<UIState>({
    error: null,
  })

  return {
    subscribe,
    showError: (error: DashboardError) => {
      update(state => ({ ...state, error }))
    },
    clearError: () => {
      update(state => ({ ...state, error: null }))
    },
  }
}

export const uiStore = createUIStore()
