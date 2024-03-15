#!/usr/bin/env bash

set -eou pipefail

#
# print full Hasura GraphQL schema
#

source .env

ADMIN_SECRET=${ADMIN_SECRET}
GRAPHQL_URL=${GRAPHQL_URL:-'https://graphql.union.build/v1/graphql'}

gq ${GRAPHQL_URL} \
  --header "X-Hasura-Admin-Secret: ${ADMIN_SECRET}" \
  --introspect >./src/generated/schema.graphql
