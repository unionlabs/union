import type { introspection } from '$generated/graphql-env';
import { initGraphQLTada } from 'gql.tada';

export const graphql = initGraphQLTada<{
  introspection: introspection;
  scalars: {
    jsonb: any;
    timestamptz: string;
  }
}>()
