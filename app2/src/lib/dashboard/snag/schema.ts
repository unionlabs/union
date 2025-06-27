import * as S from "effect/Schema"

export class CreateUserDeviceParams extends S.Class<CreateUserDeviceParams>("CreateUserDeviceParams")({
  ipAddress: S.String,
}) {}

export class UserDevice extends S.Class<UserDevice>("UserDevice")({
  id: S.String,
  websiteId: S.String,
  organizationId: S.String,
  ipAddress: S.String,
  deviceIdentifier: S.String,
  source: S.String,
  createdAt: S.String,
}) {}

// Schema that exactly matches SDK's MetadataCreateParams interface
export const CreateUserMetadataParams = S.Struct({
  discordUser: S.optionalWith(S.Union(S.String, S.Null), { exact: true }),
  discordUserId: S.optionalWith(S.Union(S.String, S.Null), { exact: true }),
  displayName: S.optionalWith(S.Union(S.String, S.Null), { exact: true }),
  emailAddress: S.optionalWith(S.Union(S.String, S.Null), { exact: true }),
  epicAccountIdentifier: S.optionalWith(S.Union(S.String, S.Null), { exact: true }),
  externalIdentifier: S.optionalWith(S.Union(S.String, S.Null), { exact: true }),
  externalLoyaltyScore: S.optionalWith(S.Union(S.String, S.Null), { exact: true }),
  googleUser: S.optionalWith(S.Union(S.String, S.Null), { exact: true }),
  googleUserId: S.optionalWith(S.Union(S.String, S.Null), { exact: true }),
  isBlocked: S.optionalWith(S.Boolean, { exact: true }),
  logoUrl: S.optionalWith(S.Union(S.String, S.Null), { exact: true }),
  organizationId: S.optionalWith(S.String, { exact: true }),
  stardustProfileIdentifier: S.optionalWith(S.Union(S.String, S.Null), { exact: true }),
  stardustWalletIdentifier: S.optionalWith(S.Union(S.String, S.Null), { exact: true }),
  steamUserId: S.optionalWith(S.Union(S.String, S.Null), { exact: true }),
  steamUsername: S.optionalWith(S.Union(S.String, S.Null), { exact: true }),
  telegramUserId: S.optionalWith(S.Union(S.String, S.Null), { exact: true }),
  telegramUsername: S.optionalWith(S.Union(S.String, S.Null), { exact: true }),
  twitterUser: S.optionalWith(S.Union(S.String, S.Null), { exact: true }),
  twitterUserFollowersCount: S.optionalWith(S.Union(S.Number, S.Null), { exact: true }),
  twitterUserId: S.optionalWith(S.Union(S.String, S.Null), { exact: true }),
  userGroupExternalIdentifier: S.optionalWith(S.Union(S.String, S.Null), { exact: true }),
  userGroupId: S.optionalWith(S.String, { exact: true }),
  userId: S.optionalWith(S.String, { exact: true }),
  walletAddress: S.optionalWith(S.String, { exact: true }),
  walletGroupIdentifier: S.optionalWith(S.Union(S.String, S.Null), { exact: true }),
  websiteId: S.optionalWith(S.String, { exact: true }),
  YTChannelId: S.optionalWith(S.Union(S.String, S.Null), { exact: true }),
})

export type CreateUserMetadataParams = typeof CreateUserMetadataParams.Type

export class UserGroup extends S.Class<UserGroup>("UserGroup")({
  id: S.String,
  externalIdentifier: S.String,
}) {}

export class SnagUser extends S.Class<SnagUser>("SnagUser")({
  id: S.String,
  walletAddress: S.String,
}) {}

export class UserMetadata extends S.Class<UserMetadata>("UserMetadata")({
  id: S.String, // Unique identifier for the user metadata
  websiteId: S.String, // Unique identifier for the website
  organizationId: S.String, // Unique identifier for the organization
  walletGroupIdentifier: S.NullishOr(S.String), // Identifier for the user wallet group set via api
  userGroupId: S.NullishOr(S.String), // Identifier for the user group set via api
  userGroup: UserGroup,
  user: SnagUser,
  createdAt: S.String, // Timestamp when the wallet was created
}) {}


// =============================================================================
// Validation Helpers
// =============================================================================

/**
 * Validates and decodes a user device response from the API
 */
export const validateUserDevice = S.decodeUnknown(UserDevice)

/**
 * Validates and decodes user metadata response from the API
 */
export const validateUserMetadata = S.decodeUnknown(UserMetadata)

/**
 * Validates create user device parameters
 */
export const validateCreateUserDeviceParams = S.decodeUnknown(CreateUserDeviceParams)

/**
 * Validates create user metadata parameters  
 */
export const validateCreateUserMetadataParams = S.decodeUnknown(CreateUserMetadataParams) 