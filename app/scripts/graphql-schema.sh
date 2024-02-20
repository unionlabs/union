#!/usr/bin/env bash

set -eou pipefail

#
# print full Hasura GraphQL schema
#

source .env

GRAPHQL_URL=${GRAPHQL_URL:-'https://graphql.union.build/v1/graphql'}

curl --silent --location \
  --request POST \
  --url $GRAPHQL_URL \
  --header 'Content-Type: application/json' \
  --data '{"query":"query IntrospectionQuery { __schema { queryType { name } mutationType { name } subscriptionType { name } types { ...FullType } directives { name description locations args { ...InputValue } } } } fragment FullType on __Type { kind name description fields(includeDeprecated: true) { name description args { ...InputValue } type { ...TypeRef } isDeprecated deprecationReason } inputFields { ...InputValue } interfaces { ...TypeRef } enumValues(includeDeprecated: true) { name description isDeprecated deprecationReason } possibleTypes { ...TypeRef } } fragment InputValue on __InputValue { name description type { ...TypeRef } defaultValue } fragment TypeRef on __Type { kind name ofType { kind name ofType { kind name ofType { kind name ofType { kind name ofType { kind name ofType { kind name ofType { kind name } } } } } } } }","variable":{}}'
