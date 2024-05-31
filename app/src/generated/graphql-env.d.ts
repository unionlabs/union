/* eslint-disable */
/* prettier-ignore */

/** An IntrospectionQuery representation of your schema.
 *
 * @remarks
 * This is an introspection of your schema saved as a file by GraphQLSP.
 * It will automatically be used by `gql.tada` to infer the types of your GraphQL documents.
 * If you need to reuse this data or update your `scalars`, update `tadaOutputLocation` to
 * instead save to a .ts instead of a .d.ts file.
 */
export type introspection = {
  "__schema": {
    "queryType": {
      "name": "query_root"
    },
    "mutationType": null,
    "subscriptionType": {
      "name": "subscription_root"
    },
    "types": [
      {
        "kind": "SCALAR",
        "name": "Boolean"
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "Boolean_comparison_exp",
        "inputFields": [
          {
            "name": "_eq",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "_gt",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "_gte",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "_in",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "SCALAR",
                  "name": "Boolean"
                }
              }
            }
          },
          {
            "name": "_is_null",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "_lt",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "_lte",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "_neq",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "_nin",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "SCALAR",
                  "name": "Boolean"
                }
              }
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "SCALAR",
        "name": "Float"
      },
      {
        "kind": "SCALAR",
        "name": "Int"
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "Int_comparison_exp",
        "inputFields": [
          {
            "name": "_eq",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "_gt",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "_gte",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "_in",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              }
            }
          },
          {
            "name": "_is_null",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "_lt",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "_lte",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "_neq",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "_nin",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              }
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "SCALAR",
        "name": "String"
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "String_comparison_exp",
        "inputFields": [
          {
            "name": "_eq",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "_gt",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "_gte",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "_ilike",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "_in",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            }
          },
          {
            "name": "_iregex",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "_is_null",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "_like",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "_lt",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "_lte",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "_neq",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "_nilike",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "_nin",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            }
          },
          {
            "name": "_niregex",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "_nlike",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "_nregex",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "_nsimilar",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "_regex",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "_similar",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "SCALAR",
        "name": "bigint"
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "bigint_comparison_exp",
        "inputFields": [
          {
            "name": "_eq",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "_gt",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "_gte",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "_in",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "SCALAR",
                  "name": "bigint"
                }
              }
            }
          },
          {
            "name": "_is_null",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "_lt",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "_lte",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "_neq",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "_nin",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "SCALAR",
                  "name": "bigint"
                }
              }
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "cursor_ordering",
        "enumValues": [
          {
            "name": "ASC",
            "isDeprecated": false
          },
          {
            "name": "DESC",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "SCALAR",
        "name": "jsonb"
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "jsonb_cast_exp",
        "inputFields": [
          {
            "name": "String",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "jsonb_comparison_exp",
        "inputFields": [
          {
            "name": "_cast",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_cast_exp"
            }
          },
          {
            "name": "_contained_in",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "_contains",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "_eq",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "_gt",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "_gte",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "_has_key",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "_has_keys_all",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            }
          },
          {
            "name": "_has_keys_any",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            }
          },
          {
            "name": "_in",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "SCALAR",
                  "name": "jsonb"
                }
              }
            }
          },
          {
            "name": "_is_null",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "_lt",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "_lte",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "_neq",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "_nin",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "SCALAR",
                  "name": "jsonb"
                }
              }
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "SCALAR",
        "name": "numeric"
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "numeric_comparison_exp",
        "inputFields": [
          {
            "name": "_eq",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            }
          },
          {
            "name": "_gt",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            }
          },
          {
            "name": "_gte",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            }
          },
          {
            "name": "_in",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "SCALAR",
                  "name": "numeric"
                }
              }
            }
          },
          {
            "name": "_is_null",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "_lt",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            }
          },
          {
            "name": "_lte",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            }
          },
          {
            "name": "_neq",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            }
          },
          {
            "name": "_nin",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "SCALAR",
                  "name": "numeric"
                }
              }
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "order_by",
        "enumValues": [
          {
            "name": "asc",
            "isDeprecated": false
          },
          {
            "name": "asc_nulls_first",
            "isDeprecated": false
          },
          {
            "name": "asc_nulls_last",
            "isDeprecated": false
          },
          {
            "name": "desc",
            "isDeprecated": false
          },
          {
            "name": "desc_nulls_first",
            "isDeprecated": false
          },
          {
            "name": "desc_nulls_last",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "OBJECT",
        "name": "query_root",
        "fields": [
          {
            "name": "queue",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "queue"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "queue_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "queue_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "queue_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "queue_by_pk",
            "type": {
              "kind": "OBJECT",
              "name": "queue"
            },
            "args": [
              {
                "name": "created_at",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "timestamptz"
                  }
                }
              },
              {
                "name": "id",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "bigint"
                  }
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_blocks",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_blocks"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_blocks_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_blocks_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_blocks_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_blocks_by_pk",
            "type": {
              "kind": "OBJECT",
              "name": "v0_blocks"
            },
            "args": [
              {
                "name": "chain_id",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "Int"
                  }
                }
              },
              {
                "name": "hash",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "String"
                  }
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_chains",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_chains"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_chains_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_chains_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_chains_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_chains_aggregate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_chains_aggregate"
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_chains_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_chains_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_chains_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_chains_by_pk",
            "type": {
              "kind": "OBJECT",
              "name": "v0_chains"
            },
            "args": [
              {
                "name": "id",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "Int"
                  }
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_channel_map",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_channel_map"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_channel_map_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_channel_map_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_channel_map_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_channel_map_aggregate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_channel_map_aggregate"
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_channel_map_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_channel_map_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_channel_map_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_clients",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_clients"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_clients_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_clients_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_clients_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_clients_aggregate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_clients_aggregate"
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_clients_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_clients_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_clients_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_clients_by_pk",
            "type": {
              "kind": "OBJECT",
              "name": "v0_clients"
            },
            "args": [
              {
                "name": "chain_id",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "Int"
                  }
                }
              },
              {
                "name": "client_id",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "String"
                  }
                }
              },
              {
                "name": "counterparty_chain_id",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "String"
                  }
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_connection_map",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_connection_map"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_connection_map_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_connection_map_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_connection_map_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_connection_map_aggregate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_connection_map_aggregate"
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_connection_map_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_connection_map_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_connection_map_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_burn",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_cosmos_burn"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_cosmos_burn_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_burn_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_burn_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_burn_aggregate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_cosmos_burn_aggregate"
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_cosmos_burn_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_burn_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_burn_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_channel_open_ack",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_cosmos_channel_open_ack"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_cosmos_channel_open_ack_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_channel_open_ack_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_channel_open_ack_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_channel_open_ack_aggregate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_cosmos_channel_open_ack_aggregate"
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_cosmos_channel_open_ack_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_channel_open_ack_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_channel_open_ack_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_channel_open_init",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_cosmos_channel_open_init"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_cosmos_channel_open_init_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_channel_open_init_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_channel_open_init_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_channel_open_init_aggregate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_cosmos_channel_open_init_aggregate"
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_cosmos_channel_open_init_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_channel_open_init_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_channel_open_init_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_transfer",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_cosmos_transfer"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_cosmos_transfer_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_transfer_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_transfer_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_transfer_aggregate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_cosmos_transfer_aggregate"
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_cosmos_transfer_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_transfer_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_transfer_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_wasm_message",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_cosmos_wasm_message"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_cosmos_wasm_message_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_wasm_message_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_wasm_message_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_withdraw_rewards",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_cosmos_withdraw_rewards"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_cosmos_withdraw_rewards_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_withdraw_rewards_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_withdraw_rewards_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_evm_client_created",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_evm_client_created"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_evm_client_created_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_evm_client_created_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_evm_client_created_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_evm_client_created_aggregate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_evm_client_created_aggregate"
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_evm_client_created_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_evm_client_created_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_evm_client_created_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_evm_recv_packet",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_evm_recv_packet"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_evm_recv_packet_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_evm_recv_packet_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_evm_recv_packet_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_index_status",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_index_status"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_index_status_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_index_status_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_index_status_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_logs",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_logs"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_logs_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_logs_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_logs_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_logs_aggregate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_logs_aggregate"
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_logs_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_logs_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_logs_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_logs_by_pk",
            "type": {
              "kind": "OBJECT",
              "name": "v0_logs"
            },
            "args": [
              {
                "name": "block_hash",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "String"
                  }
                }
              },
              {
                "name": "chain_id",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "Int"
                  }
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_packets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_packets"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_packets_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_packets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_packets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_recv_packet",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_recv_packet"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_recv_packet_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_recv_packet_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_recv_packet_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_recv_packet_aggregate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_recv_packet_aggregate"
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_recv_packet_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_recv_packet_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_recv_packet_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_transfers",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_transfers"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_transfers_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_transfers_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_transfers_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "queue",
        "fields": [
          {
            "name": "created_at",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "timestamptz"
              }
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "id",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "bigint"
              }
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "item",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "jsonb"
              }
            },
            "args": [
              {
                "name": "path",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "message",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "parent",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "status",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "status"
              }
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "queue_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "queue_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "queue_bool_exp"
            }
          },
          {
            "name": "_or",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "queue_bool_exp"
                }
              }
            }
          },
          {
            "name": "created_at",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "item",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "message",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "parent",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "status",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "status_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "queue_order_by",
        "inputFields": [
          {
            "name": "created_at",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "item",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "message",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "parent",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "status",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "queue_select_column",
        "enumValues": [
          {
            "name": "created_at",
            "isDeprecated": false
          },
          {
            "name": "id",
            "isDeprecated": false
          },
          {
            "name": "item",
            "isDeprecated": false
          },
          {
            "name": "message",
            "isDeprecated": false
          },
          {
            "name": "parent",
            "isDeprecated": false
          },
          {
            "name": "status",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "queue_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "queue_stream_cursor_value_input"
              }
            }
          },
          {
            "name": "ordering",
            "type": {
              "kind": "ENUM",
              "name": "cursor_ordering"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "queue_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "created_at",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "id",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "item",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "message",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "parent",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "status",
            "type": {
              "kind": "SCALAR",
              "name": "status"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "SCALAR",
        "name": "status"
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "status_comparison_exp",
        "inputFields": [
          {
            "name": "_eq",
            "type": {
              "kind": "SCALAR",
              "name": "status"
            }
          },
          {
            "name": "_gt",
            "type": {
              "kind": "SCALAR",
              "name": "status"
            }
          },
          {
            "name": "_gte",
            "type": {
              "kind": "SCALAR",
              "name": "status"
            }
          },
          {
            "name": "_in",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "SCALAR",
                  "name": "status"
                }
              }
            }
          },
          {
            "name": "_is_null",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "_lt",
            "type": {
              "kind": "SCALAR",
              "name": "status"
            }
          },
          {
            "name": "_lte",
            "type": {
              "kind": "SCALAR",
              "name": "status"
            }
          },
          {
            "name": "_neq",
            "type": {
              "kind": "SCALAR",
              "name": "status"
            }
          },
          {
            "name": "_nin",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "SCALAR",
                  "name": "status"
                }
              }
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "subscription_root",
        "fields": [
          {
            "name": "queue",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "queue"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "queue_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "queue_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "queue_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "queue_by_pk",
            "type": {
              "kind": "OBJECT",
              "name": "queue"
            },
            "args": [
              {
                "name": "created_at",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "timestamptz"
                  }
                }
              },
              {
                "name": "id",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "bigint"
                  }
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "queue_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "queue"
                  }
                }
              }
            },
            "args": [
              {
                "name": "batch_size",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "Int"
                  }
                }
              },
              {
                "name": "cursor",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "LIST",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "queue_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "queue_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_blocks",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_blocks"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_blocks_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_blocks_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_blocks_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_blocks_by_pk",
            "type": {
              "kind": "OBJECT",
              "name": "v0_blocks"
            },
            "args": [
              {
                "name": "chain_id",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "Int"
                  }
                }
              },
              {
                "name": "hash",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "String"
                  }
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_blocks_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_blocks"
                  }
                }
              }
            },
            "args": [
              {
                "name": "batch_size",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "Int"
                  }
                }
              },
              {
                "name": "cursor",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "LIST",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_blocks_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_blocks_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_chains",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_chains"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_chains_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_chains_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_chains_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_chains_aggregate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_chains_aggregate"
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_chains_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_chains_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_chains_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_chains_by_pk",
            "type": {
              "kind": "OBJECT",
              "name": "v0_chains"
            },
            "args": [
              {
                "name": "id",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "Int"
                  }
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_channel_map",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_channel_map"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_channel_map_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_channel_map_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_channel_map_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_channel_map_aggregate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_channel_map_aggregate"
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_channel_map_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_channel_map_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_channel_map_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_clients",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_clients"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_clients_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_clients_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_clients_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_clients_aggregate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_clients_aggregate"
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_clients_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_clients_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_clients_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_clients_by_pk",
            "type": {
              "kind": "OBJECT",
              "name": "v0_clients"
            },
            "args": [
              {
                "name": "chain_id",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "Int"
                  }
                }
              },
              {
                "name": "client_id",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "String"
                  }
                }
              },
              {
                "name": "counterparty_chain_id",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "String"
                  }
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_connection_map",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_connection_map"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_connection_map_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_connection_map_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_connection_map_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_connection_map_aggregate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_connection_map_aggregate"
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_connection_map_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_connection_map_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_connection_map_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_burn",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_cosmos_burn"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_cosmos_burn_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_burn_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_burn_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_burn_aggregate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_cosmos_burn_aggregate"
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_cosmos_burn_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_burn_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_burn_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_burn_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_cosmos_burn"
                  }
                }
              }
            },
            "args": [
              {
                "name": "batch_size",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "Int"
                  }
                }
              },
              {
                "name": "cursor",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "LIST",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_burn_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_burn_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_channel_open_ack",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_cosmos_channel_open_ack"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_cosmos_channel_open_ack_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_channel_open_ack_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_channel_open_ack_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_channel_open_ack_aggregate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_cosmos_channel_open_ack_aggregate"
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_cosmos_channel_open_ack_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_channel_open_ack_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_channel_open_ack_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_channel_open_ack_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_cosmos_channel_open_ack"
                  }
                }
              }
            },
            "args": [
              {
                "name": "batch_size",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "Int"
                  }
                }
              },
              {
                "name": "cursor",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "LIST",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_channel_open_ack_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_channel_open_ack_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_channel_open_init",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_cosmos_channel_open_init"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_cosmos_channel_open_init_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_channel_open_init_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_channel_open_init_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_channel_open_init_aggregate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_cosmos_channel_open_init_aggregate"
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_cosmos_channel_open_init_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_channel_open_init_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_channel_open_init_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_channel_open_init_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_cosmos_channel_open_init"
                  }
                }
              }
            },
            "args": [
              {
                "name": "batch_size",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "Int"
                  }
                }
              },
              {
                "name": "cursor",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "LIST",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_channel_open_init_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_channel_open_init_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_transfer",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_cosmos_transfer"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_cosmos_transfer_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_transfer_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_transfer_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_transfer_aggregate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_cosmos_transfer_aggregate"
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_cosmos_transfer_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_transfer_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_transfer_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_transfer_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_cosmos_transfer"
                  }
                }
              }
            },
            "args": [
              {
                "name": "batch_size",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "Int"
                  }
                }
              },
              {
                "name": "cursor",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "LIST",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_transfer_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_transfer_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_wasm_message",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_cosmos_wasm_message"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_cosmos_wasm_message_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_wasm_message_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_wasm_message_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_wasm_message_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_cosmos_wasm_message"
                  }
                }
              }
            },
            "args": [
              {
                "name": "batch_size",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "Int"
                  }
                }
              },
              {
                "name": "cursor",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "LIST",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_wasm_message_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_wasm_message_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_withdraw_rewards",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_cosmos_withdraw_rewards"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_cosmos_withdraw_rewards_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_withdraw_rewards_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_withdraw_rewards_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_cosmos_withdraw_rewards_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_cosmos_withdraw_rewards"
                  }
                }
              }
            },
            "args": [
              {
                "name": "batch_size",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "Int"
                  }
                }
              },
              {
                "name": "cursor",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "LIST",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_cosmos_withdraw_rewards_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_withdraw_rewards_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_evm_client_created",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_evm_client_created"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_evm_client_created_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_evm_client_created_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_evm_client_created_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_evm_client_created_aggregate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_evm_client_created_aggregate"
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_evm_client_created_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_evm_client_created_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_evm_client_created_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_evm_recv_packet",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_evm_recv_packet"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_evm_recv_packet_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_evm_recv_packet_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_evm_recv_packet_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_index_status",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_index_status"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_index_status_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_index_status_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_index_status_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_index_status_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_index_status"
                  }
                }
              }
            },
            "args": [
              {
                "name": "batch_size",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "Int"
                  }
                }
              },
              {
                "name": "cursor",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "LIST",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_index_status_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_index_status_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_logs",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_logs"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_logs_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_logs_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_logs_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_logs_aggregate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_logs_aggregate"
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_logs_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_logs_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_logs_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_logs_by_pk",
            "type": {
              "kind": "OBJECT",
              "name": "v0_logs"
            },
            "args": [
              {
                "name": "block_hash",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "String"
                  }
                }
              },
              {
                "name": "chain_id",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "Int"
                  }
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_packets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_packets"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_packets_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_packets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_packets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_packets_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_packets"
                  }
                }
              }
            },
            "args": [
              {
                "name": "batch_size",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "Int"
                  }
                }
              },
              {
                "name": "cursor",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "LIST",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_packets_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_packets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_recv_packet",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_recv_packet"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_recv_packet_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_recv_packet_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_recv_packet_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_recv_packet_aggregate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_recv_packet_aggregate"
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_recv_packet_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_recv_packet_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_recv_packet_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_recv_packet_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_recv_packet"
                  }
                }
              }
            },
            "args": [
              {
                "name": "batch_size",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "Int"
                  }
                }
              },
              {
                "name": "cursor",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "LIST",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_recv_packet_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_recv_packet_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_transfers",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_transfers"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_transfers_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_transfers_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_transfers_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_transfers_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_transfers"
                  }
                }
              }
            },
            "args": [
              {
                "name": "batch_size",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "Int"
                  }
                }
              },
              {
                "name": "cursor",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "LIST",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_transfers_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_transfers_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "SCALAR",
        "name": "timestamptz"
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "timestamptz_comparison_exp",
        "inputFields": [
          {
            "name": "_eq",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "_gt",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "_gte",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "_in",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "SCALAR",
                  "name": "timestamptz"
                }
              }
            }
          },
          {
            "name": "_is_null",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "_lt",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "_lte",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "_neq",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "_nin",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "SCALAR",
                  "name": "timestamptz"
                }
              }
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_blocks",
        "fields": [
          {
            "name": "chain",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_chains"
              }
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Int"
              }
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "data",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "jsonb"
              }
            },
            "args": [
              {
                "name": "path",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "hash",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "String"
              }
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Int"
              }
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "time",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "timestamptz"
              }
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_blocks_aggregate_order_by",
        "inputFields": [
          {
            "name": "avg",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_blocks_avg_order_by"
            }
          },
          {
            "name": "count",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "max",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_blocks_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_blocks_min_order_by"
            }
          },
          {
            "name": "stddev",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_blocks_stddev_order_by"
            }
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_blocks_stddev_pop_order_by"
            }
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_blocks_stddev_samp_order_by"
            }
          },
          {
            "name": "sum",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_blocks_sum_order_by"
            }
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_blocks_var_pop_order_by"
            }
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_blocks_var_samp_order_by"
            }
          },
          {
            "name": "variance",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_blocks_variance_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_blocks_avg_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_blocks_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_blocks_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_blocks_bool_exp"
            }
          },
          {
            "name": "_or",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_blocks_bool_exp"
                }
              }
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_chains_bool_exp"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_blocks_max_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_blocks_min_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_blocks_order_by",
        "inputFields": [
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_chains_order_by"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v0_blocks_select_column",
        "enumValues": [
          {
            "name": "chain_id",
            "isDeprecated": false
          },
          {
            "name": "data",
            "isDeprecated": false
          },
          {
            "name": "hash",
            "isDeprecated": false
          },
          {
            "name": "height",
            "isDeprecated": false
          },
          {
            "name": "time",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_blocks_stddev_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_blocks_stddev_pop_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_blocks_stddev_samp_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_blocks_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v0_blocks_stream_cursor_value_input"
              }
            }
          },
          {
            "name": "ordering",
            "type": {
              "kind": "ENUM",
              "name": "cursor_ordering"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_blocks_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_blocks_sum_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_blocks_var_pop_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_blocks_var_samp_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_blocks_variance_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_chains",
        "fields": [
          {
            "name": "blocks",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_blocks"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_blocks_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_blocks_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_blocks_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "String"
              }
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "display_name",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "id",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Int"
              }
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "logs",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_logs"
                  }
                }
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_logs_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_logs_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_logs_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "logs_aggregate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_logs_aggregate"
              }
            },
            "args": [
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_logs_select_column"
                    }
                  }
                }
              },
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offset",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "INPUT_OBJECT",
                      "name": "v0_logs_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_logs_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "testnet",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_chains_aggregate",
        "fields": [
          {
            "name": "aggregate",
            "type": {
              "kind": "OBJECT",
              "name": "v0_chains_aggregate_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "nodes",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_chains"
                  }
                }
              }
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_chains_aggregate_fields",
        "fields": [
          {
            "name": "avg",
            "type": {
              "kind": "OBJECT",
              "name": "v0_chains_avg_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "count",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Int"
              }
            },
            "args": [
              {
                "name": "columns",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_chains_select_column"
                    }
                  }
                }
              },
              {
                "name": "distinct",
                "type": {
                  "kind": "SCALAR",
                  "name": "Boolean"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "max",
            "type": {
              "kind": "OBJECT",
              "name": "v0_chains_max_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "min",
            "type": {
              "kind": "OBJECT",
              "name": "v0_chains_min_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev",
            "type": {
              "kind": "OBJECT",
              "name": "v0_chains_stddev_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "OBJECT",
              "name": "v0_chains_stddev_pop_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "OBJECT",
              "name": "v0_chains_stddev_samp_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sum",
            "type": {
              "kind": "OBJECT",
              "name": "v0_chains_sum_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "OBJECT",
              "name": "v0_chains_var_pop_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "OBJECT",
              "name": "v0_chains_var_samp_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "variance",
            "type": {
              "kind": "OBJECT",
              "name": "v0_chains_variance_fields"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_chains_avg_fields",
        "fields": [
          {
            "name": "id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_chains_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_chains_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_chains_bool_exp"
            }
          },
          {
            "name": "_or",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_chains_bool_exp"
                }
              }
            }
          },
          {
            "name": "blocks",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_blocks_bool_exp"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "display_name",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "logs",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_logs_bool_exp"
            }
          },
          {
            "name": "logs_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_logs_aggregate_bool_exp"
            }
          },
          {
            "name": "testnet",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Boolean_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_chains_max_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "display_name",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_chains_min_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "display_name",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_chains_order_by",
        "inputFields": [
          {
            "name": "blocks_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_blocks_aggregate_order_by"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "display_name",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "logs_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_logs_aggregate_order_by"
            }
          },
          {
            "name": "testnet",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v0_chains_select_column",
        "enumValues": [
          {
            "name": "chain_id",
            "isDeprecated": false
          },
          {
            "name": "display_name",
            "isDeprecated": false
          },
          {
            "name": "id",
            "isDeprecated": false
          },
          {
            "name": "testnet",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "OBJECT",
        "name": "v0_chains_stddev_fields",
        "fields": [
          {
            "name": "id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_chains_stddev_pop_fields",
        "fields": [
          {
            "name": "id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_chains_stddev_samp_fields",
        "fields": [
          {
            "name": "id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_chains_sum_fields",
        "fields": [
          {
            "name": "id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_chains_var_pop_fields",
        "fields": [
          {
            "name": "id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_chains_var_samp_fields",
        "fields": [
          {
            "name": "id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_chains_variance_fields",
        "fields": [
          {
            "name": "id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_channel_map",
        "fields": [
          {
            "name": "connection",
            "type": {
              "kind": "OBJECT",
              "name": "v0_connection_map"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination",
            "type": {
              "kind": "OBJECT",
              "name": "v0_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source",
            "type": {
              "kind": "OBJECT",
              "name": "v0_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "status",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_channel_map_aggregate",
        "fields": [
          {
            "name": "aggregate",
            "type": {
              "kind": "OBJECT",
              "name": "v0_channel_map_aggregate_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "nodes",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_channel_map"
                  }
                }
              }
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_channel_map_aggregate_fields",
        "fields": [
          {
            "name": "avg",
            "type": {
              "kind": "OBJECT",
              "name": "v0_channel_map_avg_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "count",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Int"
              }
            },
            "args": [
              {
                "name": "columns",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_channel_map_select_column"
                    }
                  }
                }
              },
              {
                "name": "distinct",
                "type": {
                  "kind": "SCALAR",
                  "name": "Boolean"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "max",
            "type": {
              "kind": "OBJECT",
              "name": "v0_channel_map_max_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "min",
            "type": {
              "kind": "OBJECT",
              "name": "v0_channel_map_min_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev",
            "type": {
              "kind": "OBJECT",
              "name": "v0_channel_map_stddev_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "OBJECT",
              "name": "v0_channel_map_stddev_pop_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "OBJECT",
              "name": "v0_channel_map_stddev_samp_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sum",
            "type": {
              "kind": "OBJECT",
              "name": "v0_channel_map_sum_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "OBJECT",
              "name": "v0_channel_map_var_pop_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "OBJECT",
              "name": "v0_channel_map_var_samp_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "variance",
            "type": {
              "kind": "OBJECT",
              "name": "v0_channel_map_variance_fields"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_channel_map_avg_fields",
        "fields": [
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_channel_map_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_channel_map_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_channel_map_bool_exp"
            }
          },
          {
            "name": "_or",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_channel_map_bool_exp"
                }
              }
            }
          },
          {
            "name": "connection",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_connection_map_bool_exp"
            }
          },
          {
            "name": "destination",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_chains_bool_exp"
            }
          },
          {
            "name": "from_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "from_channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "from_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "from_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "from_port_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_chains_bool_exp"
            }
          },
          {
            "name": "status",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "to_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "to_channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "to_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "to_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "to_port_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_channel_map_max_fields",
        "fields": [
          {
            "name": "from_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "status",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_channel_map_min_fields",
        "fields": [
          {
            "name": "from_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "status",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_channel_map_order_by",
        "inputFields": [
          {
            "name": "connection",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_connection_map_order_by"
            }
          },
          {
            "name": "destination",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_chains_order_by"
            }
          },
          {
            "name": "from_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "from_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "from_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "from_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "from_port_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_chains_order_by"
            }
          },
          {
            "name": "status",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "to_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "to_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "to_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "to_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "to_port_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v0_channel_map_select_column",
        "enumValues": [
          {
            "name": "from_chain_id",
            "isDeprecated": false
          },
          {
            "name": "from_channel_id",
            "isDeprecated": false
          },
          {
            "name": "from_connection_id",
            "isDeprecated": false
          },
          {
            "name": "from_id",
            "isDeprecated": false
          },
          {
            "name": "from_port_id",
            "isDeprecated": false
          },
          {
            "name": "status",
            "isDeprecated": false
          },
          {
            "name": "to_chain_id",
            "isDeprecated": false
          },
          {
            "name": "to_channel_id",
            "isDeprecated": false
          },
          {
            "name": "to_connection_id",
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "isDeprecated": false
          },
          {
            "name": "to_port_id",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "OBJECT",
        "name": "v0_channel_map_stddev_fields",
        "fields": [
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_channel_map_stddev_pop_fields",
        "fields": [
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_channel_map_stddev_samp_fields",
        "fields": [
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_channel_map_sum_fields",
        "fields": [
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_channel_map_var_pop_fields",
        "fields": [
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_channel_map_var_samp_fields",
        "fields": [
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_channel_map_variance_fields",
        "fields": [
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_clients",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Int"
              }
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "client_id",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "String"
              }
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "counterparty_chain_id",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "String"
              }
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_clients_aggregate",
        "fields": [
          {
            "name": "aggregate",
            "type": {
              "kind": "OBJECT",
              "name": "v0_clients_aggregate_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "nodes",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_clients"
                  }
                }
              }
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_clients_aggregate_fields",
        "fields": [
          {
            "name": "avg",
            "type": {
              "kind": "OBJECT",
              "name": "v0_clients_avg_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "count",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Int"
              }
            },
            "args": [
              {
                "name": "columns",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_clients_select_column"
                    }
                  }
                }
              },
              {
                "name": "distinct",
                "type": {
                  "kind": "SCALAR",
                  "name": "Boolean"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "max",
            "type": {
              "kind": "OBJECT",
              "name": "v0_clients_max_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "min",
            "type": {
              "kind": "OBJECT",
              "name": "v0_clients_min_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev",
            "type": {
              "kind": "OBJECT",
              "name": "v0_clients_stddev_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "OBJECT",
              "name": "v0_clients_stddev_pop_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "OBJECT",
              "name": "v0_clients_stddev_samp_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sum",
            "type": {
              "kind": "OBJECT",
              "name": "v0_clients_sum_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "OBJECT",
              "name": "v0_clients_var_pop_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "OBJECT",
              "name": "v0_clients_var_samp_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "variance",
            "type": {
              "kind": "OBJECT",
              "name": "v0_clients_variance_fields"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_clients_avg_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_clients_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_clients_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_clients_bool_exp"
            }
          },
          {
            "name": "_or",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_clients_bool_exp"
                }
              }
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "client_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "counterparty_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_clients_max_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "client_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "counterparty_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_clients_min_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "client_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "counterparty_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_clients_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "client_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "counterparty_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v0_clients_select_column",
        "enumValues": [
          {
            "name": "chain_id",
            "isDeprecated": false
          },
          {
            "name": "client_id",
            "isDeprecated": false
          },
          {
            "name": "counterparty_chain_id",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "OBJECT",
        "name": "v0_clients_stddev_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_clients_stddev_pop_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_clients_stddev_samp_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_clients_sum_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_clients_var_pop_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_clients_var_samp_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_clients_variance_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_connection_map",
        "fields": [
          {
            "name": "from_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "status",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_connection_map_aggregate",
        "fields": [
          {
            "name": "aggregate",
            "type": {
              "kind": "OBJECT",
              "name": "v0_connection_map_aggregate_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "nodes",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_connection_map"
                  }
                }
              }
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_connection_map_aggregate_fields",
        "fields": [
          {
            "name": "avg",
            "type": {
              "kind": "OBJECT",
              "name": "v0_connection_map_avg_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "count",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Int"
              }
            },
            "args": [
              {
                "name": "columns",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_connection_map_select_column"
                    }
                  }
                }
              },
              {
                "name": "distinct",
                "type": {
                  "kind": "SCALAR",
                  "name": "Boolean"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "max",
            "type": {
              "kind": "OBJECT",
              "name": "v0_connection_map_max_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "min",
            "type": {
              "kind": "OBJECT",
              "name": "v0_connection_map_min_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev",
            "type": {
              "kind": "OBJECT",
              "name": "v0_connection_map_stddev_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "OBJECT",
              "name": "v0_connection_map_stddev_pop_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "OBJECT",
              "name": "v0_connection_map_stddev_samp_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sum",
            "type": {
              "kind": "OBJECT",
              "name": "v0_connection_map_sum_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "OBJECT",
              "name": "v0_connection_map_var_pop_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "OBJECT",
              "name": "v0_connection_map_var_samp_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "variance",
            "type": {
              "kind": "OBJECT",
              "name": "v0_connection_map_variance_fields"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_connection_map_avg_fields",
        "fields": [
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_connection_map_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_connection_map_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_connection_map_bool_exp"
            }
          },
          {
            "name": "_or",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_connection_map_bool_exp"
                }
              }
            }
          },
          {
            "name": "from_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "from_client_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "from_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "from_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "status",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "to_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "to_client_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "to_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "to_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_connection_map_max_fields",
        "fields": [
          {
            "name": "from_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "status",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_connection_map_min_fields",
        "fields": [
          {
            "name": "from_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "status",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_connection_map_order_by",
        "inputFields": [
          {
            "name": "from_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "from_client_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "from_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "from_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "status",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "to_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "to_client_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "to_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "to_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v0_connection_map_select_column",
        "enumValues": [
          {
            "name": "from_chain_id",
            "isDeprecated": false
          },
          {
            "name": "from_client_id",
            "isDeprecated": false
          },
          {
            "name": "from_connection_id",
            "isDeprecated": false
          },
          {
            "name": "from_id",
            "isDeprecated": false
          },
          {
            "name": "status",
            "isDeprecated": false
          },
          {
            "name": "to_chain_id",
            "isDeprecated": false
          },
          {
            "name": "to_client_id",
            "isDeprecated": false
          },
          {
            "name": "to_connection_id",
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "OBJECT",
        "name": "v0_connection_map_stddev_fields",
        "fields": [
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_connection_map_stddev_pop_fields",
        "fields": [
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_connection_map_stddev_samp_fields",
        "fields": [
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_connection_map_sum_fields",
        "fields": [
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_connection_map_var_pop_fields",
        "fields": [
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_connection_map_var_samp_fields",
        "fields": [
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_connection_map_variance_fields",
        "fields": [
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_burn",
        "fields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "block",
            "type": {
              "kind": "OBJECT",
              "name": "v0_blocks"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "burner",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "data",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            },
            "args": [
              {
                "name": "path",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "denom",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "mode",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_burn_aggregate",
        "fields": [
          {
            "name": "aggregate",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_burn_aggregate_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "nodes",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_cosmos_burn"
                  }
                }
              }
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_burn_aggregate_fields",
        "fields": [
          {
            "name": "avg",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_burn_avg_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "count",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Int"
              }
            },
            "args": [
              {
                "name": "columns",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_cosmos_burn_select_column"
                    }
                  }
                }
              },
              {
                "name": "distinct",
                "type": {
                  "kind": "SCALAR",
                  "name": "Boolean"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "max",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_burn_max_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "min",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_burn_min_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_burn_stddev_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_burn_stddev_pop_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_burn_stddev_samp_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sum",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_burn_sum_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_burn_var_pop_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_burn_var_samp_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "variance",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_burn_variance_fields"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_burn_avg_fields",
        "fields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_cosmos_burn_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_burn_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_cosmos_burn_bool_exp"
            }
          },
          {
            "name": "_or",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_burn_bool_exp"
                }
              }
            }
          },
          {
            "name": "amount",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "block",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_blocks_bool_exp"
            }
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "burner",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "denom",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "mode",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_burn_max_fields",
        "fields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "burner",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "denom",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "mode",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_burn_min_fields",
        "fields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "burner",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "denom",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "mode",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_cosmos_burn_order_by",
        "inputFields": [
          {
            "name": "amount",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "block",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_blocks_order_by"
            }
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "burner",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "denom",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "mode",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v0_cosmos_burn_select_column",
        "enumValues": [
          {
            "name": "amount",
            "isDeprecated": false
          },
          {
            "name": "block_hash",
            "isDeprecated": false
          },
          {
            "name": "burner",
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "isDeprecated": false
          },
          {
            "name": "data",
            "isDeprecated": false
          },
          {
            "name": "denom",
            "isDeprecated": false
          },
          {
            "name": "height",
            "isDeprecated": false
          },
          {
            "name": "index",
            "isDeprecated": false
          },
          {
            "name": "mode",
            "isDeprecated": false
          },
          {
            "name": "time",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_burn_stddev_fields",
        "fields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_burn_stddev_pop_fields",
        "fields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_burn_stddev_samp_fields",
        "fields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_cosmos_burn_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v0_cosmos_burn_stream_cursor_value_input"
              }
            }
          },
          {
            "name": "ordering",
            "type": {
              "kind": "ENUM",
              "name": "cursor_ordering"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_cosmos_burn_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "burner",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "denom",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "mode",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_burn_sum_fields",
        "fields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_burn_var_pop_fields",
        "fields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_burn_var_samp_fields",
        "fields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_burn_variance_fields",
        "fields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_ack",
        "fields": [
          {
            "name": "block",
            "type": {
              "kind": "OBJECT",
              "name": "v0_blocks"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "counterparty_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "counterparty_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "data",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            },
            "args": [
              {
                "name": "path",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_ack_aggregate",
        "fields": [
          {
            "name": "aggregate",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_channel_open_ack_aggregate_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "nodes",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_cosmos_channel_open_ack"
                  }
                }
              }
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_ack_aggregate_fields",
        "fields": [
          {
            "name": "avg",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_channel_open_ack_avg_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "count",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Int"
              }
            },
            "args": [
              {
                "name": "columns",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_cosmos_channel_open_ack_select_column"
                    }
                  }
                }
              },
              {
                "name": "distinct",
                "type": {
                  "kind": "SCALAR",
                  "name": "Boolean"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "max",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_channel_open_ack_max_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "min",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_channel_open_ack_min_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_channel_open_ack_stddev_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_channel_open_ack_stddev_pop_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_channel_open_ack_stddev_samp_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sum",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_channel_open_ack_sum_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_channel_open_ack_var_pop_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_channel_open_ack_var_samp_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "variance",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_channel_open_ack_variance_fields"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_ack_avg_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_cosmos_channel_open_ack_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_channel_open_ack_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_cosmos_channel_open_ack_bool_exp"
            }
          },
          {
            "name": "_or",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_channel_open_ack_bool_exp"
                }
              }
            }
          },
          {
            "name": "block",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_blocks_bool_exp"
            }
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "counterparty_channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "counterparty_port_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "port_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_ack_max_fields",
        "fields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "counterparty_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "counterparty_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_ack_min_fields",
        "fields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "counterparty_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "counterparty_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_cosmos_channel_open_ack_order_by",
        "inputFields": [
          {
            "name": "block",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_blocks_order_by"
            }
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "counterparty_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "counterparty_port_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "port_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v0_cosmos_channel_open_ack_select_column",
        "enumValues": [
          {
            "name": "block_hash",
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "isDeprecated": false
          },
          {
            "name": "channel_id",
            "isDeprecated": false
          },
          {
            "name": "connection_id",
            "isDeprecated": false
          },
          {
            "name": "counterparty_channel_id",
            "isDeprecated": false
          },
          {
            "name": "counterparty_port_id",
            "isDeprecated": false
          },
          {
            "name": "data",
            "isDeprecated": false
          },
          {
            "name": "height",
            "isDeprecated": false
          },
          {
            "name": "index",
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "isDeprecated": false
          },
          {
            "name": "port_id",
            "isDeprecated": false
          },
          {
            "name": "time",
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_ack_stddev_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_ack_stddev_pop_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_ack_stddev_samp_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_cosmos_channel_open_ack_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v0_cosmos_channel_open_ack_stream_cursor_value_input"
              }
            }
          },
          {
            "name": "ordering",
            "type": {
              "kind": "ENUM",
              "name": "cursor_ordering"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_cosmos_channel_open_ack_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "counterparty_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "counterparty_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_ack_sum_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_ack_var_pop_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_ack_var_samp_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_ack_variance_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_init",
        "fields": [
          {
            "name": "block",
            "type": {
              "kind": "OBJECT",
              "name": "v0_blocks"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "counterparty_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "data",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            },
            "args": [
              {
                "name": "path",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "version",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_init_aggregate",
        "fields": [
          {
            "name": "aggregate",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_channel_open_init_aggregate_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "nodes",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_cosmos_channel_open_init"
                  }
                }
              }
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_init_aggregate_fields",
        "fields": [
          {
            "name": "avg",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_channel_open_init_avg_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "count",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Int"
              }
            },
            "args": [
              {
                "name": "columns",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_cosmos_channel_open_init_select_column"
                    }
                  }
                }
              },
              {
                "name": "distinct",
                "type": {
                  "kind": "SCALAR",
                  "name": "Boolean"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "max",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_channel_open_init_max_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "min",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_channel_open_init_min_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_channel_open_init_stddev_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_channel_open_init_stddev_pop_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_channel_open_init_stddev_samp_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sum",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_channel_open_init_sum_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_channel_open_init_var_pop_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_channel_open_init_var_samp_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "variance",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_channel_open_init_variance_fields"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_init_avg_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_cosmos_channel_open_init_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_channel_open_init_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_cosmos_channel_open_init_bool_exp"
            }
          },
          {
            "name": "_or",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_channel_open_init_bool_exp"
                }
              }
            }
          },
          {
            "name": "block",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_blocks_bool_exp"
            }
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "counterparty_port_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "port_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "version",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_init_max_fields",
        "fields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "counterparty_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "version",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_init_min_fields",
        "fields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "counterparty_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "version",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_cosmos_channel_open_init_order_by",
        "inputFields": [
          {
            "name": "block",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_blocks_order_by"
            }
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "counterparty_port_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "port_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "version",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v0_cosmos_channel_open_init_select_column",
        "enumValues": [
          {
            "name": "block_hash",
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "isDeprecated": false
          },
          {
            "name": "channel_id",
            "isDeprecated": false
          },
          {
            "name": "connection_id",
            "isDeprecated": false
          },
          {
            "name": "counterparty_port_id",
            "isDeprecated": false
          },
          {
            "name": "data",
            "isDeprecated": false
          },
          {
            "name": "height",
            "isDeprecated": false
          },
          {
            "name": "index",
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "isDeprecated": false
          },
          {
            "name": "port_id",
            "isDeprecated": false
          },
          {
            "name": "time",
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "isDeprecated": false
          },
          {
            "name": "version",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_init_stddev_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_init_stddev_pop_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_init_stddev_samp_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_cosmos_channel_open_init_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v0_cosmos_channel_open_init_stream_cursor_value_input"
              }
            }
          },
          {
            "name": "ordering",
            "type": {
              "kind": "ENUM",
              "name": "cursor_ordering"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_cosmos_channel_open_init_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "counterparty_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "version",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_init_sum_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_init_var_pop_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_init_var_samp_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_channel_open_init_variance_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_transfer",
        "fields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "block",
            "type": {
              "kind": "OBJECT",
              "name": "v0_blocks"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "data",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            },
            "args": [
              {
                "name": "path",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "denom",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "mode",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "recipient",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sender",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_transfer_aggregate",
        "fields": [
          {
            "name": "aggregate",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_transfer_aggregate_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "nodes",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_cosmos_transfer"
                  }
                }
              }
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_transfer_aggregate_fields",
        "fields": [
          {
            "name": "avg",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_transfer_avg_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "count",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Int"
              }
            },
            "args": [
              {
                "name": "columns",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_cosmos_transfer_select_column"
                    }
                  }
                }
              },
              {
                "name": "distinct",
                "type": {
                  "kind": "SCALAR",
                  "name": "Boolean"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "max",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_transfer_max_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "min",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_transfer_min_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_transfer_stddev_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_transfer_stddev_pop_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_transfer_stddev_samp_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sum",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_transfer_sum_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_transfer_var_pop_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_transfer_var_samp_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "variance",
            "type": {
              "kind": "OBJECT",
              "name": "v0_cosmos_transfer_variance_fields"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_transfer_avg_fields",
        "fields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_cosmos_transfer_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_transfer_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_cosmos_transfer_bool_exp"
            }
          },
          {
            "name": "_or",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_transfer_bool_exp"
                }
              }
            }
          },
          {
            "name": "amount",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "block",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_blocks_bool_exp"
            }
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "denom",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "mode",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "recipient",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "sender",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_transfer_max_fields",
        "fields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "denom",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "mode",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "recipient",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sender",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_transfer_min_fields",
        "fields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "denom",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "mode",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "recipient",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sender",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_cosmos_transfer_order_by",
        "inputFields": [
          {
            "name": "amount",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "block",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_blocks_order_by"
            }
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "denom",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "mode",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "recipient",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "sender",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v0_cosmos_transfer_select_column",
        "enumValues": [
          {
            "name": "amount",
            "isDeprecated": false
          },
          {
            "name": "block_hash",
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "isDeprecated": false
          },
          {
            "name": "data",
            "isDeprecated": false
          },
          {
            "name": "denom",
            "isDeprecated": false
          },
          {
            "name": "height",
            "isDeprecated": false
          },
          {
            "name": "index",
            "isDeprecated": false
          },
          {
            "name": "mode",
            "isDeprecated": false
          },
          {
            "name": "recipient",
            "isDeprecated": false
          },
          {
            "name": "sender",
            "isDeprecated": false
          },
          {
            "name": "time",
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_transfer_stddev_fields",
        "fields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_transfer_stddev_pop_fields",
        "fields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_transfer_stddev_samp_fields",
        "fields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_cosmos_transfer_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v0_cosmos_transfer_stream_cursor_value_input"
              }
            }
          },
          {
            "name": "ordering",
            "type": {
              "kind": "ENUM",
              "name": "cursor_ordering"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_cosmos_transfer_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "denom",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "mode",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "recipient",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "sender",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_transfer_sum_fields",
        "fields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_transfer_var_pop_fields",
        "fields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_transfer_var_samp_fields",
        "fields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_transfer_variance_fields",
        "fields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_wasm_message",
        "fields": [
          {
            "name": "_contract_address",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "block",
            "type": {
              "kind": "OBJECT",
              "name": "v0_blocks"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "data",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            },
            "args": [
              {
                "name": "path",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "module",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_cosmos_wasm_message_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_wasm_message_bool_exp"
                }
              }
            }
          },
          {
            "name": "_contract_address",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_cosmos_wasm_message_bool_exp"
            }
          },
          {
            "name": "_or",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_wasm_message_bool_exp"
                }
              }
            }
          },
          {
            "name": "block",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_blocks_bool_exp"
            }
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "module",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_cosmos_wasm_message_order_by",
        "inputFields": [
          {
            "name": "_contract_address",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "block",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_blocks_order_by"
            }
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "module",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v0_cosmos_wasm_message_select_column",
        "enumValues": [
          {
            "name": "_contract_address",
            "isDeprecated": false
          },
          {
            "name": "block_hash",
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "isDeprecated": false
          },
          {
            "name": "data",
            "isDeprecated": false
          },
          {
            "name": "height",
            "isDeprecated": false
          },
          {
            "name": "index",
            "isDeprecated": false
          },
          {
            "name": "module",
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "isDeprecated": false
          },
          {
            "name": "time",
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_cosmos_wasm_message_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v0_cosmos_wasm_message_stream_cursor_value_input"
              }
            }
          },
          {
            "name": "ordering",
            "type": {
              "kind": "ENUM",
              "name": "cursor_ordering"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_cosmos_wasm_message_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "_contract_address",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "module",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_cosmos_withdraw_rewards",
        "fields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "block",
            "type": {
              "kind": "OBJECT",
              "name": "v0_blocks"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "data",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            },
            "args": [
              {
                "name": "path",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "delegator",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "denom",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "validator",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_cosmos_withdraw_rewards_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_withdraw_rewards_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_cosmos_withdraw_rewards_bool_exp"
            }
          },
          {
            "name": "_or",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_cosmos_withdraw_rewards_bool_exp"
                }
              }
            }
          },
          {
            "name": "amount",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "block",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_blocks_bool_exp"
            }
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "delegator",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "denom",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "validator",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_cosmos_withdraw_rewards_order_by",
        "inputFields": [
          {
            "name": "amount",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "block",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_blocks_order_by"
            }
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "delegator",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "denom",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "validator",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v0_cosmos_withdraw_rewards_select_column",
        "enumValues": [
          {
            "name": "amount",
            "isDeprecated": false
          },
          {
            "name": "block_hash",
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "isDeprecated": false
          },
          {
            "name": "data",
            "isDeprecated": false
          },
          {
            "name": "delegator",
            "isDeprecated": false
          },
          {
            "name": "denom",
            "isDeprecated": false
          },
          {
            "name": "height",
            "isDeprecated": false
          },
          {
            "name": "index",
            "isDeprecated": false
          },
          {
            "name": "msg_index",
            "isDeprecated": false
          },
          {
            "name": "time",
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "isDeprecated": false
          },
          {
            "name": "validator",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_cosmos_withdraw_rewards_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v0_cosmos_withdraw_rewards_stream_cursor_value_input"
              }
            }
          },
          {
            "name": "ordering",
            "type": {
              "kind": "ENUM",
              "name": "cursor_ordering"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_cosmos_withdraw_rewards_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "delegator",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "denom",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "msg_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "validator",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_evm_client_created",
        "fields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "client_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "log_index",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "log_to_jsonb",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            },
            "args": [
              {
                "name": "path",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "name",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "raw_log",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            },
            "args": [
              {
                "name": "path",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_evm_client_created_aggregate",
        "fields": [
          {
            "name": "aggregate",
            "type": {
              "kind": "OBJECT",
              "name": "v0_evm_client_created_aggregate_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "nodes",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_evm_client_created"
                  }
                }
              }
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_evm_client_created_aggregate_fields",
        "fields": [
          {
            "name": "avg",
            "type": {
              "kind": "OBJECT",
              "name": "v0_evm_client_created_avg_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "count",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Int"
              }
            },
            "args": [
              {
                "name": "columns",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_evm_client_created_select_column"
                    }
                  }
                }
              },
              {
                "name": "distinct",
                "type": {
                  "kind": "SCALAR",
                  "name": "Boolean"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "max",
            "type": {
              "kind": "OBJECT",
              "name": "v0_evm_client_created_max_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "min",
            "type": {
              "kind": "OBJECT",
              "name": "v0_evm_client_created_min_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev",
            "type": {
              "kind": "OBJECT",
              "name": "v0_evm_client_created_stddev_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "OBJECT",
              "name": "v0_evm_client_created_stddev_pop_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "OBJECT",
              "name": "v0_evm_client_created_stddev_samp_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sum",
            "type": {
              "kind": "OBJECT",
              "name": "v0_evm_client_created_sum_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "OBJECT",
              "name": "v0_evm_client_created_var_pop_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "OBJECT",
              "name": "v0_evm_client_created_var_samp_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "variance",
            "type": {
              "kind": "OBJECT",
              "name": "v0_evm_client_created_variance_fields"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_evm_client_created_avg_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_evm_client_created_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_evm_client_created_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_evm_client_created_bool_exp"
            }
          },
          {
            "name": "_or",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_evm_client_created_bool_exp"
                }
              }
            }
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "client_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "log_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "log_to_jsonb",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "name",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "raw_log",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_evm_client_created_max_fields",
        "fields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "client_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "log_index",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "name",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_evm_client_created_min_fields",
        "fields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "client_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "log_index",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "name",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_evm_client_created_order_by",
        "inputFields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "client_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "log_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "log_to_jsonb",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "name",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "raw_log",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v0_evm_client_created_select_column",
        "enumValues": [
          {
            "name": "block_hash",
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "isDeprecated": false
          },
          {
            "name": "client_id",
            "isDeprecated": false
          },
          {
            "name": "height",
            "isDeprecated": false
          },
          {
            "name": "log_index",
            "isDeprecated": false
          },
          {
            "name": "log_to_jsonb",
            "isDeprecated": false
          },
          {
            "name": "name",
            "isDeprecated": false
          },
          {
            "name": "raw_log",
            "isDeprecated": false
          },
          {
            "name": "time",
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "OBJECT",
        "name": "v0_evm_client_created_stddev_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_evm_client_created_stddev_pop_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_evm_client_created_stddev_samp_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_evm_client_created_sum_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_evm_client_created_var_pop_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_evm_client_created_var_samp_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_evm_client_created_variance_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_evm_recv_packet",
        "fields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "data",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_channel",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_port",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "log_index",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "log_to_jsonb",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            },
            "args": [
              {
                "name": "path",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "name",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            },
            "args": [
              {
                "name": "path",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "raw_log",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            },
            "args": [
              {
                "name": "path",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "revision_height",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "revision_number",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sequence",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_channel",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_port",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "timeout_height",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            },
            "args": [
              {
                "name": "path",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "timeout_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_evm_recv_packet_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_evm_recv_packet_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_evm_recv_packet_bool_exp"
            }
          },
          {
            "name": "_or",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_evm_recv_packet_bool_exp"
                }
              }
            }
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_channel",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_port",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "log_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "log_to_jsonb",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "name",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "raw_log",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "revision_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "numeric_comparison_exp"
            }
          },
          {
            "name": "revision_number",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "numeric_comparison_exp"
            }
          },
          {
            "name": "sequence",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "numeric_comparison_exp"
            }
          },
          {
            "name": "source_channel",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_port",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "timeout_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "timeout_timestamp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "numeric_comparison_exp"
            }
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_evm_recv_packet_order_by",
        "inputFields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_channel",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_port",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "log_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "log_to_jsonb",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "name",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "raw_log",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "revision_height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "revision_number",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "sequence",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_channel",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_port",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "timeout_height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "timeout_timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v0_evm_recv_packet_select_column",
        "enumValues": [
          {
            "name": "block_hash",
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "isDeprecated": false
          },
          {
            "name": "data",
            "isDeprecated": false
          },
          {
            "name": "destination_channel",
            "isDeprecated": false
          },
          {
            "name": "destination_port",
            "isDeprecated": false
          },
          {
            "name": "height",
            "isDeprecated": false
          },
          {
            "name": "log_index",
            "isDeprecated": false
          },
          {
            "name": "log_to_jsonb",
            "isDeprecated": false
          },
          {
            "name": "name",
            "isDeprecated": false
          },
          {
            "name": "packet",
            "isDeprecated": false
          },
          {
            "name": "raw_log",
            "isDeprecated": false
          },
          {
            "name": "revision_height",
            "isDeprecated": false
          },
          {
            "name": "revision_number",
            "isDeprecated": false
          },
          {
            "name": "sequence",
            "isDeprecated": false
          },
          {
            "name": "source_channel",
            "isDeprecated": false
          },
          {
            "name": "source_port",
            "isDeprecated": false
          },
          {
            "name": "time",
            "isDeprecated": false
          },
          {
            "name": "timeout_height",
            "isDeprecated": false
          },
          {
            "name": "timeout_timestamp",
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "OBJECT",
        "name": "v0_index_status",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "display_name",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "status",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "tip_age_seconds",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_index_status_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_index_status_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_index_status_bool_exp"
            }
          },
          {
            "name": "_or",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_index_status_bool_exp"
                }
              }
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "display_name",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "status",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "timestamp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "tip_age_seconds",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "numeric_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_index_status_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "display_name",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "status",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "tip_age_seconds",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v0_index_status_select_column",
        "enumValues": [
          {
            "name": "chain_id",
            "isDeprecated": false
          },
          {
            "name": "display_name",
            "isDeprecated": false
          },
          {
            "name": "height",
            "isDeprecated": false
          },
          {
            "name": "id",
            "isDeprecated": false
          },
          {
            "name": "status",
            "isDeprecated": false
          },
          {
            "name": "timestamp",
            "isDeprecated": false
          },
          {
            "name": "tip_age_seconds",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_index_status_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v0_index_status_stream_cursor_value_input"
              }
            }
          },
          {
            "name": "ordering",
            "type": {
              "kind": "ENUM",
              "name": "cursor_ordering"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_index_status_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "display_name",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "status",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "tip_age_seconds",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_logs",
        "fields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "String"
              }
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_chains"
              }
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Int"
              }
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "data",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "jsonb"
              }
            },
            "args": [
              {
                "name": "path",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Int"
              }
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "time",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "timestamptz"
              }
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_logs_aggregate",
        "fields": [
          {
            "name": "aggregate",
            "type": {
              "kind": "OBJECT",
              "name": "v0_logs_aggregate_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "nodes",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_logs"
                  }
                }
              }
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_logs_aggregate_bool_exp",
        "inputFields": [
          {
            "name": "count",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_logs_aggregate_bool_exp_count"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_logs_aggregate_bool_exp_count",
        "inputFields": [
          {
            "name": "arguments",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "ENUM",
                  "name": "v0_logs_select_column"
                }
              }
            }
          },
          {
            "name": "distinct",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "filter",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_logs_bool_exp"
            }
          },
          {
            "name": "predicate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "Int_comparison_exp"
              }
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_logs_aggregate_fields",
        "fields": [
          {
            "name": "avg",
            "type": {
              "kind": "OBJECT",
              "name": "v0_logs_avg_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "count",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Int"
              }
            },
            "args": [
              {
                "name": "columns",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_logs_select_column"
                    }
                  }
                }
              },
              {
                "name": "distinct",
                "type": {
                  "kind": "SCALAR",
                  "name": "Boolean"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "max",
            "type": {
              "kind": "OBJECT",
              "name": "v0_logs_max_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "min",
            "type": {
              "kind": "OBJECT",
              "name": "v0_logs_min_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev",
            "type": {
              "kind": "OBJECT",
              "name": "v0_logs_stddev_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "OBJECT",
              "name": "v0_logs_stddev_pop_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "OBJECT",
              "name": "v0_logs_stddev_samp_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sum",
            "type": {
              "kind": "OBJECT",
              "name": "v0_logs_sum_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "OBJECT",
              "name": "v0_logs_var_pop_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "OBJECT",
              "name": "v0_logs_var_samp_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "variance",
            "type": {
              "kind": "OBJECT",
              "name": "v0_logs_variance_fields"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_logs_aggregate_order_by",
        "inputFields": [
          {
            "name": "avg",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_logs_avg_order_by"
            }
          },
          {
            "name": "count",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "max",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_logs_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_logs_min_order_by"
            }
          },
          {
            "name": "stddev",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_logs_stddev_order_by"
            }
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_logs_stddev_pop_order_by"
            }
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_logs_stddev_samp_order_by"
            }
          },
          {
            "name": "sum",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_logs_sum_order_by"
            }
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_logs_var_pop_order_by"
            }
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_logs_var_samp_order_by"
            }
          },
          {
            "name": "variance",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_logs_variance_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_logs_avg_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_logs_avg_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_logs_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_logs_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_logs_bool_exp"
            }
          },
          {
            "name": "_or",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_logs_bool_exp"
                }
              }
            }
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_chains_bool_exp"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_logs_max_fields",
        "fields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_logs_max_order_by",
        "inputFields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_logs_min_fields",
        "fields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_logs_min_order_by",
        "inputFields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_logs_order_by",
        "inputFields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_chains_order_by"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v0_logs_select_column",
        "enumValues": [
          {
            "name": "block_hash",
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "isDeprecated": false
          },
          {
            "name": "data",
            "isDeprecated": false
          },
          {
            "name": "height",
            "isDeprecated": false
          },
          {
            "name": "time",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "OBJECT",
        "name": "v0_logs_stddev_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_logs_stddev_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_logs_stddev_pop_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_logs_stddev_pop_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_logs_stddev_samp_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_logs_stddev_samp_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_logs_sum_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_logs_sum_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_logs_var_pop_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_logs_var_pop_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_logs_var_samp_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_logs_var_samp_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_logs_variance_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_logs_variance_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_packets",
        "fields": [
          {
            "name": "destination_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_channel",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_data",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_json",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            },
            "args": [
              {
                "name": "path",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "destination_port",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_sequence",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_timeout_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_channel",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_data",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_json",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            },
            "args": [
              {
                "name": "path",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "source_port",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_sequence",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_timeout_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "status",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_packets_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_packets_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_packets_bool_exp"
            }
          },
          {
            "name": "_or",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_packets_bool_exp"
                }
              }
            }
          },
          {
            "name": "destination_block_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "destination_channel",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_data",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "destination_json",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "destination_port",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_sequence",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "numeric_comparison_exp"
            }
          },
          {
            "name": "destination_time",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "destination_timeout_timestamp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "numeric_comparison_exp"
            }
          },
          {
            "name": "destination_transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_transaction_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "from_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "from_channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "from_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "from_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "from_port_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_block_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "source_channel",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_data",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "source_json",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "source_port",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_sequence",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "numeric_comparison_exp"
            }
          },
          {
            "name": "source_time",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "source_timeout_timestamp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "numeric_comparison_exp"
            }
          },
          {
            "name": "source_transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_transaction_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "status",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "to_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "to_channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "to_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "to_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "to_port_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_packets_order_by",
        "inputFields": [
          {
            "name": "destination_block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_channel",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_data",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_json",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_port",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_sequence",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_time",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_timeout_timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_transaction_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "from_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "from_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "from_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "from_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "from_port_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_channel",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_data",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_json",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_port",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_sequence",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_time",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_timeout_timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_transaction_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "status",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "to_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "to_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "to_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "to_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "to_port_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v0_packets_select_column",
        "enumValues": [
          {
            "name": "destination_block_hash",
            "isDeprecated": false
          },
          {
            "name": "destination_chain_id",
            "isDeprecated": false
          },
          {
            "name": "destination_channel",
            "isDeprecated": false
          },
          {
            "name": "destination_data",
            "isDeprecated": false
          },
          {
            "name": "destination_height",
            "isDeprecated": false
          },
          {
            "name": "destination_json",
            "isDeprecated": false
          },
          {
            "name": "destination_port",
            "isDeprecated": false
          },
          {
            "name": "destination_sequence",
            "isDeprecated": false
          },
          {
            "name": "destination_time",
            "isDeprecated": false
          },
          {
            "name": "destination_timeout_timestamp",
            "isDeprecated": false
          },
          {
            "name": "destination_transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "destination_transaction_index",
            "isDeprecated": false
          },
          {
            "name": "from_chain_id",
            "isDeprecated": false
          },
          {
            "name": "from_channel_id",
            "isDeprecated": false
          },
          {
            "name": "from_connection_id",
            "isDeprecated": false
          },
          {
            "name": "from_id",
            "isDeprecated": false
          },
          {
            "name": "from_port_id",
            "isDeprecated": false
          },
          {
            "name": "source_block_hash",
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
            "isDeprecated": false
          },
          {
            "name": "source_channel",
            "isDeprecated": false
          },
          {
            "name": "source_data",
            "isDeprecated": false
          },
          {
            "name": "source_height",
            "isDeprecated": false
          },
          {
            "name": "source_json",
            "isDeprecated": false
          },
          {
            "name": "source_port",
            "isDeprecated": false
          },
          {
            "name": "source_sequence",
            "isDeprecated": false
          },
          {
            "name": "source_time",
            "isDeprecated": false
          },
          {
            "name": "source_timeout_timestamp",
            "isDeprecated": false
          },
          {
            "name": "source_transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "source_transaction_index",
            "isDeprecated": false
          },
          {
            "name": "status",
            "isDeprecated": false
          },
          {
            "name": "to_chain_id",
            "isDeprecated": false
          },
          {
            "name": "to_channel_id",
            "isDeprecated": false
          },
          {
            "name": "to_connection_id",
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "isDeprecated": false
          },
          {
            "name": "to_port_id",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_packets_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v0_packets_stream_cursor_value_input"
              }
            }
          },
          {
            "name": "ordering",
            "type": {
              "kind": "ENUM",
              "name": "cursor_ordering"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_packets_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "destination_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "destination_channel",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_data",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "destination_json",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "destination_port",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_sequence",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            }
          },
          {
            "name": "destination_time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "destination_timeout_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            }
          },
          {
            "name": "destination_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "from_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "from_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "from_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "from_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "source_channel",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_data",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "source_json",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "source_port",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_sequence",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            }
          },
          {
            "name": "source_time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "source_timeout_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            }
          },
          {
            "name": "source_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "status",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "to_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "to_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "to_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "to_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_recv_packet",
        "fields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "channel",
            "type": {
              "kind": "OBJECT",
              "name": "v0_channel_map"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "data",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_channel",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_port",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "json",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            },
            "args": [
              {
                "name": "path",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "sequence",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_channel",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_port",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "timeout_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_recv_packet_aggregate",
        "fields": [
          {
            "name": "aggregate",
            "type": {
              "kind": "OBJECT",
              "name": "v0_recv_packet_aggregate_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "nodes",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_recv_packet"
                  }
                }
              }
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_recv_packet_aggregate_fields",
        "fields": [
          {
            "name": "avg",
            "type": {
              "kind": "OBJECT",
              "name": "v0_recv_packet_avg_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "count",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Int"
              }
            },
            "args": [
              {
                "name": "columns",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "v0_recv_packet_select_column"
                    }
                  }
                }
              },
              {
                "name": "distinct",
                "type": {
                  "kind": "SCALAR",
                  "name": "Boolean"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "max",
            "type": {
              "kind": "OBJECT",
              "name": "v0_recv_packet_max_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "min",
            "type": {
              "kind": "OBJECT",
              "name": "v0_recv_packet_min_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev",
            "type": {
              "kind": "OBJECT",
              "name": "v0_recv_packet_stddev_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "OBJECT",
              "name": "v0_recv_packet_stddev_pop_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "OBJECT",
              "name": "v0_recv_packet_stddev_samp_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sum",
            "type": {
              "kind": "OBJECT",
              "name": "v0_recv_packet_sum_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "OBJECT",
              "name": "v0_recv_packet_var_pop_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "OBJECT",
              "name": "v0_recv_packet_var_samp_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "variance",
            "type": {
              "kind": "OBJECT",
              "name": "v0_recv_packet_variance_fields"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_recv_packet_avg_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sequence",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "timeout_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_recv_packet_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_recv_packet_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_recv_packet_bool_exp"
            }
          },
          {
            "name": "_or",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_recv_packet_bool_exp"
                }
              }
            }
          },
          {
            "name": "block_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "channel",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_channel_map_bool_exp"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_channel",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_port",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "json",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "sequence",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "numeric_comparison_exp"
            }
          },
          {
            "name": "source_channel",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_port",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "timeout_timestamp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "numeric_comparison_exp"
            }
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_recv_packet_max_fields",
        "fields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "data",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_channel",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_port",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sequence",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_channel",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_port",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "timeout_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_recv_packet_min_fields",
        "fields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "data",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_channel",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_port",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sequence",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_channel",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_port",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "timeout_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_recv_packet_order_by",
        "inputFields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "channel",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_channel_map_order_by"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_channel",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_port",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "json",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "sequence",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_channel",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_port",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "timeout_timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v0_recv_packet_select_column",
        "enumValues": [
          {
            "name": "block_hash",
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "isDeprecated": false
          },
          {
            "name": "data",
            "isDeprecated": false
          },
          {
            "name": "destination_channel",
            "isDeprecated": false
          },
          {
            "name": "destination_port",
            "isDeprecated": false
          },
          {
            "name": "height",
            "isDeprecated": false
          },
          {
            "name": "json",
            "isDeprecated": false
          },
          {
            "name": "sequence",
            "isDeprecated": false
          },
          {
            "name": "source_channel",
            "isDeprecated": false
          },
          {
            "name": "source_port",
            "isDeprecated": false
          },
          {
            "name": "time",
            "isDeprecated": false
          },
          {
            "name": "timeout_timestamp",
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "transaction_index",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "OBJECT",
        "name": "v0_recv_packet_stddev_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sequence",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "timeout_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_recv_packet_stddev_pop_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sequence",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "timeout_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_recv_packet_stddev_samp_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sequence",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "timeout_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_recv_packet_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v0_recv_packet_stream_cursor_value_input"
              }
            }
          },
          {
            "name": "ordering",
            "type": {
              "kind": "ENUM",
              "name": "cursor_ordering"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_recv_packet_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "data",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_channel",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_port",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "json",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "sequence",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            }
          },
          {
            "name": "source_channel",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_port",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "timeout_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            }
          },
          {
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_recv_packet_sum_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sequence",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "timeout_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_recv_packet_var_pop_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sequence",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "timeout_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_recv_packet_var_samp_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sequence",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "timeout_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_recv_packet_variance_fields",
        "fields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sequence",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "timeout_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_transfers",
        "fields": [
          {
            "name": "assets",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            },
            "args": [
              {
                "name": "path",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "destination_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_channel",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_data",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_json",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            },
            "args": [
              {
                "name": "path",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "destination_port",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_sequence",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_timeout_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "from_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "receiver",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sender",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_channel",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_data",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_json",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            },
            "args": [
              {
                "name": "path",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "source_port",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_sequence",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_timeout_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "status",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "to_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_transfers_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_transfers_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transfers_bool_exp"
            }
          },
          {
            "name": "_or",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_transfers_bool_exp"
                }
              }
            }
          },
          {
            "name": "assets",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "destination_block_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "destination_channel",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_data",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "destination_json",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "destination_port",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_sequence",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "numeric_comparison_exp"
            }
          },
          {
            "name": "destination_time",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "destination_timeout_timestamp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "numeric_comparison_exp"
            }
          },
          {
            "name": "destination_transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_transaction_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "from_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "from_channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "from_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "from_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "from_port_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "receiver",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "sender",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_block_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "source_channel",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_data",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "source_json",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "source_port",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_sequence",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "numeric_comparison_exp"
            }
          },
          {
            "name": "source_time",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "source_timeout_timestamp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "numeric_comparison_exp"
            }
          },
          {
            "name": "source_transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_transaction_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "status",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "to_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "to_channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "to_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "to_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "to_port_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_transfers_order_by",
        "inputFields": [
          {
            "name": "assets",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_channel",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_data",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_json",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_port",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_sequence",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_time",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_timeout_timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_transaction_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "from_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "from_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "from_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "from_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "from_port_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "receiver",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "sender",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_channel",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_data",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_json",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_port",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_sequence",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_time",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_timeout_timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_transaction_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "status",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "to_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "to_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "to_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "to_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "to_port_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v0_transfers_select_column",
        "enumValues": [
          {
            "name": "assets",
            "isDeprecated": false
          },
          {
            "name": "destination_block_hash",
            "isDeprecated": false
          },
          {
            "name": "destination_chain_id",
            "isDeprecated": false
          },
          {
            "name": "destination_channel",
            "isDeprecated": false
          },
          {
            "name": "destination_data",
            "isDeprecated": false
          },
          {
            "name": "destination_height",
            "isDeprecated": false
          },
          {
            "name": "destination_json",
            "isDeprecated": false
          },
          {
            "name": "destination_port",
            "isDeprecated": false
          },
          {
            "name": "destination_sequence",
            "isDeprecated": false
          },
          {
            "name": "destination_time",
            "isDeprecated": false
          },
          {
            "name": "destination_timeout_timestamp",
            "isDeprecated": false
          },
          {
            "name": "destination_transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "destination_transaction_index",
            "isDeprecated": false
          },
          {
            "name": "from_chain_id",
            "isDeprecated": false
          },
          {
            "name": "from_channel_id",
            "isDeprecated": false
          },
          {
            "name": "from_connection_id",
            "isDeprecated": false
          },
          {
            "name": "from_id",
            "isDeprecated": false
          },
          {
            "name": "from_port_id",
            "isDeprecated": false
          },
          {
            "name": "receiver",
            "isDeprecated": false
          },
          {
            "name": "sender",
            "isDeprecated": false
          },
          {
            "name": "source_block_hash",
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
            "isDeprecated": false
          },
          {
            "name": "source_channel",
            "isDeprecated": false
          },
          {
            "name": "source_data",
            "isDeprecated": false
          },
          {
            "name": "source_height",
            "isDeprecated": false
          },
          {
            "name": "source_json",
            "isDeprecated": false
          },
          {
            "name": "source_port",
            "isDeprecated": false
          },
          {
            "name": "source_sequence",
            "isDeprecated": false
          },
          {
            "name": "source_time",
            "isDeprecated": false
          },
          {
            "name": "source_timeout_timestamp",
            "isDeprecated": false
          },
          {
            "name": "source_transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "source_transaction_index",
            "isDeprecated": false
          },
          {
            "name": "status",
            "isDeprecated": false
          },
          {
            "name": "to_chain_id",
            "isDeprecated": false
          },
          {
            "name": "to_channel_id",
            "isDeprecated": false
          },
          {
            "name": "to_connection_id",
            "isDeprecated": false
          },
          {
            "name": "to_id",
            "isDeprecated": false
          },
          {
            "name": "to_port_id",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_transfers_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v0_transfers_stream_cursor_value_input"
              }
            }
          },
          {
            "name": "ordering",
            "type": {
              "kind": "ENUM",
              "name": "cursor_ordering"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_transfers_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "assets",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "destination_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "destination_channel",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_data",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "destination_json",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "destination_port",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_sequence",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            }
          },
          {
            "name": "destination_time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "destination_timeout_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            }
          },
          {
            "name": "destination_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "from_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "from_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "from_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "from_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "from_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "receiver",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "sender",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "source_channel",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_data",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_height",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "source_json",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "source_port",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_sequence",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            }
          },
          {
            "name": "source_time",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "source_timeout_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            }
          },
          {
            "name": "source_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "status",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "to_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "to_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "to_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "to_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "to_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          }
        ],
        "isOneOf": false
      }
    ],
    "directives": []
  }
};

import * as gqlTada from 'gql.tada';

declare module 'gql.tada' {
  interface setupSchema {
    introspection: introspection
  }
}