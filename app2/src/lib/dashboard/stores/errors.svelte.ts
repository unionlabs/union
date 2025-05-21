import type {
  AccountError,
  AchievementError,
  AuthenticationError,
  CategoryError,
  ChainError,
  DashboardUnknownException,
  EmailLinkError,
  ExperienceError,
  LeaderboardError,
  MissionError,
  ProviderLinkError,
  RewardError,
  SupabaseClientError,
  SupabaseError,
  WalletError,
} from "$lib/dashboard/errors"

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
  | AccountError
  | ExperienceError

class ErrorStore {
  current: DashboardError | null = $state(null)

  showError(error: DashboardError) {
    this.current = error
  }

  clearError() {
    this.current = null
  }
}

export const errorStore = new ErrorStore()
