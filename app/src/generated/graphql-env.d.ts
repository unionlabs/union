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
    "mutationType": {
      "name": "mutation_root"
    },
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
        "kind": "OBJECT",
        "name": "PfmArgs",
        "fields": [
          {
            "name": "channel",
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
            "name": "memo",
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
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "PfmArgs_bool_exp_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "PfmArgs_bool_exp_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "PfmArgs_bool_exp_bool_exp"
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
                  "name": "PfmArgs_bool_exp_bool_exp"
                }
              }
            }
          },
          {
            "name": "channel",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "memo",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "PfmArgs_enum_name",
        "enumValues": [
          {
            "name": "channel",
            "isDeprecated": false
          },
          {
            "name": "memo",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "PfmArgs_order_by",
        "inputFields": [
          {
            "name": "channel",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "memo",
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
        "name": "Request",
        "fields": [
          {
            "name": "address",
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
            "name": "time",
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
            "name": "txHash",
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
        "name": "date"
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "date_comparison_exp",
        "inputFields": [
          {
            "name": "_eq",
            "type": {
              "kind": "SCALAR",
              "name": "date"
            }
          },
          {
            "name": "_gt",
            "type": {
              "kind": "SCALAR",
              "name": "date"
            }
          },
          {
            "name": "_gte",
            "type": {
              "kind": "SCALAR",
              "name": "date"
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
                  "name": "date"
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
              "name": "date"
            }
          },
          {
            "name": "_lte",
            "type": {
              "kind": "SCALAR",
              "name": "date"
            }
          },
          {
            "name": "_neq",
            "type": {
              "kind": "SCALAR",
              "name": "date"
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
                  "name": "date"
                }
              }
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "dydx_faucetMutation",
        "fields": [
          {
            "name": "send",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "String"
              }
            },
            "args": [
              {
                "name": "captchaToken",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "String"
                  }
                }
              },
              {
                "name": "toAddress",
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
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "dydx_faucetQuery",
        "fields": [
          {
            "name": "handledTransfers",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "Request"
                  }
                }
              }
            },
            "args": [
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offsetTime",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "transfersForAddress",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "Request"
                  }
                }
              }
            },
            "args": [
              {
                "name": "address",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "String"
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
                "name": "offsetTime",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "unhandledTransfers",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "Request"
                  }
                }
              }
            },
            "args": [
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offsetTime",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
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
        "name": "faucet2Mutation",
        "fields": [
          {
            "name": "send",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "String"
              }
            },
            "args": [
              {
                "name": "captchaToken",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "String"
                  }
                }
              },
              {
                "name": "toAddress",
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
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "faucet2Query",
        "fields": [
          {
            "name": "handledTransfers",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "Request"
                  }
                }
              }
            },
            "args": [
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offsetTime",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "transfersForAddress",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "Request"
                  }
                }
              }
            },
            "args": [
              {
                "name": "address",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "String"
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
                "name": "offsetTime",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "unhandledTransfers",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "Request"
                  }
                }
              }
            },
            "args": [
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offsetTime",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "get_route_arguments",
        "inputFields": [
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "String"
              }
            }
          },
          {
            "name": "forward_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "receiver",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "String"
              }
            }
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "String"
              }
            }
          }
        ],
        "isOneOf": false
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
        "kind": "OBJECT",
        "name": "mutation_root",
        "fields": [
          {
            "name": "dydx_faucet",
            "type": {
              "kind": "OBJECT",
              "name": "dydx_faucetMutation"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "faucet2",
            "type": {
              "kind": "OBJECT",
              "name": "faucet2Mutation"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stride_faucet",
            "type": {
              "kind": "OBJECT",
              "name": "stride_faucetMutation"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
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
            "name": "dydx_faucet",
            "type": {
              "kind": "OBJECT",
              "name": "dydx_faucetQuery"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "faucet2",
            "type": {
              "kind": "OBJECT",
              "name": "faucet2Query"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "get_route",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "PfmArgs"
                  }
                }
              }
            },
            "args": [
              {
                "name": "args",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "INPUT_OBJECT",
                    "name": "get_route_arguments"
                  }
                }
              },
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "PfmArgs_enum_name"
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
                      "name": "PfmArgs_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "PfmArgs_bool_exp_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "stride_faucet",
            "type": {
              "kind": "OBJECT",
              "name": "stride_faucetQuery"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "v1_acknowledge_packet",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_acknowledge_packet"
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
                      "name": "v1_acknowledge_packet_select_column"
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
                      "name": "v1_acknowledge_packet_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_acknowledge_packet_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_assets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_assets"
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
                      "name": "v1_assets_select_column"
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
                      "name": "v1_assets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_assets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_chains",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_chains"
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
                      "name": "v1_chains_select_column"
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
                      "name": "v1_chains_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_chains_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_channel_open_ack",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_channel_open_ack"
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
                      "name": "v1_channel_open_ack_select_column"
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
                      "name": "v1_channel_open_ack_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_channel_open_ack_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_channel_open_confirm",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_channel_open_confirm"
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
                      "name": "v1_channel_open_confirm_select_column"
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
                      "name": "v1_channel_open_confirm_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_channel_open_confirm_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_channel_open_init",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_channel_open_init"
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
                      "name": "v1_channel_open_init_select_column"
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
                      "name": "v1_channel_open_init_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_channel_open_init_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_channel_open_try",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_channel_open_try"
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
                      "name": "v1_channel_open_try_select_column"
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
                      "name": "v1_channel_open_try_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_channel_open_try_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_channels",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_channels"
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
                      "name": "v1_channels_select_column"
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
                      "name": "v1_channels_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_channels_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_clients",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_clients"
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
                      "name": "v1_clients_select_column"
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
                      "name": "v1_clients_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_clients_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_config",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_config"
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
                      "name": "v1_config_select_column"
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
                      "name": "v1_config_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_config_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_connection_open_ack",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_connection_open_ack"
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
                      "name": "v1_connection_open_ack_select_column"
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
                      "name": "v1_connection_open_ack_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_connection_open_ack_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_connection_open_confirm",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_connection_open_confirm"
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
                      "name": "v1_connection_open_confirm_select_column"
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
                      "name": "v1_connection_open_confirm_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_connection_open_confirm_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_connection_open_init",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_connection_open_init"
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
                      "name": "v1_connection_open_init_select_column"
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
                      "name": "v1_connection_open_init_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_connection_open_init_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_connection_open_try",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_connection_open_try"
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
                      "name": "v1_connection_open_try_select_column"
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
                      "name": "v1_connection_open_try_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_connection_open_try_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_connections",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_connections"
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
                      "name": "v1_connections_select_column"
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
                      "name": "v1_connections_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_connections_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_contracts",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_contracts"
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
                      "name": "v1_contracts_select_column"
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
                      "name": "v1_contracts_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_contracts_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_daily_packets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_daily_packets"
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
                      "name": "v1_daily_packets_select_column"
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
                      "name": "v1_daily_packets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_daily_packets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_daily_transfers",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_daily_transfers"
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
                      "name": "v1_daily_transfers_select_column"
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
                      "name": "v1_daily_transfers_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_daily_transfers_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_explorers",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_explorers"
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
                      "name": "v1_explorers_select_column"
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
                      "name": "v1_explorers_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_explorers_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_faucets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_faucets"
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
                      "name": "v1_faucets_select_column"
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
                      "name": "v1_faucets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_faucets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_health_check",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_health_check"
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
                      "name": "v1_health_check_select_column"
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
                      "name": "v1_health_check_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_health_check_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_assets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_assets"
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
                      "name": "v1_ibc_union_assets_select_column"
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
                      "name": "v1_ibc_union_assets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_assets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_chains",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_chains"
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
                      "name": "v1_ibc_union_chains_select_column"
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
                      "name": "v1_ibc_union_chains_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_chains_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_channels",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_channels"
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
                      "name": "v1_ibc_union_channels_select_column"
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
                      "name": "v1_ibc_union_channels_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_channels_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_clients",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_clients"
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
                      "name": "v1_ibc_union_clients_select_column"
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
                      "name": "v1_ibc_union_clients_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_clients_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_config",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_config"
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
                      "name": "v1_ibc_union_config_select_column"
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
                      "name": "v1_ibc_union_config_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_config_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_connections",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_connections"
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
                      "name": "v1_ibc_union_connections_select_column"
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
                      "name": "v1_ibc_union_connections_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_connections_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_contracts",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_contracts"
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
                      "name": "v1_ibc_union_contracts_select_column"
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
                      "name": "v1_ibc_union_contracts_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_contracts_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_daily_fungible_asset_orders",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_daily_fungible_asset_orders"
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
                      "name": "v1_ibc_union_daily_fungible_asset_orders_select_column"
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
                      "name": "v1_ibc_union_daily_fungible_asset_orders_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_daily_fungible_asset_orders_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_daily_packets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_daily_packets"
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
                      "name": "v1_ibc_union_daily_packets_select_column"
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
                      "name": "v1_ibc_union_daily_packets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_daily_packets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_explorers",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_explorers"
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
                      "name": "v1_ibc_union_explorers_select_column"
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
                      "name": "v1_ibc_union_explorers_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_explorers_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_faucets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_faucets"
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
                      "name": "v1_ibc_union_faucets_select_column"
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
                      "name": "v1_ibc_union_faucets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_faucets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_fungible_asset_orders",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_fungible_asset_orders"
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
                      "name": "v1_ibc_union_fungible_asset_orders_select_column"
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
                      "name": "v1_ibc_union_fungible_asset_orders_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_fungible_asset_orders_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_health_check",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_health_check"
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
                      "name": "v1_ibc_union_health_check_select_column"
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
                      "name": "v1_ibc_union_health_check_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_health_check_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_packets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_packets"
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
                      "name": "v1_ibc_union_packets_select_column"
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
                      "name": "v1_ibc_union_packets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_packets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_rpcs",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_rpcs"
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
                      "name": "v1_ibc_union_rpcs_select_column"
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
                      "name": "v1_ibc_union_rpcs_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_rpcs_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_statistics",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_statistics"
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
                      "name": "v1_ibc_union_statistics_select_column"
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
                      "name": "v1_ibc_union_statistics_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_statistics_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_traces",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_traces"
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
                      "name": "v1_ibc_union_traces_select_column"
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
                      "name": "v1_ibc_union_traces_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_traces_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_index_status",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_index_status"
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
                      "name": "v1_index_status_select_column"
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
                      "name": "v1_index_status_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_index_status_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_lightclient_update",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_lightclient_update"
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
                      "name": "v1_lightclient_update_select_column"
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
                      "name": "v1_lightclient_update_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_lightclient_update_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_packet_effect_type",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_packet_effect_type"
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
                      "name": "v1_packet_effect_type_select_column"
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
                      "name": "v1_packet_effect_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_packet_effect_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_packets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_packets"
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
                      "name": "v1_packets_select_column"
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
                      "name": "v1_packets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_packets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_rpcs",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_rpcs"
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
                      "name": "v1_rpcs_select_column"
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
                      "name": "v1_rpcs_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_rpcs_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_statistics",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_statistics"
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
                      "name": "v1_statistics_select_column"
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
                      "name": "v1_statistics_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_statistics_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_statistics_address",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_statistics_address"
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
                      "name": "v1_statistics_address_select_column"
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
                      "name": "v1_statistics_address_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_statistics_address_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_tokens_type",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_tokens_type"
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
                      "name": "v1_tokens_type_select_column"
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
                      "name": "v1_tokens_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_tokens_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_traces",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_traces"
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
                      "name": "v1_traces_select_column"
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
                      "name": "v1_traces_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_traces_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_transfer_forwards_type",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_transfer_forwards_type"
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
                      "name": "v1_transfer_forwards_type_select_column"
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
                      "name": "v1_transfer_forwards_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_transfer_forwards_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_transfers",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_transfers"
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
                      "name": "v1_transfers_select_column"
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
                      "name": "v1_transfers_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_transfers_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ucs1_configurations",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ucs1_configurations"
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
                      "name": "v1_ucs1_configurations_select_column"
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
                      "name": "v1_ucs1_configurations_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ucs1_configurations_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ucs1_paths",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ucs1_paths"
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
                      "name": "v1_ucs1_paths_select_column"
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
                      "name": "v1_ucs1_paths_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ucs1_paths_bool_exp"
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
        "name": "stride_faucetMutation",
        "fields": [
          {
            "name": "send",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "String"
              }
            },
            "args": [
              {
                "name": "captchaToken",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "String"
                  }
                }
              },
              {
                "name": "toAddress",
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
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "stride_faucetQuery",
        "fields": [
          {
            "name": "handledTransfers",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "Request"
                  }
                }
              }
            },
            "args": [
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offsetTime",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "transfersForAddress",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "Request"
                  }
                }
              }
            },
            "args": [
              {
                "name": "address",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "String"
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
                "name": "offsetTime",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "unhandledTransfers",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "Request"
                  }
                }
              }
            },
            "args": [
              {
                "name": "limit",
                "type": {
                  "kind": "SCALAR",
                  "name": "Int"
                }
              },
              {
                "name": "offsetTime",
                "type": {
                  "kind": "SCALAR",
                  "name": "String"
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
        "name": "subscription_root",
        "fields": [
          {
            "name": "get_route",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "PfmArgs"
                  }
                }
              }
            },
            "args": [
              {
                "name": "args",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "INPUT_OBJECT",
                    "name": "get_route_arguments"
                  }
                }
              },
              {
                "name": "distinct_on",
                "type": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "ENUM",
                      "name": "PfmArgs_enum_name"
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
                      "name": "PfmArgs_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "PfmArgs_bool_exp_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_acknowledge_packet",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_acknowledge_packet"
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
                      "name": "v1_acknowledge_packet_select_column"
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
                      "name": "v1_acknowledge_packet_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_acknowledge_packet_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_acknowledge_packet_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_acknowledge_packet"
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
                      "name": "v1_acknowledge_packet_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_acknowledge_packet_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_assets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_assets"
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
                      "name": "v1_assets_select_column"
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
                      "name": "v1_assets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_assets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_assets_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_assets"
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
                      "name": "v1_assets_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_assets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_chains",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_chains"
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
                      "name": "v1_chains_select_column"
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
                      "name": "v1_chains_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_chains_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_chains_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_chains"
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
                      "name": "v1_chains_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_chains_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_channel_open_ack",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_channel_open_ack"
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
                      "name": "v1_channel_open_ack_select_column"
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
                      "name": "v1_channel_open_ack_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_channel_open_ack_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_channel_open_ack_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_channel_open_ack"
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
                      "name": "v1_channel_open_ack_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_channel_open_ack_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_channel_open_confirm",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_channel_open_confirm"
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
                      "name": "v1_channel_open_confirm_select_column"
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
                      "name": "v1_channel_open_confirm_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_channel_open_confirm_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_channel_open_confirm_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_channel_open_confirm"
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
                      "name": "v1_channel_open_confirm_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_channel_open_confirm_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_channel_open_init",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_channel_open_init"
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
                      "name": "v1_channel_open_init_select_column"
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
                      "name": "v1_channel_open_init_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_channel_open_init_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_channel_open_init_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_channel_open_init"
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
                      "name": "v1_channel_open_init_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_channel_open_init_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_channel_open_try",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_channel_open_try"
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
                      "name": "v1_channel_open_try_select_column"
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
                      "name": "v1_channel_open_try_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_channel_open_try_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_channel_open_try_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_channel_open_try"
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
                      "name": "v1_channel_open_try_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_channel_open_try_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_channels",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_channels"
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
                      "name": "v1_channels_select_column"
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
                      "name": "v1_channels_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_channels_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_channels_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_channels"
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
                      "name": "v1_channels_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_channels_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_clients",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_clients"
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
                      "name": "v1_clients_select_column"
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
                      "name": "v1_clients_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_clients_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_clients_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_clients"
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
                      "name": "v1_clients_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_clients_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_config",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_config"
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
                      "name": "v1_config_select_column"
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
                      "name": "v1_config_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_config_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_config_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_config"
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
                      "name": "v1_config_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_config_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_connection_open_ack",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_connection_open_ack"
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
                      "name": "v1_connection_open_ack_select_column"
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
                      "name": "v1_connection_open_ack_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_connection_open_ack_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_connection_open_ack_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_connection_open_ack"
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
                      "name": "v1_connection_open_ack_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_connection_open_ack_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_connection_open_confirm",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_connection_open_confirm"
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
                      "name": "v1_connection_open_confirm_select_column"
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
                      "name": "v1_connection_open_confirm_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_connection_open_confirm_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_connection_open_confirm_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_connection_open_confirm"
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
                      "name": "v1_connection_open_confirm_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_connection_open_confirm_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_connection_open_init",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_connection_open_init"
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
                      "name": "v1_connection_open_init_select_column"
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
                      "name": "v1_connection_open_init_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_connection_open_init_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_connection_open_init_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_connection_open_init"
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
                      "name": "v1_connection_open_init_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_connection_open_init_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_connection_open_try",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_connection_open_try"
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
                      "name": "v1_connection_open_try_select_column"
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
                      "name": "v1_connection_open_try_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_connection_open_try_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_connection_open_try_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_connection_open_try"
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
                      "name": "v1_connection_open_try_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_connection_open_try_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_connections",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_connections"
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
                      "name": "v1_connections_select_column"
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
                      "name": "v1_connections_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_connections_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_connections_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_connections"
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
                      "name": "v1_connections_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_connections_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_contracts",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_contracts"
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
                      "name": "v1_contracts_select_column"
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
                      "name": "v1_contracts_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_contracts_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_contracts_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_contracts"
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
                      "name": "v1_contracts_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_contracts_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_daily_packets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_daily_packets"
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
                      "name": "v1_daily_packets_select_column"
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
                      "name": "v1_daily_packets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_daily_packets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_daily_packets_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_daily_packets"
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
                      "name": "v1_daily_packets_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_daily_packets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_daily_transfers",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_daily_transfers"
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
                      "name": "v1_daily_transfers_select_column"
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
                      "name": "v1_daily_transfers_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_daily_transfers_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_daily_transfers_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_daily_transfers"
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
                      "name": "v1_daily_transfers_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_daily_transfers_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_explorers",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_explorers"
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
                      "name": "v1_explorers_select_column"
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
                      "name": "v1_explorers_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_explorers_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_explorers_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_explorers"
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
                      "name": "v1_explorers_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_explorers_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_faucets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_faucets"
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
                      "name": "v1_faucets_select_column"
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
                      "name": "v1_faucets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_faucets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_faucets_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_faucets"
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
                      "name": "v1_faucets_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_faucets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_health_check",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_health_check"
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
                      "name": "v1_health_check_select_column"
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
                      "name": "v1_health_check_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_health_check_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_health_check_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_health_check"
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
                      "name": "v1_health_check_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_health_check_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_assets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_assets"
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
                      "name": "v1_ibc_union_assets_select_column"
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
                      "name": "v1_ibc_union_assets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_assets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_assets_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_assets"
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
                      "name": "v1_ibc_union_assets_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_assets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_chains",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_chains"
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
                      "name": "v1_ibc_union_chains_select_column"
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
                      "name": "v1_ibc_union_chains_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_chains_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_chains_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_chains"
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
                      "name": "v1_ibc_union_chains_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_chains_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_channels",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_channels"
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
                      "name": "v1_ibc_union_channels_select_column"
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
                      "name": "v1_ibc_union_channels_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_channels_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_channels_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_channels"
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
                      "name": "v1_ibc_union_channels_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_channels_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_clients",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_clients"
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
                      "name": "v1_ibc_union_clients_select_column"
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
                      "name": "v1_ibc_union_clients_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_clients_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_clients_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_clients"
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
                      "name": "v1_ibc_union_clients_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_clients_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_config",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_config"
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
                      "name": "v1_ibc_union_config_select_column"
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
                      "name": "v1_ibc_union_config_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_config_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_config_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_config"
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
                      "name": "v1_ibc_union_config_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_config_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_connections",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_connections"
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
                      "name": "v1_ibc_union_connections_select_column"
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
                      "name": "v1_ibc_union_connections_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_connections_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_connections_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_connections"
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
                      "name": "v1_ibc_union_connections_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_connections_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_contracts",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_contracts"
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
                      "name": "v1_ibc_union_contracts_select_column"
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
                      "name": "v1_ibc_union_contracts_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_contracts_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_contracts_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_contracts"
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
                      "name": "v1_ibc_union_contracts_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_contracts_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_daily_fungible_asset_orders",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_daily_fungible_asset_orders"
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
                      "name": "v1_ibc_union_daily_fungible_asset_orders_select_column"
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
                      "name": "v1_ibc_union_daily_fungible_asset_orders_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_daily_fungible_asset_orders_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_daily_fungible_asset_orders_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_daily_fungible_asset_orders"
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
                      "name": "v1_ibc_union_daily_fungible_asset_orders_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_daily_fungible_asset_orders_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_daily_packets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_daily_packets"
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
                      "name": "v1_ibc_union_daily_packets_select_column"
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
                      "name": "v1_ibc_union_daily_packets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_daily_packets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_daily_packets_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_daily_packets"
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
                      "name": "v1_ibc_union_daily_packets_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_daily_packets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_explorers",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_explorers"
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
                      "name": "v1_ibc_union_explorers_select_column"
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
                      "name": "v1_ibc_union_explorers_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_explorers_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_explorers_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_explorers"
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
                      "name": "v1_ibc_union_explorers_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_explorers_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_faucets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_faucets"
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
                      "name": "v1_ibc_union_faucets_select_column"
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
                      "name": "v1_ibc_union_faucets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_faucets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_faucets_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_faucets"
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
                      "name": "v1_ibc_union_faucets_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_faucets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_fungible_asset_orders",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_fungible_asset_orders"
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
                      "name": "v1_ibc_union_fungible_asset_orders_select_column"
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
                      "name": "v1_ibc_union_fungible_asset_orders_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_fungible_asset_orders_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_fungible_asset_orders_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_fungible_asset_orders"
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
                      "name": "v1_ibc_union_fungible_asset_orders_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_fungible_asset_orders_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_health_check",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_health_check"
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
                      "name": "v1_ibc_union_health_check_select_column"
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
                      "name": "v1_ibc_union_health_check_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_health_check_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_health_check_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_health_check"
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
                      "name": "v1_ibc_union_health_check_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_health_check_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_packets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_packets"
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
                      "name": "v1_ibc_union_packets_select_column"
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
                      "name": "v1_ibc_union_packets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_packets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_packets_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_packets"
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
                      "name": "v1_ibc_union_packets_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_packets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_rpcs",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_rpcs"
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
                      "name": "v1_ibc_union_rpcs_select_column"
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
                      "name": "v1_ibc_union_rpcs_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_rpcs_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_rpcs_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_rpcs"
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
                      "name": "v1_ibc_union_rpcs_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_rpcs_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_statistics",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_statistics"
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
                      "name": "v1_ibc_union_statistics_select_column"
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
                      "name": "v1_ibc_union_statistics_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_statistics_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_statistics_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_statistics"
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
                      "name": "v1_ibc_union_statistics_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_statistics_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_traces",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_traces"
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
                      "name": "v1_ibc_union_traces_select_column"
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
                      "name": "v1_ibc_union_traces_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_traces_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ibc_union_traces_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_traces"
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
                      "name": "v1_ibc_union_traces_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_traces_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_index_status",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_index_status"
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
                      "name": "v1_index_status_select_column"
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
                      "name": "v1_index_status_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_index_status_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_index_status_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_index_status"
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
                      "name": "v1_index_status_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_index_status_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_lightclient_update",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_lightclient_update"
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
                      "name": "v1_lightclient_update_select_column"
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
                      "name": "v1_lightclient_update_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_lightclient_update_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_lightclient_update_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_lightclient_update"
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
                      "name": "v1_lightclient_update_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_lightclient_update_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_packet_effect_type",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_packet_effect_type"
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
                      "name": "v1_packet_effect_type_select_column"
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
                      "name": "v1_packet_effect_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_packet_effect_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_packet_effect_type_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_packet_effect_type"
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
                      "name": "v1_packet_effect_type_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_packet_effect_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_packets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_packets"
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
                      "name": "v1_packets_select_column"
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
                      "name": "v1_packets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_packets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_packets_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_packets"
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
                      "name": "v1_packets_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_packets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_rpcs",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_rpcs"
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
                      "name": "v1_rpcs_select_column"
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
                      "name": "v1_rpcs_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_rpcs_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_rpcs_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_rpcs"
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
                      "name": "v1_rpcs_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_rpcs_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_statistics",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_statistics"
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
                      "name": "v1_statistics_select_column"
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
                      "name": "v1_statistics_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_statistics_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_statistics_address",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_statistics_address"
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
                      "name": "v1_statistics_address_select_column"
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
                      "name": "v1_statistics_address_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_statistics_address_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_statistics_address_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_statistics_address"
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
                      "name": "v1_statistics_address_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_statistics_address_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_statistics_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_statistics"
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
                      "name": "v1_statistics_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_statistics_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_tokens_type",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_tokens_type"
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
                      "name": "v1_tokens_type_select_column"
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
                      "name": "v1_tokens_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_tokens_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_tokens_type_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_tokens_type"
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
                      "name": "v1_tokens_type_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_tokens_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_traces",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_traces"
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
                      "name": "v1_traces_select_column"
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
                      "name": "v1_traces_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_traces_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_traces_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_traces"
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
                      "name": "v1_traces_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_traces_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_transfer_forwards_type",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_transfer_forwards_type"
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
                      "name": "v1_transfer_forwards_type_select_column"
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
                      "name": "v1_transfer_forwards_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_transfer_forwards_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_transfer_forwards_type_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_transfer_forwards_type"
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
                      "name": "v1_transfer_forwards_type_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_transfer_forwards_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_transfers",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_transfers"
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
                      "name": "v1_transfers_select_column"
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
                      "name": "v1_transfers_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_transfers_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_transfers_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_transfers"
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
                      "name": "v1_transfers_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_transfers_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ucs1_configurations",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ucs1_configurations"
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
                      "name": "v1_ucs1_configurations_select_column"
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
                      "name": "v1_ucs1_configurations_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ucs1_configurations_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ucs1_configurations_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ucs1_configurations"
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
                      "name": "v1_ucs1_configurations_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ucs1_configurations_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ucs1_paths",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ucs1_paths"
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
                      "name": "v1_ucs1_paths_select_column"
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
                      "name": "v1_ucs1_paths_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ucs1_paths_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v1_ucs1_paths_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ucs1_paths"
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
                      "name": "v1_ucs1_paths_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ucs1_paths_bool_exp"
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
        "name": "v1_acknowledge_packet",
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
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "event_index",
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
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sequence",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_port_id",
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
            "name": "transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
        "name": "v1_acknowledge_packet_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_acknowledge_packet_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_acknowledge_packet_bool_exp"
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
                  "name": "v1_acknowledge_packet_bool_exp"
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
              "name": "v1_chains_bool_exp"
            }
          },
          {
            "name": "event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "sequence",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "source_channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_port_id",
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
            "name": "transaction_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
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
        "name": "v1_acknowledge_packet_order_by",
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
              "name": "v1_chains_order_by"
            }
          },
          {
            "name": "event_index",
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
            "name": "sequence",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_port_id",
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
            "name": "transaction_event_index",
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
        "name": "v1_acknowledge_packet_select_column",
        "enumValues": [
          {
            "name": "block_hash",
            "isDeprecated": false
          },
          {
            "name": "event_index",
            "isDeprecated": false
          },
          {
            "name": "height",
            "isDeprecated": false
          },
          {
            "name": "sequence",
            "isDeprecated": false
          },
          {
            "name": "source_channel_id",
            "isDeprecated": false
          },
          {
            "name": "source_port_id",
            "isDeprecated": false
          },
          {
            "name": "timestamp",
            "isDeprecated": false
          },
          {
            "name": "transaction_event_index",
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
        "name": "v1_acknowledge_packet_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_acknowledge_packet_stream_cursor_value_input"
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
        "name": "v1_acknowledge_packet_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "sequence",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "source_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_port_id",
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
            "name": "transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
        "name": "v1_assets",
        "fields": [
          {
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "decimals",
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
            "name": "display_name",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "display_symbol",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "faucets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_faucets"
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
                      "name": "v1_faucets_select_column"
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
                      "name": "v1_faucets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_faucets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "gas_token",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "logo_uri",
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
        "name": "v1_assets_aggregate_order_by",
        "inputFields": [
          {
            "name": "avg",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_assets_avg_order_by"
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
              "name": "v1_assets_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_assets_min_order_by"
            }
          },
          {
            "name": "stddev",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_assets_stddev_order_by"
            }
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_assets_stddev_pop_order_by"
            }
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_assets_stddev_samp_order_by"
            }
          },
          {
            "name": "sum",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_assets_sum_order_by"
            }
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_assets_var_pop_order_by"
            }
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_assets_var_samp_order_by"
            }
          },
          {
            "name": "variance",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_assets_variance_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_assets_avg_order_by",
        "inputFields": [
          {
            "name": "decimals",
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
        "name": "v1_assets_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_assets_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_assets_bool_exp"
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
                  "name": "v1_assets_bool_exp"
                }
              }
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_bool_exp"
            }
          },
          {
            "name": "decimals",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
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
            "name": "display_name",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "display_symbol",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "faucets",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_faucets_bool_exp"
            }
          },
          {
            "name": "gas_token",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Boolean_comparison_exp"
            }
          },
          {
            "name": "logo_uri",
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
        "name": "v1_assets_max_order_by",
        "inputFields": [
          {
            "name": "decimals",
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
            "name": "display_name",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "display_symbol",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "logo_uri",
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
        "name": "v1_assets_min_order_by",
        "inputFields": [
          {
            "name": "decimals",
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
            "name": "display_name",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "display_symbol",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "logo_uri",
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
        "name": "v1_assets_order_by",
        "inputFields": [
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_order_by"
            }
          },
          {
            "name": "decimals",
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
            "name": "display_name",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "display_symbol",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "faucets_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_faucets_aggregate_order_by"
            }
          },
          {
            "name": "gas_token",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "logo_uri",
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
        "name": "v1_assets_select_column",
        "enumValues": [
          {
            "name": "decimals",
            "isDeprecated": false
          },
          {
            "name": "denom",
            "isDeprecated": false
          },
          {
            "name": "display_name",
            "isDeprecated": false
          },
          {
            "name": "display_symbol",
            "isDeprecated": false
          },
          {
            "name": "gas_token",
            "isDeprecated": false
          },
          {
            "name": "logo_uri",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_assets_stddev_order_by",
        "inputFields": [
          {
            "name": "decimals",
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
        "name": "v1_assets_stddev_pop_order_by",
        "inputFields": [
          {
            "name": "decimals",
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
        "name": "v1_assets_stddev_samp_order_by",
        "inputFields": [
          {
            "name": "decimals",
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
        "name": "v1_assets_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_assets_stream_cursor_value_input"
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
        "name": "v1_assets_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "decimals",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
            "name": "display_name",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "display_symbol",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "gas_token",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "logo_uri",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_assets_sum_order_by",
        "inputFields": [
          {
            "name": "decimals",
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
        "name": "v1_assets_var_pop_order_by",
        "inputFields": [
          {
            "name": "decimals",
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
        "name": "v1_assets_var_samp_order_by",
        "inputFields": [
          {
            "name": "decimals",
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
        "name": "v1_assets_variance_order_by",
        "inputFields": [
          {
            "name": "decimals",
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
        "name": "v1_chains",
        "fields": [
          {
            "name": "addr_prefix",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "assets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_assets"
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
                      "name": "v1_assets_select_column"
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
                      "name": "v1_assets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_assets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
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
            "name": "enabled",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "enabled_staging",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "explorers",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_explorers"
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
                      "name": "v1_explorers_select_column"
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
                      "name": "v1_explorers_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_explorers_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "logo_uri",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "rpc_type",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "rpcs",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_rpcs"
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
                      "name": "v1_rpcs_select_column"
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
                      "name": "v1_rpcs_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_rpcs_bool_exp"
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
          },
          {
            "name": "ucs1_configurations",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ucs1_configurations"
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
                      "name": "v1_ucs1_configurations_select_column"
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
                      "name": "v1_ucs1_configurations_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ucs1_configurations_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_chains_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_chains_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_bool_exp"
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
                  "name": "v1_chains_bool_exp"
                }
              }
            }
          },
          {
            "name": "addr_prefix",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "assets",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_assets_bool_exp"
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
            "name": "enabled",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Boolean_comparison_exp"
            }
          },
          {
            "name": "enabled_staging",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Boolean_comparison_exp"
            }
          },
          {
            "name": "explorers",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_explorers_bool_exp"
            }
          },
          {
            "name": "logo_uri",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "rpc_type",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "rpcs",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_rpcs_bool_exp"
            }
          },
          {
            "name": "testnet",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Boolean_comparison_exp"
            }
          },
          {
            "name": "ucs1_configurations",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ucs1_configurations_bool_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_chains_order_by",
        "inputFields": [
          {
            "name": "addr_prefix",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "assets_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_assets_aggregate_order_by"
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
            "name": "enabled",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "enabled_staging",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "explorers_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_explorers_aggregate_order_by"
            }
          },
          {
            "name": "logo_uri",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "rpc_type",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "rpcs_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_rpcs_aggregate_order_by"
            }
          },
          {
            "name": "testnet",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "ucs1_configurations_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ucs1_configurations_aggregate_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v1_chains_select_column",
        "enumValues": [
          {
            "name": "addr_prefix",
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "isDeprecated": false
          },
          {
            "name": "display_name",
            "isDeprecated": false
          },
          {
            "name": "enabled",
            "isDeprecated": false
          },
          {
            "name": "enabled_staging",
            "isDeprecated": false
          },
          {
            "name": "logo_uri",
            "isDeprecated": false
          },
          {
            "name": "rpc_type",
            "isDeprecated": false
          },
          {
            "name": "testnet",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_chains_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_chains_stream_cursor_value_input"
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
        "name": "v1_chains_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "addr_prefix",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
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
            "name": "enabled",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "enabled_staging",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "logo_uri",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "rpc_type",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "testnet",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v1_channel_open_ack",
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
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
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
              "name": "bigint"
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
            "name": "timestamp",
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
        "name": "v1_channel_open_ack_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_channel_open_ack_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_channel_open_ack_bool_exp"
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
                  "name": "v1_channel_open_ack_bool_exp"
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
              "name": "v1_chains_bool_exp"
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
            "name": "height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
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
            "name": "timestamp",
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
        "name": "v1_channel_open_ack_order_by",
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
              "name": "v1_chains_order_by"
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
            "name": "height",
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
            "name": "timestamp",
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
        "name": "v1_channel_open_ack_select_column",
        "enumValues": [
          {
            "name": "block_hash",
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
            "name": "height",
            "isDeprecated": false
          },
          {
            "name": "port_id",
            "isDeprecated": false
          },
          {
            "name": "timestamp",
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
        "name": "v1_channel_open_ack_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_channel_open_ack_stream_cursor_value_input"
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
        "name": "v1_channel_open_ack_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
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
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
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
            "name": "timestamp",
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
        "name": "v1_channel_open_confirm",
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
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
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
              "name": "bigint"
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
            "name": "timestamp",
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
        "name": "v1_channel_open_confirm_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_channel_open_confirm_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_channel_open_confirm_bool_exp"
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
                  "name": "v1_channel_open_confirm_bool_exp"
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
              "name": "v1_chains_bool_exp"
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
            "name": "height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
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
            "name": "timestamp",
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
        "name": "v1_channel_open_confirm_order_by",
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
              "name": "v1_chains_order_by"
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
            "name": "height",
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
            "name": "timestamp",
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
        "name": "v1_channel_open_confirm_select_column",
        "enumValues": [
          {
            "name": "block_hash",
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
            "name": "height",
            "isDeprecated": false
          },
          {
            "name": "port_id",
            "isDeprecated": false
          },
          {
            "name": "timestamp",
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
        "name": "v1_channel_open_confirm_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_channel_open_confirm_stream_cursor_value_input"
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
        "name": "v1_channel_open_confirm_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
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
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
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
            "name": "timestamp",
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
        "name": "v1_channel_open_init",
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
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
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
              "name": "bigint"
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
            "name": "timestamp",
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
        "name": "v1_channel_open_init_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_channel_open_init_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_channel_open_init_bool_exp"
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
                  "name": "v1_channel_open_init_bool_exp"
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
              "name": "v1_chains_bool_exp"
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
            "name": "height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
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
            "name": "timestamp",
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
        "kind": "INPUT_OBJECT",
        "name": "v1_channel_open_init_order_by",
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
              "name": "v1_chains_order_by"
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
            "name": "height",
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
            "name": "timestamp",
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
        "name": "v1_channel_open_init_select_column",
        "enumValues": [
          {
            "name": "block_hash",
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
            "name": "height",
            "isDeprecated": false
          },
          {
            "name": "port_id",
            "isDeprecated": false
          },
          {
            "name": "timestamp",
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
        "kind": "INPUT_OBJECT",
        "name": "v1_channel_open_init_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_channel_open_init_stream_cursor_value_input"
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
        "name": "v1_channel_open_init_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
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
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
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
            "name": "timestamp",
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
        "name": "v1_channel_open_try",
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
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
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
              "name": "bigint"
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
            "name": "timestamp",
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
        "name": "v1_channel_open_try_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_channel_open_try_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_channel_open_try_bool_exp"
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
                  "name": "v1_channel_open_try_bool_exp"
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
              "name": "v1_chains_bool_exp"
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
            "name": "height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
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
            "name": "timestamp",
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
        "kind": "INPUT_OBJECT",
        "name": "v1_channel_open_try_order_by",
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
              "name": "v1_chains_order_by"
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
            "name": "height",
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
            "name": "timestamp",
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
        "name": "v1_channel_open_try_select_column",
        "enumValues": [
          {
            "name": "block_hash",
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
            "name": "height",
            "isDeprecated": false
          },
          {
            "name": "port_id",
            "isDeprecated": false
          },
          {
            "name": "timestamp",
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
        "kind": "INPUT_OBJECT",
        "name": "v1_channel_open_try_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_channel_open_try_stream_cursor_value_input"
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
        "name": "v1_channel_open_try_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
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
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
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
            "name": "timestamp",
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
        "name": "v1_channels",
        "fields": [
          {
            "name": "connection",
            "type": {
              "kind": "OBJECT",
              "name": "v1_connections"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_port_id",
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
        "name": "v1_channels_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_channels_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_channels_bool_exp"
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
                  "name": "v1_channels_bool_exp"
                }
              }
            }
          },
          {
            "name": "connection",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_connections_bool_exp"
            }
          },
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_bool_exp"
            }
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_port_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_bool_exp"
            }
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_port_id",
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
        "kind": "INPUT_OBJECT",
        "name": "v1_channels_order_by",
        "inputFields": [
          {
            "name": "connection",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_connections_order_by"
            }
          },
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_order_by"
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
            "name": "destination_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_port_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_order_by"
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
            "name": "source_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_port_id",
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
        "name": "v1_channels_select_column",
        "enumValues": [
          {
            "name": "destination_chain_id",
            "isDeprecated": false
          },
          {
            "name": "destination_channel_id",
            "isDeprecated": false
          },
          {
            "name": "destination_connection_id",
            "isDeprecated": false
          },
          {
            "name": "destination_port_id",
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
            "isDeprecated": false
          },
          {
            "name": "source_channel_id",
            "isDeprecated": false
          },
          {
            "name": "source_connection_id",
            "isDeprecated": false
          },
          {
            "name": "source_port_id",
            "isDeprecated": false
          },
          {
            "name": "status",
            "isDeprecated": false
          },
          {
            "name": "version",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_channels_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_channels_stream_cursor_value_input"
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
        "name": "v1_channels_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_port_id",
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
        "name": "v1_clients",
        "fields": [
          {
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
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
        "name": "v1_clients_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_clients_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_clients_bool_exp"
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
                  "name": "v1_clients_bool_exp"
                }
              }
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_bool_exp"
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
        "kind": "INPUT_OBJECT",
        "name": "v1_clients_order_by",
        "inputFields": [
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_order_by"
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
        "name": "v1_clients_select_column",
        "enumValues": [
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
        "kind": "INPUT_OBJECT",
        "name": "v1_clients_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_clients_stream_cursor_value_input"
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
        "name": "v1_clients_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "client_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "counterparty_chain_id",
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
        "name": "v1_config",
        "fields": [
          {
            "name": "key",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "value",
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
        "name": "v1_config_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_config_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_config_bool_exp"
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
                  "name": "v1_config_bool_exp"
                }
              }
            }
          },
          {
            "name": "key",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "value",
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
        "name": "v1_config_order_by",
        "inputFields": [
          {
            "name": "key",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "value",
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
        "name": "v1_config_select_column",
        "enumValues": [
          {
            "name": "key",
            "isDeprecated": false
          },
          {
            "name": "value",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_config_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_config_stream_cursor_value_input"
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
        "name": "v1_config_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "key",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "value",
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
        "name": "v1_connection_open_ack",
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
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
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
            "name": "connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "counterparty_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "counterparty_connection_id",
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
              "name": "bigint"
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
        "name": "v1_connection_open_ack_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_connection_open_ack_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_connection_open_ack_bool_exp"
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
                  "name": "v1_connection_open_ack_bool_exp"
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
              "name": "v1_chains_bool_exp"
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
            "name": "connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "counterparty_client_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "counterparty_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
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
        "name": "v1_connection_open_ack_order_by",
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
              "name": "v1_chains_order_by"
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
            "name": "connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "counterparty_client_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "counterparty_connection_id",
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
            "name": "timestamp",
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
        "name": "v1_connection_open_ack_select_column",
        "enumValues": [
          {
            "name": "block_hash",
            "isDeprecated": false
          },
          {
            "name": "client_id",
            "isDeprecated": false
          },
          {
            "name": "connection_id",
            "isDeprecated": false
          },
          {
            "name": "counterparty_client_id",
            "isDeprecated": false
          },
          {
            "name": "counterparty_connection_id",
            "isDeprecated": false
          },
          {
            "name": "height",
            "isDeprecated": false
          },
          {
            "name": "timestamp",
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
        "name": "v1_connection_open_ack_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_connection_open_ack_stream_cursor_value_input"
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
        "name": "v1_connection_open_ack_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "client_id",
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
            "name": "counterparty_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "counterparty_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
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
        "name": "v1_connection_open_confirm",
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
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
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
            "name": "connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "counterparty_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "counterparty_connection_id",
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
              "name": "bigint"
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
        "name": "v1_connection_open_confirm_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_connection_open_confirm_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_connection_open_confirm_bool_exp"
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
                  "name": "v1_connection_open_confirm_bool_exp"
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
              "name": "v1_chains_bool_exp"
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
            "name": "connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "counterparty_client_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "counterparty_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
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
        "name": "v1_connection_open_confirm_order_by",
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
              "name": "v1_chains_order_by"
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
            "name": "connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "counterparty_client_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "counterparty_connection_id",
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
            "name": "timestamp",
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
        "name": "v1_connection_open_confirm_select_column",
        "enumValues": [
          {
            "name": "block_hash",
            "isDeprecated": false
          },
          {
            "name": "client_id",
            "isDeprecated": false
          },
          {
            "name": "connection_id",
            "isDeprecated": false
          },
          {
            "name": "counterparty_client_id",
            "isDeprecated": false
          },
          {
            "name": "counterparty_connection_id",
            "isDeprecated": false
          },
          {
            "name": "height",
            "isDeprecated": false
          },
          {
            "name": "timestamp",
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
        "name": "v1_connection_open_confirm_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_connection_open_confirm_stream_cursor_value_input"
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
        "name": "v1_connection_open_confirm_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "client_id",
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
            "name": "counterparty_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "counterparty_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
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
        "name": "v1_connection_open_init",
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
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
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
            "name": "connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "counterparty_client_id",
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
              "name": "bigint"
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
        "name": "v1_connection_open_init_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_connection_open_init_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_connection_open_init_bool_exp"
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
                  "name": "v1_connection_open_init_bool_exp"
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
              "name": "v1_chains_bool_exp"
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
            "name": "connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "counterparty_client_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
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
        "name": "v1_connection_open_init_order_by",
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
              "name": "v1_chains_order_by"
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
            "name": "connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "counterparty_client_id",
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
            "name": "timestamp",
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
        "name": "v1_connection_open_init_select_column",
        "enumValues": [
          {
            "name": "block_hash",
            "isDeprecated": false
          },
          {
            "name": "client_id",
            "isDeprecated": false
          },
          {
            "name": "connection_id",
            "isDeprecated": false
          },
          {
            "name": "counterparty_client_id",
            "isDeprecated": false
          },
          {
            "name": "height",
            "isDeprecated": false
          },
          {
            "name": "timestamp",
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
        "name": "v1_connection_open_init_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_connection_open_init_stream_cursor_value_input"
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
        "name": "v1_connection_open_init_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "client_id",
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
            "name": "counterparty_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
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
        "name": "v1_connection_open_try",
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
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
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
            "name": "connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "counterparty_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "counterparty_connection_id",
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
              "name": "bigint"
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
        "name": "v1_connection_open_try_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_connection_open_try_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_connection_open_try_bool_exp"
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
                  "name": "v1_connection_open_try_bool_exp"
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
              "name": "v1_chains_bool_exp"
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
            "name": "connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "counterparty_client_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "counterparty_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
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
        "name": "v1_connection_open_try_order_by",
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
              "name": "v1_chains_order_by"
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
            "name": "connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "counterparty_client_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "counterparty_connection_id",
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
            "name": "timestamp",
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
        "name": "v1_connection_open_try_select_column",
        "enumValues": [
          {
            "name": "block_hash",
            "isDeprecated": false
          },
          {
            "name": "client_id",
            "isDeprecated": false
          },
          {
            "name": "connection_id",
            "isDeprecated": false
          },
          {
            "name": "counterparty_client_id",
            "isDeprecated": false
          },
          {
            "name": "counterparty_connection_id",
            "isDeprecated": false
          },
          {
            "name": "height",
            "isDeprecated": false
          },
          {
            "name": "timestamp",
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
        "name": "v1_connection_open_try_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_connection_open_try_stream_cursor_value_input"
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
        "name": "v1_connection_open_try_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "client_id",
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
            "name": "counterparty_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "counterparty_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
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
        "name": "v1_connections",
        "fields": [
          {
            "name": "destination_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_connection_id",
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
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_connections_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_connections_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_connections_bool_exp"
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
                  "name": "v1_connections_bool_exp"
                }
              }
            }
          },
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_bool_exp"
            }
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_client_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_bool_exp"
            }
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_client_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_connection_id",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_connections_order_by",
        "inputFields": [
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_order_by"
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
            "name": "destination_client_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_order_by"
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
            "name": "source_client_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_connection_id",
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
        "name": "v1_connections_select_column",
        "enumValues": [
          {
            "name": "destination_chain_id",
            "isDeprecated": false
          },
          {
            "name": "destination_client_id",
            "isDeprecated": false
          },
          {
            "name": "destination_connection_id",
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
            "isDeprecated": false
          },
          {
            "name": "source_client_id",
            "isDeprecated": false
          },
          {
            "name": "source_connection_id",
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
        "name": "v1_connections_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_connections_stream_cursor_value_input"
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
        "name": "v1_connections_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_connection_id",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v1_contracts",
        "fields": [
          {
            "name": "abi",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "address",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "created_at",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "description",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "max_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "min_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "updated_at",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
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
        "name": "v1_contracts_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_contracts_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_contracts_bool_exp"
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
                  "name": "v1_contracts_bool_exp"
                }
              }
            }
          },
          {
            "name": "abi",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "address",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_bool_exp"
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
            "name": "description",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "max_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "min_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "updated_at",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
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
        "kind": "INPUT_OBJECT",
        "name": "v1_contracts_order_by",
        "inputFields": [
          {
            "name": "abi",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "address",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_order_by"
            }
          },
          {
            "name": "created_at",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "description",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "max_height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "min_height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "updated_at",
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
        "name": "v1_contracts_select_column",
        "enumValues": [
          {
            "name": "abi",
            "isDeprecated": false
          },
          {
            "name": "address",
            "isDeprecated": false
          },
          {
            "name": "created_at",
            "isDeprecated": false
          },
          {
            "name": "description",
            "isDeprecated": false
          },
          {
            "name": "max_height",
            "isDeprecated": false
          },
          {
            "name": "min_height",
            "isDeprecated": false
          },
          {
            "name": "updated_at",
            "isDeprecated": false
          },
          {
            "name": "version",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_contracts_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_contracts_stream_cursor_value_input"
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
        "name": "v1_contracts_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "abi",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "address",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "created_at",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "description",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "max_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "min_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "updated_at",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
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
        "name": "v1_daily_packets",
        "fields": [
          {
            "name": "count",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "day",
            "type": {
              "kind": "SCALAR",
              "name": "date"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_daily_packets_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_daily_packets_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_daily_packets_bool_exp"
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
                  "name": "v1_daily_packets_bool_exp"
                }
              }
            }
          },
          {
            "name": "count",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "day",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "date_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_daily_packets_order_by",
        "inputFields": [
          {
            "name": "count",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "day",
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
        "name": "v1_daily_packets_select_column",
        "enumValues": [
          {
            "name": "count",
            "isDeprecated": false
          },
          {
            "name": "day",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_daily_packets_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_daily_packets_stream_cursor_value_input"
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
        "name": "v1_daily_packets_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "count",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "day",
            "type": {
              "kind": "SCALAR",
              "name": "date"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v1_daily_transfers",
        "fields": [
          {
            "name": "count",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "day",
            "type": {
              "kind": "SCALAR",
              "name": "date"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_daily_transfers_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_daily_transfers_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_daily_transfers_bool_exp"
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
                  "name": "v1_daily_transfers_bool_exp"
                }
              }
            }
          },
          {
            "name": "count",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "day",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "date_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_daily_transfers_order_by",
        "inputFields": [
          {
            "name": "count",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "day",
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
        "name": "v1_daily_transfers_select_column",
        "enumValues": [
          {
            "name": "count",
            "isDeprecated": false
          },
          {
            "name": "day",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_daily_transfers_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_daily_transfers_stream_cursor_value_input"
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
        "name": "v1_daily_transfers_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "count",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "day",
            "type": {
              "kind": "SCALAR",
              "name": "date"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v1_explorers",
        "fields": [
          {
            "name": "address_url",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "block_url",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "description",
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
            "name": "home_url",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "logo_uri",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "tx_url",
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
        "name": "v1_explorers_aggregate_order_by",
        "inputFields": [
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
              "name": "v1_explorers_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_explorers_min_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_explorers_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_explorers_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_explorers_bool_exp"
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
                  "name": "v1_explorers_bool_exp"
                }
              }
            }
          },
          {
            "name": "address_url",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "block_url",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_bool_exp"
            }
          },
          {
            "name": "description",
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
            "name": "home_url",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "logo_uri",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "tx_url",
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
        "name": "v1_explorers_max_order_by",
        "inputFields": [
          {
            "name": "address_url",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "block_url",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "description",
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
            "name": "home_url",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "logo_uri",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "tx_url",
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
        "name": "v1_explorers_min_order_by",
        "inputFields": [
          {
            "name": "address_url",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "block_url",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "description",
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
            "name": "home_url",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "logo_uri",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "tx_url",
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
        "name": "v1_explorers_order_by",
        "inputFields": [
          {
            "name": "address_url",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "block_url",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_order_by"
            }
          },
          {
            "name": "description",
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
            "name": "home_url",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "logo_uri",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "tx_url",
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
        "name": "v1_explorers_select_column",
        "enumValues": [
          {
            "name": "address_url",
            "isDeprecated": false
          },
          {
            "name": "block_url",
            "isDeprecated": false
          },
          {
            "name": "description",
            "isDeprecated": false
          },
          {
            "name": "display_name",
            "isDeprecated": false
          },
          {
            "name": "home_url",
            "isDeprecated": false
          },
          {
            "name": "logo_uri",
            "isDeprecated": false
          },
          {
            "name": "tx_url",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_explorers_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_explorers_stream_cursor_value_input"
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
        "name": "v1_explorers_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "address_url",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "block_url",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "description",
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
            "name": "home_url",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "logo_uri",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "tx_url",
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
        "name": "v1_faucets",
        "fields": [
          {
            "name": "asset",
            "type": {
              "kind": "OBJECT",
              "name": "v1_assets"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
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
            "name": "display_name",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "enabled",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "url",
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
        "name": "v1_faucets_aggregate_order_by",
        "inputFields": [
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
              "name": "v1_faucets_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_faucets_min_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_faucets_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_faucets_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_faucets_bool_exp"
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
                  "name": "v1_faucets_bool_exp"
                }
              }
            }
          },
          {
            "name": "asset",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_assets_bool_exp"
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_bool_exp"
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
            "name": "display_name",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "enabled",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Boolean_comparison_exp"
            }
          },
          {
            "name": "url",
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
        "name": "v1_faucets_max_order_by",
        "inputFields": [
          {
            "name": "denom",
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
            "name": "url",
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
        "name": "v1_faucets_min_order_by",
        "inputFields": [
          {
            "name": "denom",
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
            "name": "url",
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
        "name": "v1_faucets_order_by",
        "inputFields": [
          {
            "name": "asset",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_assets_order_by"
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_order_by"
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
            "name": "display_name",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "enabled",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "url",
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
        "name": "v1_faucets_select_column",
        "enumValues": [
          {
            "name": "denom",
            "isDeprecated": false
          },
          {
            "name": "display_name",
            "isDeprecated": false
          },
          {
            "name": "enabled",
            "isDeprecated": false
          },
          {
            "name": "url",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_faucets_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_faucets_stream_cursor_value_input"
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
        "name": "v1_faucets_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "denom",
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
            "name": "enabled",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "url",
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
        "name": "v1_health_check",
        "fields": [
          {
            "name": "result",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "test",
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
        "name": "v1_health_check_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_health_check_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_health_check_bool_exp"
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
                  "name": "v1_health_check_bool_exp"
                }
              }
            }
          },
          {
            "name": "result",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "test",
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
        "name": "v1_health_check_order_by",
        "inputFields": [
          {
            "name": "result",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "test",
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
        "name": "v1_health_check_select_column",
        "enumValues": [
          {
            "name": "result",
            "isDeprecated": false
          },
          {
            "name": "test",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_health_check_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_health_check_stream_cursor_value_input"
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
        "name": "v1_health_check_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "result",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "test",
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
        "name": "v1_ibc_union_assets",
        "fields": [
          {
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_ibc_union_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "decimals",
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
            "name": "display_name",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "display_symbol",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "faucets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_faucets"
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
                      "name": "v1_ibc_union_faucets_select_column"
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
                      "name": "v1_ibc_union_faucets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_faucets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "gas_token",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "logo_uri",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "origin",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "unwrapped_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_ibc_union_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "unwrapped_denom",
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
        "name": "v1_ibc_union_assets_aggregate_order_by",
        "inputFields": [
          {
            "name": "avg",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_assets_avg_order_by"
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
              "name": "v1_ibc_union_assets_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_assets_min_order_by"
            }
          },
          {
            "name": "stddev",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_assets_stddev_order_by"
            }
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_assets_stddev_pop_order_by"
            }
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_assets_stddev_samp_order_by"
            }
          },
          {
            "name": "sum",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_assets_sum_order_by"
            }
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_assets_var_pop_order_by"
            }
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_assets_var_samp_order_by"
            }
          },
          {
            "name": "variance",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_assets_variance_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_assets_avg_order_by",
        "inputFields": [
          {
            "name": "decimals",
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
        "name": "v1_ibc_union_assets_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_assets_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_assets_bool_exp"
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
                  "name": "v1_ibc_union_assets_bool_exp"
                }
              }
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_bool_exp"
            }
          },
          {
            "name": "decimals",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
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
            "name": "display_name",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "display_symbol",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "faucets",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_faucets_bool_exp"
            }
          },
          {
            "name": "gas_token",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Boolean_comparison_exp"
            }
          },
          {
            "name": "logo_uri",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "origin",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "unwrapped_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_bool_exp"
            }
          },
          {
            "name": "unwrapped_denom",
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
        "name": "v1_ibc_union_assets_max_order_by",
        "inputFields": [
          {
            "name": "decimals",
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
            "name": "display_name",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "display_symbol",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "logo_uri",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "origin",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "unwrapped_denom",
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
        "name": "v1_ibc_union_assets_min_order_by",
        "inputFields": [
          {
            "name": "decimals",
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
            "name": "display_name",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "display_symbol",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "logo_uri",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "origin",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "unwrapped_denom",
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
        "name": "v1_ibc_union_assets_order_by",
        "inputFields": [
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_order_by"
            }
          },
          {
            "name": "decimals",
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
            "name": "display_name",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "display_symbol",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "faucets_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_faucets_aggregate_order_by"
            }
          },
          {
            "name": "gas_token",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "logo_uri",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "origin",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "unwrapped_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_order_by"
            }
          },
          {
            "name": "unwrapped_denom",
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
        "name": "v1_ibc_union_assets_select_column",
        "enumValues": [
          {
            "name": "decimals",
            "isDeprecated": false
          },
          {
            "name": "denom",
            "isDeprecated": false
          },
          {
            "name": "display_name",
            "isDeprecated": false
          },
          {
            "name": "display_symbol",
            "isDeprecated": false
          },
          {
            "name": "gas_token",
            "isDeprecated": false
          },
          {
            "name": "logo_uri",
            "isDeprecated": false
          },
          {
            "name": "origin",
            "isDeprecated": false
          },
          {
            "name": "unwrapped_denom",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_assets_stddev_order_by",
        "inputFields": [
          {
            "name": "decimals",
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
        "name": "v1_ibc_union_assets_stddev_pop_order_by",
        "inputFields": [
          {
            "name": "decimals",
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
        "name": "v1_ibc_union_assets_stddev_samp_order_by",
        "inputFields": [
          {
            "name": "decimals",
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
        "name": "v1_ibc_union_assets_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_ibc_union_assets_stream_cursor_value_input"
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
        "name": "v1_ibc_union_assets_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "decimals",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
            "name": "display_name",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "display_symbol",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "gas_token",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "logo_uri",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "origin",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "unwrapped_denom",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_assets_sum_order_by",
        "inputFields": [
          {
            "name": "decimals",
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
        "name": "v1_ibc_union_assets_var_pop_order_by",
        "inputFields": [
          {
            "name": "decimals",
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
        "name": "v1_ibc_union_assets_var_samp_order_by",
        "inputFields": [
          {
            "name": "decimals",
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
        "name": "v1_ibc_union_assets_variance_order_by",
        "inputFields": [
          {
            "name": "decimals",
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
        "name": "v1_ibc_union_chains",
        "fields": [
          {
            "name": "addr_prefix",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "assets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_assets"
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
                      "name": "v1_ibc_union_assets_select_column"
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
                      "name": "v1_ibc_union_assets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_assets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
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
            "name": "enabled",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "enabled_staging",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "explorers",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_explorers"
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
                      "name": "v1_ibc_union_explorers_select_column"
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
                      "name": "v1_ibc_union_explorers_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_explorers_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "logo_uri",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "rpc_type",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "rpcs",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_rpcs"
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
                      "name": "v1_ibc_union_rpcs_select_column"
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
                      "name": "v1_ibc_union_rpcs_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_rpcs_bool_exp"
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
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_chains_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_chains_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_bool_exp"
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
                  "name": "v1_ibc_union_chains_bool_exp"
                }
              }
            }
          },
          {
            "name": "addr_prefix",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "assets",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_assets_bool_exp"
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
            "name": "enabled",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Boolean_comparison_exp"
            }
          },
          {
            "name": "enabled_staging",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Boolean_comparison_exp"
            }
          },
          {
            "name": "explorers",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_explorers_bool_exp"
            }
          },
          {
            "name": "logo_uri",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "rpc_type",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "rpcs",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_rpcs_bool_exp"
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
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_chains_order_by",
        "inputFields": [
          {
            "name": "addr_prefix",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "assets_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_assets_aggregate_order_by"
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
            "name": "enabled",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "enabled_staging",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "explorers_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_explorers_aggregate_order_by"
            }
          },
          {
            "name": "logo_uri",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "rpc_type",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "rpcs_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_rpcs_aggregate_order_by"
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
        "name": "v1_ibc_union_chains_select_column",
        "enumValues": [
          {
            "name": "addr_prefix",
            "isDeprecated": false
          },
          {
            "name": "chain_id",
            "isDeprecated": false
          },
          {
            "name": "display_name",
            "isDeprecated": false
          },
          {
            "name": "enabled",
            "isDeprecated": false
          },
          {
            "name": "enabled_staging",
            "isDeprecated": false
          },
          {
            "name": "logo_uri",
            "isDeprecated": false
          },
          {
            "name": "rpc_type",
            "isDeprecated": false
          },
          {
            "name": "testnet",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_chains_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_ibc_union_chains_stream_cursor_value_input"
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
        "name": "v1_ibc_union_chains_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "addr_prefix",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
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
            "name": "enabled",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "enabled_staging",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "logo_uri",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "rpc_type",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "testnet",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v1_ibc_union_channels",
        "fields": [
          {
            "name": "connection",
            "type": {
              "kind": "OBJECT",
              "name": "v1_ibc_union_connections"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_ibc_union_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "internal_source_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_ibc_union_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_port_id",
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
        "name": "v1_ibc_union_channels_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_channels_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_channels_bool_exp"
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
                  "name": "v1_ibc_union_channels_bool_exp"
                }
              }
            }
          },
          {
            "name": "connection",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_connections_bool_exp"
            }
          },
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_bool_exp"
            }
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "destination_port_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "internal_source_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_bool_exp"
            }
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "source_port_id",
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
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_channels_order_by",
        "inputFields": [
          {
            "name": "connection",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_connections_order_by"
            }
          },
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_order_by"
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
            "name": "destination_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_port_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "internal_source_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_order_by"
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
            "name": "source_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_port_id",
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
        "name": "v1_ibc_union_channels_select_column",
        "enumValues": [
          {
            "name": "destination_chain_id",
            "isDeprecated": false
          },
          {
            "name": "destination_channel_id",
            "isDeprecated": false
          },
          {
            "name": "destination_connection_id",
            "isDeprecated": false
          },
          {
            "name": "destination_port_id",
            "isDeprecated": false
          },
          {
            "name": "internal_source_chain_id",
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
            "isDeprecated": false
          },
          {
            "name": "source_channel_id",
            "isDeprecated": false
          },
          {
            "name": "source_connection_id",
            "isDeprecated": false
          },
          {
            "name": "source_port_id",
            "isDeprecated": false
          },
          {
            "name": "status",
            "isDeprecated": false
          },
          {
            "name": "version",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_channels_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_ibc_union_channels_stream_cursor_value_input"
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
        "name": "v1_ibc_union_channels_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "destination_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "internal_source_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "source_port_id",
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
        "name": "v1_ibc_union_clients",
        "fields": [
          {
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_ibc_union_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "client_id",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "counterparty_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_ibc_union_chains"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_clients_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_clients_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_clients_bool_exp"
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
                  "name": "v1_ibc_union_clients_bool_exp"
                }
              }
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_bool_exp"
            }
          },
          {
            "name": "client_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "counterparty_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_bool_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_clients_order_by",
        "inputFields": [
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_order_by"
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
            "name": "counterparty_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v1_ibc_union_clients_select_column",
        "enumValues": [
          {
            "name": "client_id",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_clients_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_ibc_union_clients_stream_cursor_value_input"
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
        "name": "v1_ibc_union_clients_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "client_id",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v1_ibc_union_config",
        "fields": [
          {
            "name": "key",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "value",
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
        "name": "v1_ibc_union_config_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_config_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_config_bool_exp"
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
                  "name": "v1_ibc_union_config_bool_exp"
                }
              }
            }
          },
          {
            "name": "key",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "value",
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
        "name": "v1_ibc_union_config_order_by",
        "inputFields": [
          {
            "name": "key",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "value",
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
        "name": "v1_ibc_union_config_select_column",
        "enumValues": [
          {
            "name": "key",
            "isDeprecated": false
          },
          {
            "name": "value",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_config_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_ibc_union_config_stream_cursor_value_input"
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
        "name": "v1_ibc_union_config_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "key",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "value",
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
        "name": "v1_ibc_union_connections",
        "fields": [
          {
            "name": "destination_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_ibc_union_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_ibc_union_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_connection_id",
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
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_connections_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_connections_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_connections_bool_exp"
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
                  "name": "v1_ibc_union_connections_bool_exp"
                }
              }
            }
          },
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_bool_exp"
            }
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_client_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_bool_exp"
            }
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_client_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "source_connection_id",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_connections_order_by",
        "inputFields": [
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_order_by"
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
            "name": "destination_client_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_order_by"
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
            "name": "source_client_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_connection_id",
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
        "name": "v1_ibc_union_connections_select_column",
        "enumValues": [
          {
            "name": "destination_chain_id",
            "isDeprecated": false
          },
          {
            "name": "destination_client_id",
            "isDeprecated": false
          },
          {
            "name": "destination_connection_id",
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
            "isDeprecated": false
          },
          {
            "name": "source_client_id",
            "isDeprecated": false
          },
          {
            "name": "source_connection_id",
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
        "name": "v1_ibc_union_connections_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_ibc_union_connections_stream_cursor_value_input"
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
        "name": "v1_ibc_union_connections_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "source_connection_id",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v1_ibc_union_contracts",
        "fields": [
          {
            "name": "abi",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "address",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_ibc_union_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "created_at",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "description",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "end_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "start_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "type",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "updated_at",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
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
        "name": "v1_ibc_union_contracts_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_contracts_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_contracts_bool_exp"
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
                  "name": "v1_ibc_union_contracts_bool_exp"
                }
              }
            }
          },
          {
            "name": "abi",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "address",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_bool_exp"
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
            "name": "description",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "end_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "start_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "type",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "updated_at",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
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
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_contracts_order_by",
        "inputFields": [
          {
            "name": "abi",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "address",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_order_by"
            }
          },
          {
            "name": "created_at",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "description",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "end_height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "start_height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "type",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "updated_at",
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
        "name": "v1_ibc_union_contracts_select_column",
        "enumValues": [
          {
            "name": "abi",
            "isDeprecated": false
          },
          {
            "name": "address",
            "isDeprecated": false
          },
          {
            "name": "created_at",
            "isDeprecated": false
          },
          {
            "name": "description",
            "isDeprecated": false
          },
          {
            "name": "end_height",
            "isDeprecated": false
          },
          {
            "name": "start_height",
            "isDeprecated": false
          },
          {
            "name": "type",
            "isDeprecated": false
          },
          {
            "name": "updated_at",
            "isDeprecated": false
          },
          {
            "name": "version",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_contracts_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_ibc_union_contracts_stream_cursor_value_input"
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
        "name": "v1_ibc_union_contracts_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "abi",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "address",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "created_at",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "description",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "end_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "start_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "type",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "updated_at",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
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
        "name": "v1_ibc_union_daily_fungible_asset_orders",
        "fields": [
          {
            "name": "count",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "day",
            "type": {
              "kind": "SCALAR",
              "name": "date"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_daily_fungible_asset_orders_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_daily_fungible_asset_orders_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_daily_fungible_asset_orders_bool_exp"
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
                  "name": "v1_ibc_union_daily_fungible_asset_orders_bool_exp"
                }
              }
            }
          },
          {
            "name": "count",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "day",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "date_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_daily_fungible_asset_orders_order_by",
        "inputFields": [
          {
            "name": "count",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "day",
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
        "name": "v1_ibc_union_daily_fungible_asset_orders_select_column",
        "enumValues": [
          {
            "name": "count",
            "isDeprecated": false
          },
          {
            "name": "day",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_daily_fungible_asset_orders_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_ibc_union_daily_fungible_asset_orders_stream_cursor_value_input"
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
        "name": "v1_ibc_union_daily_fungible_asset_orders_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "count",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "day",
            "type": {
              "kind": "SCALAR",
              "name": "date"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v1_ibc_union_daily_packets",
        "fields": [
          {
            "name": "count",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "day",
            "type": {
              "kind": "SCALAR",
              "name": "date"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_daily_packets_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_daily_packets_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_daily_packets_bool_exp"
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
                  "name": "v1_ibc_union_daily_packets_bool_exp"
                }
              }
            }
          },
          {
            "name": "count",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "day",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "date_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_daily_packets_order_by",
        "inputFields": [
          {
            "name": "count",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "day",
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
        "name": "v1_ibc_union_daily_packets_select_column",
        "enumValues": [
          {
            "name": "count",
            "isDeprecated": false
          },
          {
            "name": "day",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_daily_packets_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_ibc_union_daily_packets_stream_cursor_value_input"
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
        "name": "v1_ibc_union_daily_packets_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "count",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "day",
            "type": {
              "kind": "SCALAR",
              "name": "date"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v1_ibc_union_explorers",
        "fields": [
          {
            "name": "address_url",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "block_url",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_ibc_union_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "description",
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
            "name": "home_url",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "logo_uri",
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
            "name": "tx_url",
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
        "name": "v1_ibc_union_explorers_aggregate_order_by",
        "inputFields": [
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
              "name": "v1_ibc_union_explorers_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_explorers_min_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_explorers_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_explorers_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_explorers_bool_exp"
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
                  "name": "v1_ibc_union_explorers_bool_exp"
                }
              }
            }
          },
          {
            "name": "address_url",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "block_url",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_bool_exp"
            }
          },
          {
            "name": "description",
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
            "name": "home_url",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "logo_uri",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
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
            "name": "tx_url",
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
        "name": "v1_ibc_union_explorers_max_order_by",
        "inputFields": [
          {
            "name": "address_url",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "block_url",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "description",
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
            "name": "home_url",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "logo_uri",
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
            "name": "tx_url",
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
        "name": "v1_ibc_union_explorers_min_order_by",
        "inputFields": [
          {
            "name": "address_url",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "block_url",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "description",
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
            "name": "home_url",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "logo_uri",
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
            "name": "tx_url",
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
        "name": "v1_ibc_union_explorers_order_by",
        "inputFields": [
          {
            "name": "address_url",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "block_url",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_order_by"
            }
          },
          {
            "name": "description",
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
            "name": "home_url",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "logo_uri",
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
            "name": "tx_url",
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
        "name": "v1_ibc_union_explorers_select_column",
        "enumValues": [
          {
            "name": "address_url",
            "isDeprecated": false
          },
          {
            "name": "block_url",
            "isDeprecated": false
          },
          {
            "name": "description",
            "isDeprecated": false
          },
          {
            "name": "display_name",
            "isDeprecated": false
          },
          {
            "name": "home_url",
            "isDeprecated": false
          },
          {
            "name": "logo_uri",
            "isDeprecated": false
          },
          {
            "name": "name",
            "isDeprecated": false
          },
          {
            "name": "tx_url",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_explorers_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_ibc_union_explorers_stream_cursor_value_input"
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
        "name": "v1_ibc_union_explorers_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "address_url",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "block_url",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "description",
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
            "name": "home_url",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "logo_uri",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "name",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "tx_url",
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
        "name": "v1_ibc_union_faucets",
        "fields": [
          {
            "name": "asset",
            "type": {
              "kind": "OBJECT",
              "name": "v1_ibc_union_assets"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_ibc_union_chains"
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
            "name": "display_name",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "enabled",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "url",
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
        "name": "v1_ibc_union_faucets_aggregate_order_by",
        "inputFields": [
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
              "name": "v1_ibc_union_faucets_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_faucets_min_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_faucets_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_faucets_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_faucets_bool_exp"
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
                  "name": "v1_ibc_union_faucets_bool_exp"
                }
              }
            }
          },
          {
            "name": "asset",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_assets_bool_exp"
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_bool_exp"
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
            "name": "display_name",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "enabled",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Boolean_comparison_exp"
            }
          },
          {
            "name": "url",
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
        "name": "v1_ibc_union_faucets_max_order_by",
        "inputFields": [
          {
            "name": "denom",
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
            "name": "url",
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
        "name": "v1_ibc_union_faucets_min_order_by",
        "inputFields": [
          {
            "name": "denom",
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
            "name": "url",
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
        "name": "v1_ibc_union_faucets_order_by",
        "inputFields": [
          {
            "name": "asset",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_assets_order_by"
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_order_by"
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
            "name": "display_name",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "enabled",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "url",
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
        "name": "v1_ibc_union_faucets_select_column",
        "enumValues": [
          {
            "name": "denom",
            "isDeprecated": false
          },
          {
            "name": "display_name",
            "isDeprecated": false
          },
          {
            "name": "enabled",
            "isDeprecated": false
          },
          {
            "name": "url",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_faucets_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_ibc_union_faucets_stream_cursor_value_input"
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
        "name": "v1_ibc_union_faucets_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "denom",
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
            "name": "enabled",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "url",
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
        "name": "v1_ibc_union_fungible_asset_orders",
        "fields": [
          {
            "name": "ack_fill_type",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "ack_market_maker",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "ack_tag",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "acknowledgement",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "acknowledgement_decoded",
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
            "name": "base_amount",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "base_token",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "base_token_details",
            "type": {
              "kind": "OBJECT",
              "name": "v1_ibc_union_assets"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "base_token_name",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "base_token_path",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "base_token_symbol",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "channel_version",
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
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "data_decoded",
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
            "name": "destination_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_ibc_union_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "instruction_path",
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
              "kind": "OBJECT",
              "name": "v1_ibc_union_packets"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_ack_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_ack_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_ack_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_ack_maker",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_ack_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_ack_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_ack_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_ack_transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_recv_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_recv_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_recv_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_recv_maker",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_recv_maker_msg",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_recv_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_recv_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_recv_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_recv_transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_send_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_send_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_send_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_send_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_send_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_send_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_send_transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "path",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "quote_amount",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "quote_token",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "quote_token_details",
            "type": {
              "kind": "OBJECT",
              "name": "v1_ibc_union_assets"
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
            "name": "salt",
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
            "name": "source_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_ibc_union_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_port_id",
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
            "name": "timeout_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
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
            "name": "traces",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_traces"
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
                      "name": "v1_ibc_union_traces_select_column"
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
                      "name": "v1_ibc_union_traces_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_traces_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "write_ack_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "write_ack_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "write_ack_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "write_ack_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "write_ack_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "write_ack_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "write_ack_transaction_index",
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
        "name": "v1_ibc_union_fungible_asset_orders_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_fungible_asset_orders_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_fungible_asset_orders_bool_exp"
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
                  "name": "v1_ibc_union_fungible_asset_orders_bool_exp"
                }
              }
            }
          },
          {
            "name": "ack_fill_type",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "ack_market_maker",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "ack_tag",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "acknowledgement",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "acknowledgement_decoded",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "base_amount",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "base_token",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "base_token_details",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_assets_bool_exp"
            }
          },
          {
            "name": "base_token_name",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "base_token_path",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "base_token_symbol",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "channel_version",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
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
            "name": "data_decoded",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_bool_exp"
            }
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "destination_port_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "instruction_path",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_packets_bool_exp"
            }
          },
          {
            "name": "packet_ack_block_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet_ack_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "packet_ack_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "packet_ack_maker",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet_ack_timestamp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "packet_ack_transaction_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "packet_ack_transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet_ack_transaction_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "packet_recv_block_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet_recv_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "packet_recv_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "packet_recv_maker",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet_recv_maker_msg",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet_recv_timestamp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "packet_recv_transaction_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "packet_recv_transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet_recv_transaction_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "packet_send_block_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet_send_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "packet_send_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "packet_send_timestamp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "packet_send_transaction_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "packet_send_transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet_send_transaction_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "path",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "quote_amount",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "quote_token",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "quote_token_details",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_assets_bool_exp"
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
            "name": "salt",
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
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_bool_exp"
            }
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "source_port_id",
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
            "name": "timeout_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
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
            "name": "traces",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_traces_bool_exp"
            }
          },
          {
            "name": "write_ack_block_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "write_ack_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "write_ack_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "write_ack_timestamp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "write_ack_transaction_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "write_ack_transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "write_ack_transaction_index",
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
        "name": "v1_ibc_union_fungible_asset_orders_order_by",
        "inputFields": [
          {
            "name": "ack_fill_type",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "ack_market_maker",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "ack_tag",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "acknowledgement",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "acknowledgement_decoded",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "base_amount",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "base_token",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "base_token_details",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_assets_order_by"
            }
          },
          {
            "name": "base_token_name",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "base_token_path",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "base_token_symbol",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "channel_version",
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
            "name": "data_decoded",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_order_by"
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
            "name": "destination_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_port_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "instruction_path",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_packets_order_by"
            }
          },
          {
            "name": "packet_ack_block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_ack_event_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_ack_height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_ack_maker",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_ack_timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_ack_transaction_event_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_ack_transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_ack_transaction_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_recv_block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_recv_event_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_recv_height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_recv_maker",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_recv_maker_msg",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_recv_timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_recv_transaction_event_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_recv_transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_recv_transaction_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_send_block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_send_event_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_send_height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_send_timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_send_transaction_event_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_send_transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_send_transaction_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "path",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "quote_amount",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "quote_token",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "quote_token_details",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_assets_order_by"
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
            "name": "salt",
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
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_order_by"
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
            "name": "source_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_port_id",
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
            "name": "traces_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_traces_aggregate_order_by"
            }
          },
          {
            "name": "write_ack_block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "write_ack_event_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "write_ack_height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "write_ack_timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "write_ack_transaction_event_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "write_ack_transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "write_ack_transaction_index",
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
        "name": "v1_ibc_union_fungible_asset_orders_select_column",
        "enumValues": [
          {
            "name": "ack_fill_type",
            "isDeprecated": false
          },
          {
            "name": "ack_market_maker",
            "isDeprecated": false
          },
          {
            "name": "ack_tag",
            "isDeprecated": false
          },
          {
            "name": "acknowledgement",
            "isDeprecated": false
          },
          {
            "name": "acknowledgement_decoded",
            "isDeprecated": false
          },
          {
            "name": "base_amount",
            "isDeprecated": false
          },
          {
            "name": "base_token",
            "isDeprecated": false
          },
          {
            "name": "base_token_name",
            "isDeprecated": false
          },
          {
            "name": "base_token_path",
            "isDeprecated": false
          },
          {
            "name": "base_token_symbol",
            "isDeprecated": false
          },
          {
            "name": "channel_version",
            "isDeprecated": false
          },
          {
            "name": "data",
            "isDeprecated": false
          },
          {
            "name": "data_decoded",
            "isDeprecated": false
          },
          {
            "name": "destination_chain_id",
            "isDeprecated": false
          },
          {
            "name": "destination_channel_id",
            "isDeprecated": false
          },
          {
            "name": "destination_connection_id",
            "isDeprecated": false
          },
          {
            "name": "destination_port_id",
            "isDeprecated": false
          },
          {
            "name": "instruction_path",
            "isDeprecated": false
          },
          {
            "name": "packet_ack_block_hash",
            "isDeprecated": false
          },
          {
            "name": "packet_ack_event_index",
            "isDeprecated": false
          },
          {
            "name": "packet_ack_height",
            "isDeprecated": false
          },
          {
            "name": "packet_ack_maker",
            "isDeprecated": false
          },
          {
            "name": "packet_ack_timestamp",
            "isDeprecated": false
          },
          {
            "name": "packet_ack_transaction_event_index",
            "isDeprecated": false
          },
          {
            "name": "packet_ack_transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "packet_ack_transaction_index",
            "isDeprecated": false
          },
          {
            "name": "packet_recv_block_hash",
            "isDeprecated": false
          },
          {
            "name": "packet_recv_event_index",
            "isDeprecated": false
          },
          {
            "name": "packet_recv_height",
            "isDeprecated": false
          },
          {
            "name": "packet_recv_maker",
            "isDeprecated": false
          },
          {
            "name": "packet_recv_maker_msg",
            "isDeprecated": false
          },
          {
            "name": "packet_recv_timestamp",
            "isDeprecated": false
          },
          {
            "name": "packet_recv_transaction_event_index",
            "isDeprecated": false
          },
          {
            "name": "packet_recv_transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "packet_recv_transaction_index",
            "isDeprecated": false
          },
          {
            "name": "packet_send_block_hash",
            "isDeprecated": false
          },
          {
            "name": "packet_send_event_index",
            "isDeprecated": false
          },
          {
            "name": "packet_send_height",
            "isDeprecated": false
          },
          {
            "name": "packet_send_timestamp",
            "isDeprecated": false
          },
          {
            "name": "packet_send_transaction_event_index",
            "isDeprecated": false
          },
          {
            "name": "packet_send_transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "packet_send_transaction_index",
            "isDeprecated": false
          },
          {
            "name": "path",
            "isDeprecated": false
          },
          {
            "name": "quote_amount",
            "isDeprecated": false
          },
          {
            "name": "quote_token",
            "isDeprecated": false
          },
          {
            "name": "receiver",
            "isDeprecated": false
          },
          {
            "name": "salt",
            "isDeprecated": false
          },
          {
            "name": "sender",
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
            "isDeprecated": false
          },
          {
            "name": "source_channel_id",
            "isDeprecated": false
          },
          {
            "name": "source_connection_id",
            "isDeprecated": false
          },
          {
            "name": "source_port_id",
            "isDeprecated": false
          },
          {
            "name": "status",
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
            "name": "write_ack_block_hash",
            "isDeprecated": false
          },
          {
            "name": "write_ack_event_index",
            "isDeprecated": false
          },
          {
            "name": "write_ack_height",
            "isDeprecated": false
          },
          {
            "name": "write_ack_timestamp",
            "isDeprecated": false
          },
          {
            "name": "write_ack_transaction_event_index",
            "isDeprecated": false
          },
          {
            "name": "write_ack_transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "write_ack_transaction_index",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_fungible_asset_orders_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_ibc_union_fungible_asset_orders_stream_cursor_value_input"
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
        "name": "v1_ibc_union_fungible_asset_orders_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "ack_fill_type",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "ack_market_maker",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "ack_tag",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "acknowledgement",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "acknowledgement_decoded",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "base_amount",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "base_token",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "base_token_name",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "base_token_path",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "base_token_symbol",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "channel_version",
            "type": {
              "kind": "SCALAR",
              "name": "String"
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
            "name": "data_decoded",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "destination_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "instruction_path",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "packet_ack_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "packet_ack_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "packet_ack_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "packet_ack_maker",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "packet_ack_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "packet_ack_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "packet_ack_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "packet_ack_transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "packet_recv_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "packet_recv_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "packet_recv_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "packet_recv_maker",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "packet_recv_maker_msg",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "packet_recv_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "packet_recv_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "packet_recv_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "packet_recv_transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "packet_send_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "packet_send_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "packet_send_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "packet_send_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "packet_send_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "packet_send_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "packet_send_transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "path",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "quote_amount",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "quote_token",
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
            "name": "salt",
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
            "name": "source_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "source_port_id",
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
            "name": "timeout_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
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
            "name": "write_ack_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "write_ack_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "write_ack_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "write_ack_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "write_ack_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "write_ack_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "write_ack_transaction_index",
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
        "name": "v1_ibc_union_health_check",
        "fields": [
          {
            "name": "result",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "test",
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
        "name": "v1_ibc_union_health_check_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_health_check_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_health_check_bool_exp"
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
                  "name": "v1_ibc_union_health_check_bool_exp"
                }
              }
            }
          },
          {
            "name": "result",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "test",
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
        "name": "v1_ibc_union_health_check_order_by",
        "inputFields": [
          {
            "name": "result",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "test",
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
        "name": "v1_ibc_union_health_check_select_column",
        "enumValues": [
          {
            "name": "result",
            "isDeprecated": false
          },
          {
            "name": "test",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_health_check_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_ibc_union_health_check_stream_cursor_value_input"
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
        "name": "v1_ibc_union_health_check_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "result",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "test",
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
        "name": "v1_ibc_union_packets",
        "fields": [
          {
            "name": "acknowledgement",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "acknowledgement_decoded",
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
            "name": "channel_version",
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
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "data_decoded",
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
            "name": "destination_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_ibc_union_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "fungible_asset_order",
            "type": {
              "kind": "OBJECT",
              "name": "v1_ibc_union_fungible_asset_orders"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_ack_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_ack_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_ack_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_ack_maker",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_ack_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_ack_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_ack_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_ack_transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_recv_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_recv_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_recv_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_recv_maker",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_recv_maker_msg",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_recv_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_recv_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_recv_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_recv_transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_send_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_send_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_send_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_send_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_send_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_send_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_send_transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_ibc_union_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_port_id",
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
            "name": "timeout_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
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
            "name": "traces",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ibc_union_traces"
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
                      "name": "v1_ibc_union_traces_select_column"
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
                      "name": "v1_ibc_union_traces_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_traces_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "write_ack_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "write_ack_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "write_ack_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "write_ack_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "write_ack_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "write_ack_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "write_ack_transaction_index",
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
        "name": "v1_ibc_union_packets_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_packets_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_packets_bool_exp"
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
                  "name": "v1_ibc_union_packets_bool_exp"
                }
              }
            }
          },
          {
            "name": "acknowledgement",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "acknowledgement_decoded",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "channel_version",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
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
            "name": "data_decoded",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_bool_exp"
            }
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "destination_port_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "fungible_asset_order",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_fungible_asset_orders_bool_exp"
            }
          },
          {
            "name": "packet_ack_block_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet_ack_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "packet_ack_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "packet_ack_maker",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet_ack_timestamp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "packet_ack_transaction_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "packet_ack_transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet_ack_transaction_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "packet_recv_block_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet_recv_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "packet_recv_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "packet_recv_maker",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet_recv_maker_msg",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet_recv_timestamp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "packet_recv_transaction_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "packet_recv_transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet_recv_transaction_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "packet_send_block_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet_send_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "packet_send_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "packet_send_timestamp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "packet_send_transaction_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "packet_send_transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet_send_transaction_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_bool_exp"
            }
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "source_port_id",
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
            "name": "timeout_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
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
            "name": "traces",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_traces_bool_exp"
            }
          },
          {
            "name": "write_ack_block_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "write_ack_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "write_ack_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "write_ack_timestamp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "write_ack_transaction_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "write_ack_transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "write_ack_transaction_index",
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
        "name": "v1_ibc_union_packets_order_by",
        "inputFields": [
          {
            "name": "acknowledgement",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "acknowledgement_decoded",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "channel_version",
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
            "name": "data_decoded",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_order_by"
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
            "name": "destination_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_port_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "fungible_asset_order",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_fungible_asset_orders_order_by"
            }
          },
          {
            "name": "packet_ack_block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_ack_event_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_ack_height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_ack_maker",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_ack_timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_ack_transaction_event_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_ack_transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_ack_transaction_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_recv_block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_recv_event_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_recv_height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_recv_maker",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_recv_maker_msg",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_recv_timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_recv_transaction_event_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_recv_transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_recv_transaction_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_send_block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_send_event_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_send_height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_send_timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_send_transaction_event_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_send_transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_send_transaction_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_order_by"
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
            "name": "source_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_port_id",
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
            "name": "traces_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_traces_aggregate_order_by"
            }
          },
          {
            "name": "write_ack_block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "write_ack_event_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "write_ack_height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "write_ack_timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "write_ack_transaction_event_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "write_ack_transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "write_ack_transaction_index",
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
        "name": "v1_ibc_union_packets_select_column",
        "enumValues": [
          {
            "name": "acknowledgement",
            "isDeprecated": false
          },
          {
            "name": "acknowledgement_decoded",
            "isDeprecated": false
          },
          {
            "name": "channel_version",
            "isDeprecated": false
          },
          {
            "name": "data",
            "isDeprecated": false
          },
          {
            "name": "data_decoded",
            "isDeprecated": false
          },
          {
            "name": "destination_chain_id",
            "isDeprecated": false
          },
          {
            "name": "destination_channel_id",
            "isDeprecated": false
          },
          {
            "name": "destination_connection_id",
            "isDeprecated": false
          },
          {
            "name": "destination_port_id",
            "isDeprecated": false
          },
          {
            "name": "packet_ack_block_hash",
            "isDeprecated": false
          },
          {
            "name": "packet_ack_event_index",
            "isDeprecated": false
          },
          {
            "name": "packet_ack_height",
            "isDeprecated": false
          },
          {
            "name": "packet_ack_maker",
            "isDeprecated": false
          },
          {
            "name": "packet_ack_timestamp",
            "isDeprecated": false
          },
          {
            "name": "packet_ack_transaction_event_index",
            "isDeprecated": false
          },
          {
            "name": "packet_ack_transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "packet_ack_transaction_index",
            "isDeprecated": false
          },
          {
            "name": "packet_recv_block_hash",
            "isDeprecated": false
          },
          {
            "name": "packet_recv_event_index",
            "isDeprecated": false
          },
          {
            "name": "packet_recv_height",
            "isDeprecated": false
          },
          {
            "name": "packet_recv_maker",
            "isDeprecated": false
          },
          {
            "name": "packet_recv_maker_msg",
            "isDeprecated": false
          },
          {
            "name": "packet_recv_timestamp",
            "isDeprecated": false
          },
          {
            "name": "packet_recv_transaction_event_index",
            "isDeprecated": false
          },
          {
            "name": "packet_recv_transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "packet_recv_transaction_index",
            "isDeprecated": false
          },
          {
            "name": "packet_send_block_hash",
            "isDeprecated": false
          },
          {
            "name": "packet_send_event_index",
            "isDeprecated": false
          },
          {
            "name": "packet_send_height",
            "isDeprecated": false
          },
          {
            "name": "packet_send_timestamp",
            "isDeprecated": false
          },
          {
            "name": "packet_send_transaction_event_index",
            "isDeprecated": false
          },
          {
            "name": "packet_send_transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "packet_send_transaction_index",
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
            "isDeprecated": false
          },
          {
            "name": "source_channel_id",
            "isDeprecated": false
          },
          {
            "name": "source_connection_id",
            "isDeprecated": false
          },
          {
            "name": "source_port_id",
            "isDeprecated": false
          },
          {
            "name": "status",
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
            "name": "write_ack_block_hash",
            "isDeprecated": false
          },
          {
            "name": "write_ack_event_index",
            "isDeprecated": false
          },
          {
            "name": "write_ack_height",
            "isDeprecated": false
          },
          {
            "name": "write_ack_timestamp",
            "isDeprecated": false
          },
          {
            "name": "write_ack_transaction_event_index",
            "isDeprecated": false
          },
          {
            "name": "write_ack_transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "write_ack_transaction_index",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_packets_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_ibc_union_packets_stream_cursor_value_input"
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
        "name": "v1_ibc_union_packets_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "acknowledgement",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "acknowledgement_decoded",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "channel_version",
            "type": {
              "kind": "SCALAR",
              "name": "String"
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
            "name": "data_decoded",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "destination_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "packet_ack_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "packet_ack_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "packet_ack_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "packet_ack_maker",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "packet_ack_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "packet_ack_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "packet_ack_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "packet_ack_transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "packet_recv_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "packet_recv_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "packet_recv_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "packet_recv_maker",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "packet_recv_maker_msg",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "packet_recv_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "packet_recv_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "packet_recv_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "packet_recv_transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "packet_send_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "packet_send_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "packet_send_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "packet_send_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "packet_send_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "packet_send_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "packet_send_transaction_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "source_port_id",
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
            "name": "timeout_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
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
            "name": "write_ack_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "write_ack_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "write_ack_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "write_ack_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "write_ack_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "write_ack_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "write_ack_transaction_index",
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
        "name": "v1_ibc_union_rpcs",
        "fields": [
          {
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_ibc_union_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "contact",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "description",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "enabled",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "type",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "url",
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
        "name": "v1_ibc_union_rpcs_aggregate_order_by",
        "inputFields": [
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
              "name": "v1_ibc_union_rpcs_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_rpcs_min_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_rpcs_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_rpcs_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_rpcs_bool_exp"
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
                  "name": "v1_ibc_union_rpcs_bool_exp"
                }
              }
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_bool_exp"
            }
          },
          {
            "name": "contact",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "description",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "enabled",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Boolean_comparison_exp"
            }
          },
          {
            "name": "type",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "url",
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
        "name": "v1_ibc_union_rpcs_max_order_by",
        "inputFields": [
          {
            "name": "contact",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "description",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "type",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "url",
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
        "name": "v1_ibc_union_rpcs_min_order_by",
        "inputFields": [
          {
            "name": "contact",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "description",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "type",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "url",
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
        "name": "v1_ibc_union_rpcs_order_by",
        "inputFields": [
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_chains_order_by"
            }
          },
          {
            "name": "contact",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "description",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "enabled",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "type",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "url",
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
        "name": "v1_ibc_union_rpcs_select_column",
        "enumValues": [
          {
            "name": "contact",
            "isDeprecated": false
          },
          {
            "name": "description",
            "isDeprecated": false
          },
          {
            "name": "enabled",
            "isDeprecated": false
          },
          {
            "name": "type",
            "isDeprecated": false
          },
          {
            "name": "url",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_rpcs_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_ibc_union_rpcs_stream_cursor_value_input"
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
        "name": "v1_ibc_union_rpcs_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "contact",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "description",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "enabled",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "type",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "url",
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
        "name": "v1_ibc_union_statistics",
        "fields": [
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
            "name": "value",
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
        "name": "v1_ibc_union_statistics_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_statistics_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_statistics_bool_exp"
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
                  "name": "v1_ibc_union_statistics_bool_exp"
                }
              }
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
            "name": "value",
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
        "name": "v1_ibc_union_statistics_order_by",
        "inputFields": [
          {
            "name": "name",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "value",
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
        "name": "v1_ibc_union_statistics_select_column",
        "enumValues": [
          {
            "name": "name",
            "isDeprecated": false
          },
          {
            "name": "value",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_statistics_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_ibc_union_statistics_stream_cursor_value_input"
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
        "name": "v1_ibc_union_statistics_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "name",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "value",
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
        "name": "v1_ibc_union_traces",
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
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_ibc_union_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "initiating_transaction_hash",
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
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "type",
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
        "name": "v1_ibc_union_traces_aggregate_order_by",
        "inputFields": [
          {
            "name": "avg",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_traces_avg_order_by"
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
              "name": "v1_ibc_union_traces_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_traces_min_order_by"
            }
          },
          {
            "name": "stddev",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_traces_stddev_order_by"
            }
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_traces_stddev_pop_order_by"
            }
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_traces_stddev_samp_order_by"
            }
          },
          {
            "name": "sum",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_traces_sum_order_by"
            }
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_traces_var_pop_order_by"
            }
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_traces_var_samp_order_by"
            }
          },
          {
            "name": "variance",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_traces_variance_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_traces_avg_order_by",
        "inputFields": [
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
        "name": "v1_ibc_union_traces_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ibc_union_traces_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ibc_union_traces_bool_exp"
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
                  "name": "v1_ibc_union_traces_bool_exp"
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
              "name": "v1_ibc_union_chains_bool_exp"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "initiating_transaction_hash",
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
            "name": "transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "type",
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
        "name": "v1_ibc_union_traces_max_order_by",
        "inputFields": [
          {
            "name": "block_hash",
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
            "name": "initiating_transaction_hash",
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
            "name": "transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "type",
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
        "name": "v1_ibc_union_traces_min_order_by",
        "inputFields": [
          {
            "name": "block_hash",
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
            "name": "initiating_transaction_hash",
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
            "name": "transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "type",
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
        "name": "v1_ibc_union_traces_order_by",
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
              "name": "v1_ibc_union_chains_order_by"
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
            "name": "initiating_transaction_hash",
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
            "name": "transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "type",
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
        "name": "v1_ibc_union_traces_select_column",
        "enumValues": [
          {
            "name": "block_hash",
            "isDeprecated": false
          },
          {
            "name": "height",
            "isDeprecated": false
          },
          {
            "name": "initiating_transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "timestamp",
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "type",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_traces_stddev_order_by",
        "inputFields": [
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
        "name": "v1_ibc_union_traces_stddev_pop_order_by",
        "inputFields": [
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
        "name": "v1_ibc_union_traces_stddev_samp_order_by",
        "inputFields": [
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
        "name": "v1_ibc_union_traces_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_ibc_union_traces_stream_cursor_value_input"
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
        "name": "v1_ibc_union_traces_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "initiating_transaction_hash",
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
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "type",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ibc_union_traces_sum_order_by",
        "inputFields": [
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
        "name": "v1_ibc_union_traces_var_pop_order_by",
        "inputFields": [
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
        "name": "v1_ibc_union_traces_var_samp_order_by",
        "inputFields": [
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
        "name": "v1_ibc_union_traces_variance_order_by",
        "inputFields": [
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
        "name": "v1_index_status",
        "fields": [
          {
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
            },
            "args": [],
            "isDeprecated": false
          },
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
              "name": "bigint"
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
        "name": "v1_index_status_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_index_status_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_index_status_bool_exp"
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
                  "name": "v1_index_status_bool_exp"
                }
              }
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_bool_exp"
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
              "name": "bigint_comparison_exp"
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
        "name": "v1_index_status_order_by",
        "inputFields": [
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_order_by"
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
            "name": "height",
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
        "name": "v1_index_status_select_column",
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
        "name": "v1_index_status_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_index_status_stream_cursor_value_input"
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
        "name": "v1_index_status_stream_cursor_value_input",
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
              "name": "bigint"
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
        "name": "v1_lightclient_update",
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
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
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
            "name": "counterparty_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "execution_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "revision_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
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
        "name": "v1_lightclient_update_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_lightclient_update_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_lightclient_update_bool_exp"
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
                  "name": "v1_lightclient_update_bool_exp"
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
              "name": "v1_chains_bool_exp"
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
            "name": "counterparty_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_bool_exp"
            }
          },
          {
            "name": "execution_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "revision_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
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
        "name": "v1_lightclient_update_order_by",
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
              "name": "v1_chains_order_by"
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
            "name": "counterparty_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_order_by"
            }
          },
          {
            "name": "execution_height",
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
            "name": "revision_height",
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
        "name": "v1_lightclient_update_select_column",
        "enumValues": [
          {
            "name": "block_hash",
            "isDeprecated": false
          },
          {
            "name": "client_id",
            "isDeprecated": false
          },
          {
            "name": "execution_height",
            "isDeprecated": false
          },
          {
            "name": "height",
            "isDeprecated": false
          },
          {
            "name": "revision_height",
            "isDeprecated": false
          },
          {
            "name": "timestamp",
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
        "name": "v1_lightclient_update_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_lightclient_update_stream_cursor_value_input"
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
        "name": "v1_lightclient_update_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "client_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "execution_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "revision_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
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
        "name": "v1_packet_effect_type",
        "fields": [
          {
            "name": "event",
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
            "name": "event_index",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_packet_effect_type_aggregate_order_by",
        "inputFields": [
          {
            "name": "avg",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_packet_effect_type_avg_order_by"
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
              "name": "v1_packet_effect_type_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_packet_effect_type_min_order_by"
            }
          },
          {
            "name": "stddev",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_packet_effect_type_stddev_order_by"
            }
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_packet_effect_type_stddev_pop_order_by"
            }
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_packet_effect_type_stddev_samp_order_by"
            }
          },
          {
            "name": "sum",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_packet_effect_type_sum_order_by"
            }
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_packet_effect_type_var_pop_order_by"
            }
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_packet_effect_type_var_samp_order_by"
            }
          },
          {
            "name": "variance",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_packet_effect_type_variance_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_packet_effect_type_avg_order_by",
        "inputFields": [
          {
            "name": "event_index",
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
        "name": "v1_packet_effect_type_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_packet_effect_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_packet_effect_type_bool_exp"
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
                  "name": "v1_packet_effect_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "event",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_packet_effect_type_max_order_by",
        "inputFields": [
          {
            "name": "event_index",
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
        "name": "v1_packet_effect_type_min_order_by",
        "inputFields": [
          {
            "name": "event_index",
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
        "name": "v1_packet_effect_type_order_by",
        "inputFields": [
          {
            "name": "event",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "event_index",
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
        "name": "v1_packet_effect_type_select_column",
        "enumValues": [
          {
            "name": "event",
            "isDeprecated": false
          },
          {
            "name": "event_index",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_packet_effect_type_stddev_order_by",
        "inputFields": [
          {
            "name": "event_index",
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
        "name": "v1_packet_effect_type_stddev_pop_order_by",
        "inputFields": [
          {
            "name": "event_index",
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
        "name": "v1_packet_effect_type_stddev_samp_order_by",
        "inputFields": [
          {
            "name": "event_index",
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
        "name": "v1_packet_effect_type_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_packet_effect_type_stream_cursor_value_input"
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
        "name": "v1_packet_effect_type_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "event",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "event_index",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_packet_effect_type_sum_order_by",
        "inputFields": [
          {
            "name": "event_index",
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
        "name": "v1_packet_effect_type_var_pop_order_by",
        "inputFields": [
          {
            "name": "event_index",
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
        "name": "v1_packet_effect_type_var_samp_order_by",
        "inputFields": [
          {
            "name": "event_index",
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
        "name": "v1_packet_effect_type_variance_order_by",
        "inputFields": [
          {
            "name": "event_index",
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
        "name": "v1_packets",
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
            "name": "destination_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_event_json",
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
            "name": "destination_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_packet_data",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_port_id",
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
              "name": "bigint"
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
            "name": "destination_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "effects",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "OBJECT",
                  "name": "v1_packet_effect_type"
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
                      "name": "v1_packet_effect_type_select_column"
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
                      "name": "v1_packet_effect_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_packet_effect_type_bool_exp"
                }
              }
            ],
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
            "name": "source_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_event_json",
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
            "name": "source_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_packet_data",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_port_id",
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
              "name": "bigint"
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
            "name": "source_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "traces",
            "type": {
              "kind": "OBJECT",
              "name": "v1_traces"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transfer",
            "type": {
              "kind": "OBJECT",
              "name": "v1_transfers"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_packets_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_packets_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_packets_bool_exp"
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
                  "name": "v1_packets_bool_exp"
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
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_bool_exp"
            }
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "destination_event_json",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "destination_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "destination_packet_data",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_port_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_sequence",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
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
            "name": "destination_timestamp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "destination_transaction_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
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
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "effects",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_packet_effect_type_bool_exp"
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
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_bool_exp"
            }
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "source_event_json",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "source_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "source_packet_data",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_port_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_sequence",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
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
            "name": "source_timestamp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "source_transaction_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
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
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "traces",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_traces_bool_exp"
            }
          },
          {
            "name": "transfer",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_transfers_bool_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_packets_order_by",
        "inputFields": [
          {
            "name": "destination_block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_order_by"
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
            "name": "destination_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_event_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_event_json",
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
            "name": "destination_packet_data",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_port_id",
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
            "name": "destination_timeout_timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_transaction_event_index",
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
            "name": "effects_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_packet_effect_type_aggregate_order_by"
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
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_order_by"
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
            "name": "source_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_event_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_event_json",
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
            "name": "source_packet_data",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_port_id",
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
            "name": "source_timeout_timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_transaction_event_index",
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
            "name": "traces",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_traces_order_by"
            }
          },
          {
            "name": "transfer",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_transfers_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v1_packets_select_column",
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
            "name": "destination_channel_id",
            "isDeprecated": false
          },
          {
            "name": "destination_connection_id",
            "isDeprecated": false
          },
          {
            "name": "destination_event_index",
            "isDeprecated": false
          },
          {
            "name": "destination_event_json",
            "isDeprecated": false
          },
          {
            "name": "destination_height",
            "isDeprecated": false
          },
          {
            "name": "destination_packet_data",
            "isDeprecated": false
          },
          {
            "name": "destination_port_id",
            "isDeprecated": false
          },
          {
            "name": "destination_sequence",
            "isDeprecated": false
          },
          {
            "name": "destination_timeout_timestamp",
            "isDeprecated": false
          },
          {
            "name": "destination_timestamp",
            "isDeprecated": false
          },
          {
            "name": "destination_transaction_event_index",
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
            "name": "source_block_hash",
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
            "isDeprecated": false
          },
          {
            "name": "source_channel_id",
            "isDeprecated": false
          },
          {
            "name": "source_connection_id",
            "isDeprecated": false
          },
          {
            "name": "source_event_index",
            "isDeprecated": false
          },
          {
            "name": "source_event_json",
            "isDeprecated": false
          },
          {
            "name": "source_height",
            "isDeprecated": false
          },
          {
            "name": "source_packet_data",
            "isDeprecated": false
          },
          {
            "name": "source_port_id",
            "isDeprecated": false
          },
          {
            "name": "source_sequence",
            "isDeprecated": false
          },
          {
            "name": "source_timeout_timestamp",
            "isDeprecated": false
          },
          {
            "name": "source_timestamp",
            "isDeprecated": false
          },
          {
            "name": "source_transaction_event_index",
            "isDeprecated": false
          },
          {
            "name": "source_transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "source_transaction_index",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_packets_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_packets_stream_cursor_value_input"
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
        "name": "v1_packets_stream_cursor_value_input",
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
              "name": "String"
            }
          },
          {
            "name": "destination_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "destination_event_json",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "destination_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "destination_packet_data",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_sequence",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
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
            "name": "destination_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "destination_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
              "name": "Int"
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
              "name": "String"
            }
          },
          {
            "name": "source_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "source_event_json",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "source_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "source_packet_data",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_sequence",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
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
            "name": "source_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "source_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
              "name": "Int"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v1_rpcs",
        "fields": [
          {
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "contact",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "description",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "enabled",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "type",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "url",
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
        "name": "v1_rpcs_aggregate_order_by",
        "inputFields": [
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
              "name": "v1_rpcs_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_rpcs_min_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_rpcs_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_rpcs_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_rpcs_bool_exp"
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
                  "name": "v1_rpcs_bool_exp"
                }
              }
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_bool_exp"
            }
          },
          {
            "name": "contact",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "description",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "enabled",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Boolean_comparison_exp"
            }
          },
          {
            "name": "type",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "url",
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
        "name": "v1_rpcs_max_order_by",
        "inputFields": [
          {
            "name": "contact",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "description",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "type",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "url",
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
        "name": "v1_rpcs_min_order_by",
        "inputFields": [
          {
            "name": "contact",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "description",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "type",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "url",
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
        "name": "v1_rpcs_order_by",
        "inputFields": [
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_order_by"
            }
          },
          {
            "name": "contact",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "description",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "enabled",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "type",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "url",
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
        "name": "v1_rpcs_select_column",
        "enumValues": [
          {
            "name": "contact",
            "isDeprecated": false
          },
          {
            "name": "description",
            "isDeprecated": false
          },
          {
            "name": "enabled",
            "isDeprecated": false
          },
          {
            "name": "type",
            "isDeprecated": false
          },
          {
            "name": "url",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_rpcs_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_rpcs_stream_cursor_value_input"
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
        "name": "v1_rpcs_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "contact",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "description",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "enabled",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "type",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "url",
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
        "name": "v1_statistics",
        "fields": [
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
            "name": "value",
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
        "name": "v1_statistics_address",
        "fields": [
          {
            "name": "normalized_address",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "total",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_statistics_address_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_statistics_address_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_statistics_address_bool_exp"
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
                  "name": "v1_statistics_address_bool_exp"
                }
              }
            }
          },
          {
            "name": "normalized_address",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "total",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_statistics_address_order_by",
        "inputFields": [
          {
            "name": "normalized_address",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "total",
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
        "name": "v1_statistics_address_select_column",
        "enumValues": [
          {
            "name": "normalized_address",
            "isDeprecated": false
          },
          {
            "name": "total",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_statistics_address_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_statistics_address_stream_cursor_value_input"
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
        "name": "v1_statistics_address_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "normalized_address",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "total",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_statistics_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_statistics_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_statistics_bool_exp"
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
                  "name": "v1_statistics_bool_exp"
                }
              }
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
            "name": "value",
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
        "name": "v1_statistics_order_by",
        "inputFields": [
          {
            "name": "name",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "value",
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
        "name": "v1_statistics_select_column",
        "enumValues": [
          {
            "name": "name",
            "isDeprecated": false
          },
          {
            "name": "value",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_statistics_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_statistics_stream_cursor_value_input"
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
        "name": "v1_statistics_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "name",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "value",
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
        "name": "v1_tokens_type",
        "fields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "asset",
            "type": {
              "kind": "OBJECT",
              "name": "v1_assets"
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
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_tokens_type_aggregate_order_by",
        "inputFields": [
          {
            "name": "avg",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_tokens_type_avg_order_by"
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
              "name": "v1_tokens_type_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_tokens_type_min_order_by"
            }
          },
          {
            "name": "stddev",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_tokens_type_stddev_order_by"
            }
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_tokens_type_stddev_pop_order_by"
            }
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_tokens_type_stddev_samp_order_by"
            }
          },
          {
            "name": "sum",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_tokens_type_sum_order_by"
            }
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_tokens_type_var_pop_order_by"
            }
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_tokens_type_var_samp_order_by"
            }
          },
          {
            "name": "variance",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_tokens_type_variance_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_tokens_type_avg_order_by",
        "inputFields": [
          {
            "name": "amount",
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
        "name": "v1_tokens_type_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_tokens_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_tokens_type_bool_exp"
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
                  "name": "v1_tokens_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "amount",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "numeric_comparison_exp"
            }
          },
          {
            "name": "asset",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_assets_bool_exp"
            }
          },
          {
            "name": "denom",
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
        "name": "v1_tokens_type_max_order_by",
        "inputFields": [
          {
            "name": "amount",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_tokens_type_min_order_by",
        "inputFields": [
          {
            "name": "amount",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_tokens_type_order_by",
        "inputFields": [
          {
            "name": "amount",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "asset",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_assets_order_by"
            }
          },
          {
            "name": "denom",
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
        "name": "v1_tokens_type_select_column",
        "enumValues": [
          {
            "name": "amount",
            "isDeprecated": false
          },
          {
            "name": "denom",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_tokens_type_stddev_order_by",
        "inputFields": [
          {
            "name": "amount",
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
        "name": "v1_tokens_type_stddev_pop_order_by",
        "inputFields": [
          {
            "name": "amount",
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
        "name": "v1_tokens_type_stddev_samp_order_by",
        "inputFields": [
          {
            "name": "amount",
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
        "name": "v1_tokens_type_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_tokens_type_stream_cursor_value_input"
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
        "name": "v1_tokens_type_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "amount",
            "type": {
              "kind": "SCALAR",
              "name": "numeric"
            }
          },
          {
            "name": "denom",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_tokens_type_sum_order_by",
        "inputFields": [
          {
            "name": "amount",
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
        "name": "v1_tokens_type_var_pop_order_by",
        "inputFields": [
          {
            "name": "amount",
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
        "name": "v1_tokens_type_var_samp_order_by",
        "inputFields": [
          {
            "name": "amount",
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
        "name": "v1_tokens_type_variance_order_by",
        "inputFields": [
          {
            "name": "amount",
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
        "name": "v1_traces",
        "fields": [
          {
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
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
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "initiating_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "order",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_sequence",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
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
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transfer",
            "type": {
              "kind": "OBJECT",
              "name": "v1_transfers"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "type",
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
        "name": "v1_traces_aggregate_order_by",
        "inputFields": [
          {
            "name": "avg",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_traces_avg_order_by"
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
              "name": "v1_traces_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_traces_min_order_by"
            }
          },
          {
            "name": "stddev",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_traces_stddev_order_by"
            }
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_traces_stddev_pop_order_by"
            }
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_traces_stddev_samp_order_by"
            }
          },
          {
            "name": "sum",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_traces_sum_order_by"
            }
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_traces_var_pop_order_by"
            }
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_traces_var_samp_order_by"
            }
          },
          {
            "name": "variance",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_traces_variance_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_traces_avg_order_by",
        "inputFields": [
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "order",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_traces_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_traces_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_traces_bool_exp"
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
                  "name": "v1_traces_bool_exp"
                }
              }
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_bool_exp"
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
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "initiating_transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "order",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "source_sequence",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
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
            "name": "transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "transfer",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_transfers_bool_exp"
            }
          },
          {
            "name": "type",
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
        "name": "v1_traces_max_order_by",
        "inputFields": [
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "initiating_transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "order",
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
            "name": "timestamp",
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
            "name": "type",
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
        "name": "v1_traces_min_order_by",
        "inputFields": [
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "initiating_transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "order",
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
            "name": "timestamp",
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
            "name": "type",
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
        "name": "v1_traces_order_by",
        "inputFields": [
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_order_by"
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
            "name": "initiating_transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "order",
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
            "name": "timestamp",
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
            "name": "transfer",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_transfers_order_by"
            }
          },
          {
            "name": "type",
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
        "name": "v1_traces_select_column",
        "enumValues": [
          {
            "name": "data",
            "isDeprecated": false
          },
          {
            "name": "height",
            "isDeprecated": false
          },
          {
            "name": "initiating_transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "order",
            "isDeprecated": false
          },
          {
            "name": "source_sequence",
            "isDeprecated": false
          },
          {
            "name": "timestamp",
            "isDeprecated": false
          },
          {
            "name": "transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "type",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_traces_stddev_order_by",
        "inputFields": [
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "order",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_traces_stddev_pop_order_by",
        "inputFields": [
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "order",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_traces_stddev_samp_order_by",
        "inputFields": [
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "order",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_traces_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_traces_stream_cursor_value_input"
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
        "name": "v1_traces_stream_cursor_value_input",
        "inputFields": [
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
              "name": "bigint"
            }
          },
          {
            "name": "initiating_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "order",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "source_sequence",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
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
            "name": "transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "type",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_traces_sum_order_by",
        "inputFields": [
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "order",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_traces_var_pop_order_by",
        "inputFields": [
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "order",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_traces_var_samp_order_by",
        "inputFields": [
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "order",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_traces_variance_order_by",
        "inputFields": [
          {
            "name": "height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "order",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v1_transfer_forwards_type",
        "fields": [
          {
            "name": "destination_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_port_id",
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
            "name": "retries",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
            "name": "source_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "timeout",
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
        "name": "v1_transfer_forwards_type_aggregate_order_by",
        "inputFields": [
          {
            "name": "avg",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_transfer_forwards_type_avg_order_by"
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
              "name": "v1_transfer_forwards_type_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_transfer_forwards_type_min_order_by"
            }
          },
          {
            "name": "stddev",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_transfer_forwards_type_stddev_order_by"
            }
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_transfer_forwards_type_stddev_pop_order_by"
            }
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_transfer_forwards_type_stddev_samp_order_by"
            }
          },
          {
            "name": "sum",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_transfer_forwards_type_sum_order_by"
            }
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_transfer_forwards_type_var_pop_order_by"
            }
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_transfer_forwards_type_var_samp_order_by"
            }
          },
          {
            "name": "variance",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_transfer_forwards_type_variance_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_transfer_forwards_type_avg_order_by",
        "inputFields": [
          {
            "name": "retries",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_event_index",
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
            "name": "source_transaction_event_index",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_transfer_forwards_type_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_transfer_forwards_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_transfer_forwards_type_bool_exp"
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
                  "name": "v1_transfer_forwards_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_bool_exp"
            }
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_port_id",
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
            "name": "retries",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
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
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_bool_exp"
            }
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "source_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "source_port_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_transaction_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
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
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "timeout",
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
        "name": "v1_transfer_forwards_type_max_order_by",
        "inputFields": [
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_port_id",
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
            "name": "retries",
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
            "name": "source_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_event_index",
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
            "name": "source_port_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_transaction_event_index",
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
            "name": "timeout",
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
        "name": "v1_transfer_forwards_type_min_order_by",
        "inputFields": [
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_port_id",
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
            "name": "retries",
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
            "name": "source_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_event_index",
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
            "name": "source_port_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_transaction_event_index",
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
            "name": "timeout",
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
        "name": "v1_transfer_forwards_type_order_by",
        "inputFields": [
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_order_by"
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
            "name": "destination_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_port_id",
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
            "name": "retries",
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
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_order_by"
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
            "name": "source_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_event_index",
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
            "name": "source_port_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_transaction_event_index",
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
            "name": "timeout",
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
        "name": "v1_transfer_forwards_type_select_column",
        "enumValues": [
          {
            "name": "destination_chain_id",
            "isDeprecated": false
          },
          {
            "name": "destination_channel_id",
            "isDeprecated": false
          },
          {
            "name": "destination_connection_id",
            "isDeprecated": false
          },
          {
            "name": "destination_port_id",
            "isDeprecated": false
          },
          {
            "name": "receiver",
            "isDeprecated": false
          },
          {
            "name": "retries",
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
            "name": "source_channel_id",
            "isDeprecated": false
          },
          {
            "name": "source_connection_id",
            "isDeprecated": false
          },
          {
            "name": "source_event_index",
            "isDeprecated": false
          },
          {
            "name": "source_height",
            "isDeprecated": false
          },
          {
            "name": "source_port_id",
            "isDeprecated": false
          },
          {
            "name": "source_transaction_event_index",
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
            "name": "timeout",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_transfer_forwards_type_stddev_order_by",
        "inputFields": [
          {
            "name": "retries",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_event_index",
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
            "name": "source_transaction_event_index",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_transfer_forwards_type_stddev_pop_order_by",
        "inputFields": [
          {
            "name": "retries",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_event_index",
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
            "name": "source_transaction_event_index",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_transfer_forwards_type_stddev_samp_order_by",
        "inputFields": [
          {
            "name": "retries",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_event_index",
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
            "name": "source_transaction_event_index",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_transfer_forwards_type_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_transfer_forwards_type_stream_cursor_value_input"
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
        "name": "v1_transfer_forwards_type_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_port_id",
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
            "name": "retries",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
              "name": "String"
            }
          },
          {
            "name": "source_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "source_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "source_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
              "name": "Int"
            }
          },
          {
            "name": "timeout",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_transfer_forwards_type_sum_order_by",
        "inputFields": [
          {
            "name": "retries",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_event_index",
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
            "name": "source_transaction_event_index",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_transfer_forwards_type_var_pop_order_by",
        "inputFields": [
          {
            "name": "retries",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_event_index",
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
            "name": "source_transaction_event_index",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_transfer_forwards_type_var_samp_order_by",
        "inputFields": [
          {
            "name": "retries",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_event_index",
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
            "name": "source_transaction_event_index",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_transfer_forwards_type_variance_order_by",
        "inputFields": [
          {
            "name": "retries",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_event_index",
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
            "name": "source_transaction_event_index",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v1_transfers",
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
            "name": "destination_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_port_id",
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
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "forwards",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "OBJECT",
                  "name": "v1_transfer_forwards_type"
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
                      "name": "v1_transfer_forwards_type_select_column"
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
                      "name": "v1_transfer_forwards_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_transfer_forwards_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "hop",
            "type": {
              "kind": "OBJECT",
              "name": "v1_transfers"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "normalized_receiver",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "normalized_sender",
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
              "kind": "OBJECT",
              "name": "v1_packets"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_data",
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
            "name": "pfm_destination_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "pfm_destination_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "pfm_recv_sequence",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "pfm_sent_sequence",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "pfm_source_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "pfm_source_port_id",
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
            "name": "source_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_port_id",
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
              "name": "bigint"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "tokens",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "OBJECT",
                  "name": "v1_tokens_type"
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
                      "name": "v1_tokens_type_select_column"
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
                      "name": "v1_tokens_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_tokens_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "traces",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_traces"
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
                      "name": "v1_traces_select_column"
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
                      "name": "v1_traces_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_traces_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_transfers_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_transfers_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_transfers_bool_exp"
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
                  "name": "v1_transfers_bool_exp"
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
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_bool_exp"
            }
          },
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "destination_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "destination_port_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_sequence",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "destination_timestamp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "destination_transaction_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
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
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "forwards",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_transfer_forwards_type_bool_exp"
            }
          },
          {
            "name": "hop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_transfers_bool_exp"
            }
          },
          {
            "name": "normalized_receiver",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "normalized_sender",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_packets_bool_exp"
            }
          },
          {
            "name": "packet_data",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "pfm_destination_channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "pfm_destination_port_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "pfm_recv_sequence",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "pfm_sent_sequence",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "pfm_source_channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "pfm_source_port_id",
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
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_bool_exp"
            }
          },
          {
            "name": "source_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "source_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "source_port_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_sequence",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "bigint_comparison_exp"
            }
          },
          {
            "name": "source_timestamp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "source_transaction_event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
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
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "tokens",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_tokens_type_bool_exp"
            }
          },
          {
            "name": "traces",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_traces_bool_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_transfers_order_by",
        "inputFields": [
          {
            "name": "destination_block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_order_by"
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
            "name": "destination_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_event_index",
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
            "name": "destination_port_id",
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
            "name": "destination_timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_transaction_event_index",
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
            "name": "forwards_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_transfer_forwards_type_aggregate_order_by"
            }
          },
          {
            "name": "hop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_transfers_order_by"
            }
          },
          {
            "name": "normalized_receiver",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "normalized_sender",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_packets_order_by"
            }
          },
          {
            "name": "packet_data",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "pfm_destination_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "pfm_destination_port_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "pfm_recv_sequence",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "pfm_sent_sequence",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "pfm_source_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "pfm_source_port_id",
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
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_order_by"
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
            "name": "source_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_event_index",
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
            "name": "source_port_id",
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
            "name": "source_timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_transaction_event_index",
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
            "name": "tokens_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_tokens_type_aggregate_order_by"
            }
          },
          {
            "name": "traces_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_traces_aggregate_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v1_transfers_select_column",
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
            "name": "destination_channel_id",
            "isDeprecated": false
          },
          {
            "name": "destination_connection_id",
            "isDeprecated": false
          },
          {
            "name": "destination_event_index",
            "isDeprecated": false
          },
          {
            "name": "destination_height",
            "isDeprecated": false
          },
          {
            "name": "destination_port_id",
            "isDeprecated": false
          },
          {
            "name": "destination_sequence",
            "isDeprecated": false
          },
          {
            "name": "destination_timestamp",
            "isDeprecated": false
          },
          {
            "name": "destination_transaction_event_index",
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
            "name": "normalized_receiver",
            "isDeprecated": false
          },
          {
            "name": "normalized_sender",
            "isDeprecated": false
          },
          {
            "name": "packet_data",
            "isDeprecated": false
          },
          {
            "name": "pfm_destination_channel_id",
            "isDeprecated": false
          },
          {
            "name": "pfm_destination_port_id",
            "isDeprecated": false
          },
          {
            "name": "pfm_recv_sequence",
            "isDeprecated": false
          },
          {
            "name": "pfm_sent_sequence",
            "isDeprecated": false
          },
          {
            "name": "pfm_source_channel_id",
            "isDeprecated": false
          },
          {
            "name": "pfm_source_port_id",
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
            "name": "source_channel_id",
            "isDeprecated": false
          },
          {
            "name": "source_connection_id",
            "isDeprecated": false
          },
          {
            "name": "source_event_index",
            "isDeprecated": false
          },
          {
            "name": "source_height",
            "isDeprecated": false
          },
          {
            "name": "source_port_id",
            "isDeprecated": false
          },
          {
            "name": "source_sequence",
            "isDeprecated": false
          },
          {
            "name": "source_timestamp",
            "isDeprecated": false
          },
          {
            "name": "source_transaction_event_index",
            "isDeprecated": false
          },
          {
            "name": "source_transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "source_transaction_index",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_transfers_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_transfers_stream_cursor_value_input"
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
        "name": "v1_transfers_stream_cursor_value_input",
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
              "name": "String"
            }
          },
          {
            "name": "destination_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "destination_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "destination_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "destination_sequence",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "destination_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "destination_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
              "name": "Int"
            }
          },
          {
            "name": "normalized_receiver",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "normalized_sender",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "packet_data",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "pfm_destination_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "pfm_destination_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "pfm_recv_sequence",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "pfm_sent_sequence",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "pfm_source_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "pfm_source_port_id",
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
              "name": "String"
            }
          },
          {
            "name": "source_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "source_height",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "source_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "source_sequence",
            "type": {
              "kind": "SCALAR",
              "name": "bigint"
            }
          },
          {
            "name": "source_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "source_transaction_event_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
              "name": "Int"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v1_ucs1_configurations",
        "fields": [
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
            "name": "contract_address",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "forwards",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v1_ucs1_configurations"
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
                      "name": "v1_ucs1_configurations_select_column"
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
                      "name": "v1_ucs1_configurations_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ucs1_configurations_bool_exp"
                }
              }
            ],
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
            "name": "source_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ucs1_configurations_aggregate_order_by",
        "inputFields": [
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
              "name": "v1_ucs1_configurations_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ucs1_configurations_min_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ucs1_configurations_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ucs1_configurations_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ucs1_configurations_bool_exp"
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
                  "name": "v1_ucs1_configurations_bool_exp"
                }
              }
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
            "name": "contract_address",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_bool_exp"
            }
          },
          {
            "name": "forwards",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ucs1_configurations_bool_exp"
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
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_bool_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ucs1_configurations_max_order_by",
        "inputFields": [
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
            "name": "contract_address",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ucs1_configurations_min_order_by",
        "inputFields": [
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
            "name": "contract_address",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ucs1_configurations_order_by",
        "inputFields": [
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
            "name": "contract_address",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_order_by"
            }
          },
          {
            "name": "forwards_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ucs1_configurations_aggregate_order_by"
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
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v1_ucs1_configurations_select_column",
        "enumValues": [
          {
            "name": "channel_id",
            "isDeprecated": false
          },
          {
            "name": "connection_id",
            "isDeprecated": false
          },
          {
            "name": "contract_address",
            "isDeprecated": false
          },
          {
            "name": "port_id",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ucs1_configurations_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_ucs1_configurations_stream_cursor_value_input"
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
        "name": "v1_ucs1_configurations_stream_cursor_value_input",
        "inputFields": [
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
            "name": "contract_address",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "port_id",
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
        "name": "v1_ucs1_paths",
        "fields": [
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
            "name": "destination_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v1_chains"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ucs1_paths_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v1_ucs1_paths_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_ucs1_paths_bool_exp"
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
                  "name": "v1_ucs1_paths_bool_exp"
                }
              }
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
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_bool_exp"
            }
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_bool_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ucs1_paths_order_by",
        "inputFields": [
          {
            "name": "channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_order_by"
            }
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v1_chains_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v1_ucs1_paths_select_column",
        "enumValues": [
          {
            "name": "channel_id",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v1_ucs1_paths_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v1_ucs1_paths_stream_cursor_value_input"
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
        "name": "v1_ucs1_paths_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "channel_id",
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