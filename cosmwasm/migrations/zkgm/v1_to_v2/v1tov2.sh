#!/usr/bin/env nix-shell
#!nix-shell --pure -i runghc -p "haskellPackages.ghcWithPackages (pkgs: with pkgs; [ text bytestring base16-bytestring aeson ])"

{-# LANGUAGE OverloadedStrings #-}
{-# LANGUAGE DeriveGeneric #-}
{-# LANGUAGE LambdaCase #-}

import Data.Either
import Data.Aeson
import Data.Aeson.Types (Parser)
import qualified Data.Aeson as A
import Data.ByteString (ByteString)
import qualified Data.ByteString.Lazy as LBS
import qualified Data.ByteString.Lazy.Char8 as LBS8
import qualified Data.ByteString.Base16 as B16
import Data.Text (Text)
import qualified Data.Text as T
import qualified Data.Text.Encoding as TE
import GHC.Generics
import Control.Monad (mzero)
import Data.Maybe (mapMaybe)
import System.Environment (getArgs)

data ChainData = ChainData
  { wrappedUniversalChainId :: Text
  , direction               :: Direction
  , sourceChannelId         :: Int
  , destinationChannelId    :: Int
  , baseToken               :: ByteString
  , quoteToken              :: ByteString
  } deriving (Show, Eq, Generic)

instance FromJSON ChainData where
  parseJSON = withObject "ChainData" $ \o -> ChainData
    <$> o .: "wrapped_universal_chain_id"
    <*> o .: "direction"
    <*> o .: "source_channel_id"
    <*> o .: "destination_channel_id"
    <*> (o .: "base_token" >>= parseHexByteString)
    <*> (o .: "quote_token" >>= parseHexByteString)

instance ToJSON ChainData where
  toJSON (ChainData wuci dir sci dci bt qt) = object
    [ "wrapped_universal_chain_id" .= wuci
    , "direction"                  .= dir
    , "source_channel_id"          .= sci
    , "destination_channel_id"     .= dci
    , "base_token"                 .= ("0x" <> TE.decodeUtf8 (B16.encode bt))
    , "quote_token"                .= ("0x" <> TE.decodeUtf8 (B16.encode qt))
    ]

data V1ToV2Migration = V1ToV2Migration
  { v1Path       :: Text
  , v1ChannelId  :: Int
  , v1BaseToken  :: Text
  , v1QuoteToken :: ByteString
  } deriving (Show, Eq, Generic)

data V1ToV2WrappedMigration = V1ToV2WrappedMigration
  { v2Path       :: Text
  , v2ChannelId  :: Int
  , v2BaseToken  :: ByteString
  , v2QuoteToken :: Text
  } deriving (Show, Eq, Generic)

data Direction = In | Out
  deriving (Show, Eq, Generic)

instance FromJSON Direction where
  parseJSON = withText "Direction" $ \case
    "in"  -> pure In
    "out" -> pure Out
    x     -> fail $ "Unknown direction: " <> T.unpack x

instance ToJSON Direction where
  toJSON In  = String "in"
  toJSON Out = String "out"

instance ToJSON V1ToV2Migration where
  toJSON (V1ToV2Migration path channelId baseToken quoteToken) = object
    [ "path"        .= path
    , "channel_id"  .= channelId
    , "base_token"  .= baseToken
    , "quote_token" .= ("0x" <> TE.decodeUtf8 (B16.encode quoteToken))
    ]

instance FromJSON V1ToV2Migration where
  parseJSON = withObject "V1ToV2Migration" $ \o -> V1ToV2Migration
    <$> o .: "path"
    <*> o .: "channel_id"
    <*> o .: "base_token"
    <*> (o .: "quote_token" >>= parseHexByteString)

instance ToJSON V1ToV2WrappedMigration where
  toJSON (V1ToV2WrappedMigration path channelId baseToken quoteToken) = object
    [ "path"        .= path
    , "channel_id"  .= channelId
    , "base_token"  .= ("0x" <> TE.decodeUtf8 (B16.encode baseToken))
    , "quote_token" .= quoteToken
    ]

instance FromJSON V1ToV2WrappedMigration where
  parseJSON = withObject "V1ToV2WrappedMigration" $ \o -> V1ToV2WrappedMigration
    <$> o .: "path"
    <*> o .: "channel_id"
    <*> (o .: "base_token" >>= parseHexByteString)
    <*> o .: "quote_token"

parseHexByteString :: Text -> Parser ByteString
parseHexByteString hexText =
  either (const $ fail "Invalid hex string") pure $
    B16.decode (TE.encodeUtf8 cleanPrefix)
  where
    cleanPrefix = if T.isPrefixOf "0x" hexText
                 then T.drop 2 hexText
                 else hexText

byteStringToHexText :: ByteString -> Text
byteStringToHexText = TE.decodeUtf8

decodeChainDataList :: ByteString -> [ChainData]
decodeChainDataList =
  fromRight (fail "Invalid json") . eitherDecode . LBS.fromStrict

extractV1ToV2Migrations :: [ChainData] -> [V1ToV2Migration]
extractV1ToV2Migrations = mapMaybe extractV1ToV2Migration
  where
    extractV1ToV2Migration chainData =
      case direction chainData of
        Out -> Just $ V1ToV2Migration
          { v1Path       = "0"
          , v1ChannelId  = sourceChannelId chainData
          , v1BaseToken  = byteStringToHexText (baseToken chainData)
          , v1QuoteToken = quoteToken chainData
          }
        In -> Nothing

extractV1ToV2WrappedMigrations :: [ChainData] -> [V1ToV2WrappedMigration]
extractV1ToV2WrappedMigrations = mapMaybe extractV1ToV2WrappedMigration
  where
    extractV1ToV2WrappedMigration chainData =
      case direction chainData of
        In -> Just $ V1ToV2WrappedMigration
          { v2Path       = "0"
          , v2ChannelId  = destinationChannelId chainData
          , v2BaseToken  = baseToken chainData
          , v2QuoteToken = byteStringToHexText (quoteToken chainData)
          }
        Out -> Nothing

processChainDataToMigrations :: ByteString -> Text -> ([V1ToV2Migration], [V1ToV2WrappedMigration])
processChainDataToMigrations jsonBytes chainId =
  let chainDataList     = decodeChainDataList jsonBytes
      filteredChainData = filter (\cd -> wrappedUniversalChainId cd == chainId) chainDataList
  in ( extractV1ToV2Migrations filteredChainData
     , extractV1ToV2WrappedMigrations filteredChainData
     )

processFile :: String -> Text -> IO ()
processFile inputFile chainId = do
  inputJson <- LBS.readFile inputFile
  let inputBytes = LBS.toStrict inputJson
      (v1ToV2Migrations, v1ToV2WrappedMigrations) = processChainDataToMigrations inputBytes chainId
  LBS.writeFile "v1tov2_migrations.json" (encode v1ToV2Migrations)
  putStrLn $ "Written " <> show (length v1ToV2Migrations) <> " v1tov2 migrations to v1tov2_migrations.json"
  LBS.writeFile "v1tov2wrapped_migrations.json" (encode v1ToV2WrappedMigrations)
  putStrLn $ "Written " <> show (length v1ToV2WrappedMigrations) <> " v1tov2wrapped migrations to v1tov2wrapped_migrations.json"
  putStrLn $ "Filtered for chain ID: " <> T.unpack chainId

main :: IO ()
main = do
  args <- getArgs
  case args of
    [inputFile, chainIdFilter] -> processFile inputFile (T.pack chainIdFilter)
    _ -> do
      putStrLn "Usage: ./v1tov2.sh <input.json> <chain_id_filter>"
      putStrLn "  input.json        - JSON file containing chain data array"
      putStrLn "  chain_id_filter   - Only process entries matching this wrapped_universal_chain_id"
