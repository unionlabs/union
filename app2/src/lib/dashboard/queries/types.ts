export interface SnagDevicePayload {
  ipAddress: string;
  userId?: string;
  walletAddress?: string;
  deviceIdentifier?: string;
}

export interface SnagMetadataPayload {
  userId?: string;
  walletAddress?: string;
  stardustProfileIdentifier?: string | null;
  isBlocked?: boolean;
  stardustWalletIdentifier?: string | null;
  epicAccountIdentifier?: string | null;
  discordUser?: string | null;
  discordUserId?: string | null;
  twitterUser?: string | null;
  twitterUserFollowersCount?: number | null;
  twitterUserId?: string | null;
  emailAddress?: string | null;
  walletGroupIdentifier?: string | null;
  userGroupId?: string;
  telegramUserId?: string | null;
  telegramUsername?: string | null;
  steamUserId?: string | null;
  steamUsername?: string | null;
  googleUserId?: string | null;
  googleUser?: string | null;
  YTChannelId?: string | null;
  displayName?: string | null;
  externalLoyaltyScore?: string | null;
  logoUrl?: string | null;
  userGroupExternalIdentifier?: string | null;
  externalIdentifier?: string | null;
  websiteId?: string;
  organizationId?: string;
}
