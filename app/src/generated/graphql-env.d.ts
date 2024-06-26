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
        "name": "Address"
      },
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
        "kind": "OBJECT",
        "name": "Configuration",
        "fields": [
          {
            "name": "amountSend",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Long"
              }
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "chainId",
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
            "name": "denom",
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
            "name": "feeAmount",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Long"
              }
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "gasLimit",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "UInt64"
              }
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "memo",
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
            "name": "prefix",
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
        "name": "Long"
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "SendInput",
        "inputFields": [
          {
            "name": "captchaToken",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "toAddress",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Address"
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
        "name": "UInt64"
      },
      {
        "kind": "SCALAR",
        "name": "Void"
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
        "kind": "OBJECT",
        "name": "faucetMutation",
        "fields": [
          {
            "name": "send",
            "type": {
              "kind": "SCALAR",
              "name": "Void"
            },
            "args": [
              {
                "name": "input",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "INPUT_OBJECT",
                    "name": "SendInput"
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
        "name": "faucetQuery",
        "fields": [
          {
            "name": "configuration",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "Configuration"
              }
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "SCALAR",
        "name": "json"
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
            "name": "faucet",
            "type": {
              "kind": "OBJECT",
              "name": "faucetMutation"
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
            "name": "faucet",
            "type": {
              "kind": "OBJECT",
              "name": "faucetQuery"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "v0_assets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_assets"
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
                      "name": "v0_assets_select_column"
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
                      "name": "v0_assets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_assets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_assets_by_pk",
            "type": {
              "kind": "OBJECT",
              "name": "v0_assets"
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
                "name": "denom",
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
            "name": "v0_channels",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_channels"
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
                      "name": "v0_channels_select_column"
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
                      "name": "v0_channels_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_channels_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_channels_aggregate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_channels_aggregate"
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
                      "name": "v0_channels_select_column"
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
                      "name": "v0_channels_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_channels_bool_exp"
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
            "name": "v0_connections",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_connections"
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
                      "name": "v0_connections_select_column"
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
                      "name": "v0_connections_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_connections_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_explorers",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_explorers"
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
                      "name": "v0_explorers_select_column"
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
                      "name": "v0_explorers_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_explorers_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_explorers_by_pk",
            "type": {
              "kind": "OBJECT",
              "name": "v0_explorers"
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
            "name": "v0_faucets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_faucets"
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
                      "name": "v0_faucets_select_column"
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
                      "name": "v0_faucets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_faucets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_get_transfer_forwards",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_transfer_forwards"
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
                    "name": "v0_get_transfer_forwards_args"
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
                      "name": "v0_transfer_forwards_select_column"
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
                      "name": "v0_transfer_forwards_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_transfer_forwards_bool_exp"
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
            "name": "v0_rpcs",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_rpcs"
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
                      "name": "v0_rpcs_select_column"
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
                      "name": "v0_rpcs_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_rpcs_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_rpcs_by_pk",
            "type": {
              "kind": "OBJECT",
              "name": "v0_rpcs"
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
                "name": "url",
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
            "name": "v0_traces",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_traces"
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
                      "name": "v0_traces_select_column"
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
                      "name": "v0_traces_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_traces_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_transactions",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_transactions"
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
                      "name": "v0_transactions_select_column"
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
                      "name": "v0_transactions_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_transactions_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_transactions_by_pk",
            "type": {
              "kind": "OBJECT",
              "name": "v0_transactions"
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
            "name": "v0_transfer_forwards",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_transfer_forwards"
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
                      "name": "v0_transfer_forwards_select_column"
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
                      "name": "v0_transfer_forwards_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_transfer_forwards_bool_exp"
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
            "name": "v0_ucs1_configuration",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_ucs1_configuration"
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
                      "name": "v0_ucs1_configuration_select_column"
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
                      "name": "v0_ucs1_configuration_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_ucs1_configuration_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_ucs1_configuration_by_pk",
            "type": {
              "kind": "OBJECT",
              "name": "v0_ucs1_configuration"
            },
            "args": [
              {
                "name": "destination_chain_id",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "Int"
                  }
                }
              },
              {
                "name": "source_chain_id",
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
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "status_v0_transactions_args",
        "inputFields": [
          {
            "name": "hasura_session",
            "type": {
              "kind": "SCALAR",
              "name": "json"
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
            "name": "v0_assets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_assets"
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
                      "name": "v0_assets_select_column"
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
                      "name": "v0_assets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_assets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_assets_by_pk",
            "type": {
              "kind": "OBJECT",
              "name": "v0_assets"
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
                "name": "denom",
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
            "name": "v0_assets_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_assets"
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
                      "name": "v0_assets_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_assets_bool_exp"
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
            "name": "v0_chains_stream",
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
                      "name": "v0_chains_stream_cursor_input"
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
            "name": "v0_channel_map_stream",
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
                      "name": "v0_channel_map_stream_cursor_input"
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
            "name": "v0_channels",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_channels"
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
                      "name": "v0_channels_select_column"
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
                      "name": "v0_channels_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_channels_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_channels_aggregate",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v0_channels_aggregate"
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
                      "name": "v0_channels_select_column"
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
                      "name": "v0_channels_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_channels_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_channels_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_channels"
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
                      "name": "v0_channels_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_channels_bool_exp"
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
            "name": "v0_connection_map_stream",
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
                      "name": "v0_connection_map_stream_cursor_input"
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
            "name": "v0_connections",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_connections"
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
                      "name": "v0_connections_select_column"
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
                      "name": "v0_connections_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_connections_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_connections_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_connections"
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
                      "name": "v0_connections_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_connections_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_explorers",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_explorers"
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
                      "name": "v0_explorers_select_column"
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
                      "name": "v0_explorers_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_explorers_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_explorers_by_pk",
            "type": {
              "kind": "OBJECT",
              "name": "v0_explorers"
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
            "name": "v0_explorers_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_explorers"
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
                      "name": "v0_explorers_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_explorers_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_faucets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_faucets"
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
                      "name": "v0_faucets_select_column"
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
                      "name": "v0_faucets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_faucets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_faucets_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_faucets"
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
                      "name": "v0_faucets_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_faucets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_get_transfer_forwards",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_transfer_forwards"
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
                    "name": "v0_get_transfer_forwards_args"
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
                      "name": "v0_transfer_forwards_select_column"
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
                      "name": "v0_transfer_forwards_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_transfer_forwards_bool_exp"
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
            "name": "v0_rpcs",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_rpcs"
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
                      "name": "v0_rpcs_select_column"
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
                      "name": "v0_rpcs_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_rpcs_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_rpcs_by_pk",
            "type": {
              "kind": "OBJECT",
              "name": "v0_rpcs"
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
                "name": "url",
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
            "name": "v0_rpcs_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_rpcs"
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
                      "name": "v0_rpcs_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_rpcs_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_traces",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_traces"
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
                      "name": "v0_traces_select_column"
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
                      "name": "v0_traces_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_traces_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_traces_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_traces"
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
                      "name": "v0_traces_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_traces_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_transactions",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_transactions"
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
                      "name": "v0_transactions_select_column"
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
                      "name": "v0_transactions_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_transactions_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_transactions_by_pk",
            "type": {
              "kind": "OBJECT",
              "name": "v0_transactions"
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
            "name": "v0_transactions_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_transactions"
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
                      "name": "v0_transactions_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_transactions_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_transfer_forwards",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_transfer_forwards"
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
                      "name": "v0_transfer_forwards_select_column"
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
                      "name": "v0_transfer_forwards_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_transfer_forwards_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_transfer_forwards_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_transfer_forwards"
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
                      "name": "v0_transfer_forwards_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_transfer_forwards_bool_exp"
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
          },
          {
            "name": "v0_ucs1_configuration",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_ucs1_configuration"
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
                      "name": "v0_ucs1_configuration_select_column"
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
                      "name": "v0_ucs1_configuration_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_ucs1_configuration_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v0_ucs1_configuration_by_pk",
            "type": {
              "kind": "OBJECT",
              "name": "v0_ucs1_configuration"
            },
            "args": [
              {
                "name": "destination_chain_id",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "Int"
                  }
                }
              },
              {
                "name": "source_chain_id",
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
            "name": "v0_ucs1_configuration_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_ucs1_configuration"
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
                      "name": "v0_ucs1_configuration_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_ucs1_configuration_bool_exp"
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
        "kind": "SCALAR",
        "name": "transfers_scalar"
      },
      {
        "kind": "OBJECT",
        "name": "v0_assets",
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
                    "name": "v0_faucets"
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
                      "name": "v0_faucets_select_column"
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
                      "name": "v0_faucets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_faucets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "gas_token",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Boolean"
              }
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
        "name": "v0_assets_aggregate_order_by",
        "inputFields": [
          {
            "name": "avg",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_assets_avg_order_by"
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
              "name": "v0_assets_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_assets_min_order_by"
            }
          },
          {
            "name": "stddev",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_assets_stddev_order_by"
            }
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_assets_stddev_pop_order_by"
            }
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_assets_stddev_samp_order_by"
            }
          },
          {
            "name": "sum",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_assets_sum_order_by"
            }
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_assets_var_pop_order_by"
            }
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_assets_var_samp_order_by"
            }
          },
          {
            "name": "variance",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_assets_variance_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_assets_avg_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
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
        "name": "v0_assets_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_assets_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_assets_bool_exp"
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
                  "name": "v0_assets_bool_exp"
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
              "name": "v0_faucets_bool_exp"
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
        "name": "v0_assets_max_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
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
        "name": "v0_assets_min_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
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
        "name": "v0_assets_order_by",
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
              "name": "v0_faucets_aggregate_order_by"
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
        "name": "v0_assets_select_column",
        "enumValues": [
          {
            "name": "chain_id",
            "isDeprecated": false
          },
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
        "name": "v0_assets_stddev_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
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
        "name": "v0_assets_stddev_pop_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
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
        "name": "v0_assets_stddev_samp_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
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
        "name": "v0_assets_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v0_assets_stream_cursor_value_input"
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
        "name": "v0_assets_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
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
        "name": "v0_assets_sum_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
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
        "name": "v0_assets_var_pop_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
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
        "name": "v0_assets_var_samp_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
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
        "name": "v0_assets_variance_order_by",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
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
          },
          {
            "name": "transactions",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_transactions"
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
                      "name": "v0_transactions_select_column"
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
                      "name": "v0_transactions_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_transactions_bool_exp"
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
          },
          {
            "name": "transactions",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transactions_bool_exp"
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
          },
          {
            "name": "transactions_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transactions_aggregate_order_by"
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
                    "name": "v0_assets"
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
                      "name": "v0_assets_select_column"
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
                      "name": "v0_assets_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_assets_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
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
            "name": "enabled",
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
                    "name": "v0_explorers"
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
                      "name": "v0_explorers_select_column"
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
                      "name": "v0_explorers_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_explorers_bool_exp"
                }
              }
            ],
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
                    "name": "v0_rpcs"
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
                      "name": "v0_rpcs_select_column"
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
                      "name": "v0_rpcs_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_rpcs_bool_exp"
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
            "name": "transactions",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_transactions"
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
                      "name": "v0_transactions_select_column"
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
                      "name": "v0_transactions_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_transactions_bool_exp"
                }
              }
            ],
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
                    "name": "v0_ucs1_configuration"
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
                      "name": "v0_ucs1_configuration_select_column"
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
                      "name": "v0_ucs1_configuration_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_ucs1_configuration_bool_exp"
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
              "name": "v0_assets_bool_exp"
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
            "name": "enabled",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Boolean_comparison_exp"
            }
          },
          {
            "name": "explorers",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_explorers_bool_exp"
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
              "name": "v0_rpcs_bool_exp"
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
            "name": "transactions",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transactions_bool_exp"
            }
          },
          {
            "name": "ucs1_configurations",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_ucs1_configuration_bool_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_chains_order_by",
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
              "name": "v0_assets_aggregate_order_by"
            }
          },
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
            "name": "enabled",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "explorers_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_explorers_aggregate_order_by"
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
              "name": "v0_rpcs_aggregate_order_by"
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
            "name": "transactions_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transactions_aggregate_order_by"
            }
          },
          {
            "name": "ucs1_configurations_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_ucs1_configuration_aggregate_order_by"
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
            "name": "id",
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
        "name": "v0_chains_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v0_chains_stream_cursor_value_input"
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
        "name": "v0_chains_stream_cursor_value_input",
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
            "name": "id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
        "kind": "INPUT_OBJECT",
        "name": "v0_channel_map_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v0_channel_map_stream_cursor_value_input"
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
        "name": "v0_channel_map_stream_cursor_value_input",
        "inputFields": [
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
        "name": "v0_channels",
        "fields": [
          {
            "name": "destination_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v0_chains"
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
              "name": "v0_chains"
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
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_channels_aggregate",
        "fields": [
          {
            "name": "aggregate",
            "type": {
              "kind": "OBJECT",
              "name": "v0_channels_aggregate_fields"
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
                    "name": "v0_channels"
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
        "name": "v0_channels_aggregate_fields",
        "fields": [
          {
            "name": "avg",
            "type": {
              "kind": "OBJECT",
              "name": "v0_channels_avg_fields"
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
                      "name": "v0_channels_select_column"
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
              "name": "v0_channels_max_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "min",
            "type": {
              "kind": "OBJECT",
              "name": "v0_channels_min_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev",
            "type": {
              "kind": "OBJECT",
              "name": "v0_channels_stddev_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "OBJECT",
              "name": "v0_channels_stddev_pop_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "OBJECT",
              "name": "v0_channels_stddev_samp_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sum",
            "type": {
              "kind": "OBJECT",
              "name": "v0_channels_sum_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "OBJECT",
              "name": "v0_channels_var_pop_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "OBJECT",
              "name": "v0_channels_var_samp_fields"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "variance",
            "type": {
              "kind": "OBJECT",
              "name": "v0_channels_variance_fields"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_channels_avg_fields",
        "fields": [
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
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
        "name": "v0_channels_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_channels_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_channels_bool_exp"
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
                  "name": "v0_channels_bool_exp"
                }
              }
            }
          },
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_chains_bool_exp"
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
              "name": "v0_chains_bool_exp"
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_channels_max_fields",
        "fields": [
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
            "name": "source_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v0_channels_min_fields",
        "fields": [
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
            "name": "source_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_channels_order_by",
        "inputFields": [
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_chains_order_by"
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
              "name": "v0_chains_order_by"
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v0_channels_select_column",
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
          }
        ]
      },
      {
        "kind": "OBJECT",
        "name": "v0_channels_stddev_fields",
        "fields": [
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
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
        "name": "v0_channels_stddev_pop_fields",
        "fields": [
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
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
        "name": "v0_channels_stddev_samp_fields",
        "fields": [
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
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
        "name": "v0_channels_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v0_channels_stream_cursor_value_input"
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
        "name": "v0_channels_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
              "name": "Int"
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_channels_sum_fields",
        "fields": [
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
            "name": "source_chain_id",
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
        "name": "v0_channels_var_pop_fields",
        "fields": [
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
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
        "name": "v0_channels_var_samp_fields",
        "fields": [
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
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
        "name": "v0_channels_variance_fields",
        "fields": [
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Float"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
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
        "kind": "INPUT_OBJECT",
        "name": "v0_connection_map_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v0_connection_map_stream_cursor_value_input"
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
        "name": "v0_connection_map_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "from_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "from_client_id",
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
            "name": "to_client_id",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_connections",
        "fields": [
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
            "name": "source_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
        "name": "v0_connections_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_connections_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_connections_bool_exp"
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
                  "name": "v0_connections_bool_exp"
                }
              }
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
            "name": "source_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
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
        "name": "v0_connections_order_by",
        "inputFields": [
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
        "name": "v0_connections_select_column",
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
        "name": "v0_connections_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v0_connections_stream_cursor_value_input"
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
        "name": "v0_connections_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
              "name": "Int"
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
        "name": "v0_explorers",
        "fields": [
          {
            "name": "address_url",
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
            "name": "block_url",
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
            "name": "description",
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
            "name": "home_url",
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
            "name": "name",
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
            "name": "tx_url",
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
        "kind": "INPUT_OBJECT",
        "name": "v0_explorers_aggregate_order_by",
        "inputFields": [
          {
            "name": "avg",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_explorers_avg_order_by"
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
              "name": "v0_explorers_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_explorers_min_order_by"
            }
          },
          {
            "name": "stddev",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_explorers_stddev_order_by"
            }
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_explorers_stddev_pop_order_by"
            }
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_explorers_stddev_samp_order_by"
            }
          },
          {
            "name": "sum",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_explorers_sum_order_by"
            }
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_explorers_var_pop_order_by"
            }
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_explorers_var_samp_order_by"
            }
          },
          {
            "name": "variance",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_explorers_variance_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_explorers_avg_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_explorers_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_explorers_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_explorers_bool_exp"
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
                  "name": "v0_explorers_bool_exp"
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
            "name": "id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
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
        "name": "v0_explorers_max_order_by",
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
            "name": "chain_id",
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
            "name": "id",
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
        "name": "v0_explorers_min_order_by",
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
            "name": "chain_id",
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
            "name": "id",
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
        "name": "v0_explorers_order_by",
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
            "name": "id",
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
        "name": "v0_explorers_select_column",
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
            "name": "chain_id",
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
            "name": "id",
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
        "name": "v0_explorers_stddev_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_explorers_stddev_pop_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_explorers_stddev_samp_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_explorers_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v0_explorers_stream_cursor_value_input"
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
        "name": "v0_explorers_stream_cursor_value_input",
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
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
            "name": "id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
        "kind": "INPUT_OBJECT",
        "name": "v0_explorers_sum_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_explorers_var_pop_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_explorers_var_samp_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_explorers_variance_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_faucets",
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
            "name": "denom",
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
            "name": "enabled",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Boolean"
              }
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "url",
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
        "kind": "INPUT_OBJECT",
        "name": "v0_faucets_aggregate_order_by",
        "inputFields": [
          {
            "name": "avg",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_faucets_avg_order_by"
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
              "name": "v0_faucets_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_faucets_min_order_by"
            }
          },
          {
            "name": "stddev",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_faucets_stddev_order_by"
            }
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_faucets_stddev_pop_order_by"
            }
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_faucets_stddev_samp_order_by"
            }
          },
          {
            "name": "sum",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_faucets_sum_order_by"
            }
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_faucets_var_pop_order_by"
            }
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_faucets_var_samp_order_by"
            }
          },
          {
            "name": "variance",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_faucets_variance_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_faucets_avg_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
        "name": "v0_faucets_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_faucets_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_faucets_bool_exp"
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
                  "name": "v0_faucets_bool_exp"
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
        "name": "v0_faucets_max_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
        "name": "v0_faucets_min_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
        "name": "v0_faucets_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
        "name": "v0_faucets_select_column",
        "enumValues": [
          {
            "name": "chain_id",
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
        "name": "v0_faucets_stddev_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
        "name": "v0_faucets_stddev_pop_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
        "name": "v0_faucets_stddev_samp_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
        "name": "v0_faucets_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v0_faucets_stream_cursor_value_input"
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
        "name": "v0_faucets_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "chain_id",
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
        "kind": "INPUT_OBJECT",
        "name": "v0_faucets_sum_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
        "name": "v0_faucets_var_pop_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
        "name": "v0_faucets_var_samp_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
        "name": "v0_faucets_variance_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
        "name": "v0_get_transfer_forwards_args",
        "inputFields": [
          {
            "name": "transfer",
            "type": {
              "kind": "SCALAR",
              "name": "transfers_scalar"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_index_status",
        "fields": [
          {
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v0_chains"
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
            "name": "destination_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v0_chains"
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
              "name": "v0_chains"
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
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_chains_bool_exp"
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
              "name": "v0_chains_bool_exp"
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
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_chains_order_by"
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
              "name": "v0_chains_order_by"
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v0_rpcs",
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
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Boolean"
              }
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "type",
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
            "name": "url",
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
        "kind": "INPUT_OBJECT",
        "name": "v0_rpcs_aggregate_order_by",
        "inputFields": [
          {
            "name": "avg",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_rpcs_avg_order_by"
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
              "name": "v0_rpcs_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_rpcs_min_order_by"
            }
          },
          {
            "name": "stddev",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_rpcs_stddev_order_by"
            }
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_rpcs_stddev_pop_order_by"
            }
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_rpcs_stddev_samp_order_by"
            }
          },
          {
            "name": "sum",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_rpcs_sum_order_by"
            }
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_rpcs_var_pop_order_by"
            }
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_rpcs_var_samp_order_by"
            }
          },
          {
            "name": "variance",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_rpcs_variance_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_rpcs_avg_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
        "name": "v0_rpcs_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_rpcs_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_rpcs_bool_exp"
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
                  "name": "v0_rpcs_bool_exp"
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
        "name": "v0_rpcs_max_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
        "name": "v0_rpcs_min_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
        "name": "v0_rpcs_order_by",
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
        "name": "v0_rpcs_select_column",
        "enumValues": [
          {
            "name": "chain_id",
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
        "name": "v0_rpcs_stddev_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
        "name": "v0_rpcs_stddev_pop_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
        "name": "v0_rpcs_stddev_samp_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
        "name": "v0_rpcs_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v0_rpcs_stream_cursor_value_input"
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
        "name": "v0_rpcs_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
        "kind": "INPUT_OBJECT",
        "name": "v0_rpcs_sum_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
        "name": "v0_rpcs_var_pop_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
        "name": "v0_rpcs_var_samp_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
        "name": "v0_rpcs_variance_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
        "name": "v0_traces",
        "fields": [
          {
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v0_chains"
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
            "name": "transfer",
            "type": {
              "kind": "OBJECT",
              "name": "v0_transfers"
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
        "name": "v0_traces_aggregate_order_by",
        "inputFields": [
          {
            "name": "avg",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_traces_avg_order_by"
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
              "name": "v0_traces_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_traces_min_order_by"
            }
          },
          {
            "name": "stddev",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_traces_stddev_order_by"
            }
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_traces_stddev_pop_order_by"
            }
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_traces_stddev_samp_order_by"
            }
          },
          {
            "name": "sum",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_traces_sum_order_by"
            }
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_traces_var_pop_order_by"
            }
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_traces_var_samp_order_by"
            }
          },
          {
            "name": "variance",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_traces_variance_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_traces_avg_order_by",
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
        "name": "v0_traces_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_traces_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_traces_bool_exp"
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
                  "name": "v0_traces_bool_exp"
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
            "name": "transfer",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transfers_bool_exp"
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
        "name": "v0_traces_max_order_by",
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
        "name": "v0_traces_min_order_by",
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
        "name": "v0_traces_order_by",
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
            "name": "transfer",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transfers_order_by"
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
        "name": "v0_traces_select_column",
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
        "name": "v0_traces_stddev_order_by",
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
        "name": "v0_traces_stddev_pop_order_by",
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
        "name": "v0_traces_stddev_samp_order_by",
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
        "name": "v0_traces_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v0_traces_stream_cursor_value_input"
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
        "name": "v0_traces_stream_cursor_value_input",
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
        "name": "v0_traces_sum_order_by",
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
        "name": "v0_traces_var_pop_order_by",
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
        "name": "v0_traces_var_samp_order_by",
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
        "name": "v0_traces_variance_order_by",
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
        "name": "v0_transactions",
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
            "name": "index",
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
            "name": "status",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [
              {
                "name": "args",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "INPUT_OBJECT",
                    "name": "status_v0_transactions_args"
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
        "kind": "INPUT_OBJECT",
        "name": "v0_transactions_aggregate_order_by",
        "inputFields": [
          {
            "name": "avg",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transactions_avg_order_by"
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
              "name": "v0_transactions_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transactions_min_order_by"
            }
          },
          {
            "name": "stddev",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transactions_stddev_order_by"
            }
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transactions_stddev_pop_order_by"
            }
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transactions_stddev_samp_order_by"
            }
          },
          {
            "name": "sum",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transactions_sum_order_by"
            }
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transactions_var_pop_order_by"
            }
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transactions_var_samp_order_by"
            }
          },
          {
            "name": "variance",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transactions_variance_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_transactions_avg_order_by",
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
          },
          {
            "name": "index",
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
        "name": "v0_transactions_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_transactions_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transactions_bool_exp"
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
                  "name": "v0_transactions_bool_exp"
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
            "name": "index",
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
        "name": "v0_transactions_max_order_by",
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
            "name": "index",
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
        "name": "v0_transactions_min_order_by",
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
            "name": "index",
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
        "name": "v0_transactions_order_by",
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
            "name": "index",
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
        "name": "v0_transactions_select_column",
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
            "name": "hash",
            "isDeprecated": false
          },
          {
            "name": "height",
            "isDeprecated": false
          },
          {
            "name": "index",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_transactions_stddev_order_by",
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
          },
          {
            "name": "index",
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
        "name": "v0_transactions_stddev_pop_order_by",
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
          },
          {
            "name": "index",
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
        "name": "v0_transactions_stddev_samp_order_by",
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
          },
          {
            "name": "index",
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
        "name": "v0_transactions_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v0_transactions_stream_cursor_value_input"
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
        "name": "v0_transactions_stream_cursor_value_input",
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
            "name": "index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_transactions_sum_order_by",
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
          },
          {
            "name": "index",
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
        "name": "v0_transactions_var_pop_order_by",
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
          },
          {
            "name": "index",
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
        "name": "v0_transactions_var_samp_order_by",
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
          },
          {
            "name": "index",
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
        "name": "v0_transactions_variance_order_by",
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
          },
          {
            "name": "index",
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
        "name": "v0_transfer_forwards",
        "fields": [
          {
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v0_chains"
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
            "name": "port",
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
            "name": "timeout",
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
        "name": "v0_transfer_forwards_aggregate_order_by",
        "inputFields": [
          {
            "name": "avg",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transfer_forwards_avg_order_by"
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
              "name": "v0_transfer_forwards_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transfer_forwards_min_order_by"
            }
          },
          {
            "name": "stddev",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transfer_forwards_stddev_order_by"
            }
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transfer_forwards_stddev_pop_order_by"
            }
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transfer_forwards_stddev_samp_order_by"
            }
          },
          {
            "name": "sum",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transfer_forwards_sum_order_by"
            }
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transfer_forwards_var_pop_order_by"
            }
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transfer_forwards_var_samp_order_by"
            }
          },
          {
            "name": "variance",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transfer_forwards_variance_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_transfer_forwards_avg_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
            "name": "source_chain_id",
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
        "name": "v0_transfer_forwards_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_transfer_forwards_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transfer_forwards_bool_exp"
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
                  "name": "v0_transfer_forwards_bool_exp"
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
            "name": "channel",
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
            "name": "port",
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
            "name": "timeout",
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
        "name": "v0_transfer_forwards_max_order_by",
        "inputFields": [
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
            "name": "port",
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
            "name": "timeout",
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
        "kind": "INPUT_OBJECT",
        "name": "v0_transfer_forwards_min_order_by",
        "inputFields": [
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
            "name": "port",
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
            "name": "timeout",
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
        "kind": "INPUT_OBJECT",
        "name": "v0_transfer_forwards_order_by",
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
            "name": "channel",
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
            "name": "port",
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
            "name": "timeout",
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
        "name": "v0_transfer_forwards_select_column",
        "enumValues": [
          {
            "name": "chain_id",
            "isDeprecated": false
          },
          {
            "name": "channel",
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
            "name": "port",
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
            "name": "timeout",
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
        "name": "v0_transfer_forwards_stddev_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
            "name": "source_chain_id",
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
        "name": "v0_transfer_forwards_stddev_pop_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
            "name": "source_chain_id",
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
        "name": "v0_transfer_forwards_stddev_samp_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
            "name": "source_chain_id",
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
        "name": "v0_transfer_forwards_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v0_transfer_forwards_stream_cursor_value_input"
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
        "name": "v0_transfer_forwards_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "channel",
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
            "name": "port",
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
            "name": "timeout",
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
        "kind": "INPUT_OBJECT",
        "name": "v0_transfer_forwards_sum_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
            "name": "source_chain_id",
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
        "name": "v0_transfer_forwards_var_pop_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
            "name": "source_chain_id",
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
        "name": "v0_transfer_forwards_var_samp_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
            "name": "source_chain_id",
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
        "name": "v0_transfer_forwards_variance_order_by",
        "inputFields": [
          {
            "name": "chain_id",
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
            "name": "source_chain_id",
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
            "name": "destination_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v0_chains"
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
            "name": "destination_id",
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
            "name": "forwards",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_transfer_forwards"
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
                      "name": "v0_transfer_forwards_select_column"
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
                      "name": "v0_transfer_forwards_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_transfer_forwards_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "forwards_2",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "OBJECT",
                  "name": "v0_transfer_forwards"
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
                      "name": "v0_transfer_forwards_select_column"
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
                      "name": "v0_transfer_forwards_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_transfer_forwards_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "hop",
            "type": {
              "kind": "OBJECT",
              "name": "v0_transfers"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "is_initiating",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
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
            "name": "packet_data_jsonb",
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
            "name": "pfm_destination_channel",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "pfm_destination_port",
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
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "pfm_sent_sequence",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "pfm_source_channel",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "pfm_source_port",
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
              "name": "v0_chains"
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
            "name": "source_id",
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
            "name": "traces",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_traces"
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
                      "name": "v0_traces_select_column"
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
                      "name": "v0_traces_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_traces_bool_exp"
                }
              }
            ],
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
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_chains_bool_exp"
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
            "name": "destination_id",
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
            "name": "forwards",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transfer_forwards_bool_exp"
            }
          },
          {
            "name": "forwards_2",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transfer_forwards_bool_exp"
            }
          },
          {
            "name": "hop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transfers_bool_exp"
            }
          },
          {
            "name": "is_initiating",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Boolean_comparison_exp"
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
            "name": "packet_data_jsonb",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "pfm_destination_channel",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "pfm_destination_port",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "pfm_recv_sequence",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "pfm_sent_sequence",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "pfm_source_channel",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "pfm_source_port",
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
              "name": "v0_chains_bool_exp"
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
            "name": "source_id",
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
            "name": "traces",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_traces_bool_exp"
            }
          },
          {
            "name": "transaction_hash",
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
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_chains_order_by"
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
            "name": "destination_id",
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
            "name": "forwards_2_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transfer_forwards_aggregate_order_by"
            }
          },
          {
            "name": "forwards_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transfer_forwards_aggregate_order_by"
            }
          },
          {
            "name": "hop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_transfers_order_by"
            }
          },
          {
            "name": "is_initiating",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
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
            "name": "packet_data_jsonb",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "pfm_destination_channel",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "pfm_destination_port",
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
            "name": "pfm_source_channel",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "pfm_source_port",
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
              "name": "v0_chains_order_by"
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
            "name": "source_id",
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
            "name": "traces_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_traces_aggregate_order_by"
            }
          },
          {
            "name": "transaction_hash",
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
            "name": "destination_channel_id",
            "isDeprecated": false
          },
          {
            "name": "destination_connection_id",
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
            "name": "destination_id",
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
            "name": "destination_timeout_timestamp",
            "isDeprecated": false
          },
          {
            "name": "destination_timestamp",
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
            "name": "is_initiating",
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
            "name": "packet_data_jsonb",
            "isDeprecated": false
          },
          {
            "name": "pfm_destination_channel",
            "isDeprecated": false
          },
          {
            "name": "pfm_destination_port",
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
            "name": "pfm_source_channel",
            "isDeprecated": false
          },
          {
            "name": "pfm_source_port",
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
            "name": "source_data",
            "isDeprecated": false
          },
          {
            "name": "source_height",
            "isDeprecated": false
          },
          {
            "name": "source_id",
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
            "name": "source_timeout_timestamp",
            "isDeprecated": false
          },
          {
            "name": "source_timestamp",
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
            "name": "transaction_hash",
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
            "name": "destination_id",
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
            "name": "is_initiating",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
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
            "name": "packet_data_jsonb",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "pfm_destination_channel",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "pfm_destination_port",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "pfm_recv_sequence",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "pfm_sent_sequence",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "pfm_source_channel",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "pfm_source_port",
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
            "name": "source_id",
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
            "name": "transaction_hash",
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
        "name": "v0_ucs1_configuration",
        "fields": [
          {
            "name": "channel_id",
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
            "name": "connection_id",
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
            "name": "contract_address",
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
            "name": "destination_chain",
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
            "name": "destination_chain_id",
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
            "name": "forward",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v0_ucs1_configuration"
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
                      "name": "v0_ucs1_configuration_select_column"
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
                      "name": "v0_ucs1_configuration_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_ucs1_configuration_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "port",
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
            "name": "source_chain",
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
            "name": "source_chain_id",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Int"
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
        "name": "v0_ucs1_configuration_aggregate_order_by",
        "inputFields": [
          {
            "name": "avg",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_ucs1_configuration_avg_order_by"
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
              "name": "v0_ucs1_configuration_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_ucs1_configuration_min_order_by"
            }
          },
          {
            "name": "stddev",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_ucs1_configuration_stddev_order_by"
            }
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_ucs1_configuration_stddev_pop_order_by"
            }
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_ucs1_configuration_stddev_samp_order_by"
            }
          },
          {
            "name": "sum",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_ucs1_configuration_sum_order_by"
            }
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_ucs1_configuration_var_pop_order_by"
            }
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_ucs1_configuration_var_samp_order_by"
            }
          },
          {
            "name": "variance",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_ucs1_configuration_variance_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_ucs1_configuration_avg_order_by",
        "inputFields": [
          {
            "name": "destination_chain_id",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_ucs1_configuration_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v0_ucs1_configuration_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_ucs1_configuration_bool_exp"
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
                  "name": "v0_ucs1_configuration_bool_exp"
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
              "name": "v0_chains_bool_exp"
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
            "name": "forward",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_ucs1_configuration_bool_exp"
            }
          },
          {
            "name": "port",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_chains_bool_exp"
            }
          },
          {
            "name": "source_chain_id",
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
        "name": "v0_ucs1_configuration_max_order_by",
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
            "name": "destination_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "port",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_ucs1_configuration_min_order_by",
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
            "name": "destination_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "port",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_ucs1_configuration_order_by",
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
              "name": "v0_chains_order_by"
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
            "name": "forward_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_ucs1_configuration_aggregate_order_by"
            }
          },
          {
            "name": "port",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v0_chains_order_by"
            }
          },
          {
            "name": "source_chain_id",
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
        "name": "v0_ucs1_configuration_select_column",
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
            "name": "destination_chain_id",
            "isDeprecated": false
          },
          {
            "name": "port",
            "isDeprecated": false
          },
          {
            "name": "source_chain_id",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_ucs1_configuration_stddev_order_by",
        "inputFields": [
          {
            "name": "destination_chain_id",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_ucs1_configuration_stddev_pop_order_by",
        "inputFields": [
          {
            "name": "destination_chain_id",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_ucs1_configuration_stddev_samp_order_by",
        "inputFields": [
          {
            "name": "destination_chain_id",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_ucs1_configuration_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v0_ucs1_configuration_stream_cursor_value_input"
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
        "name": "v0_ucs1_configuration_stream_cursor_value_input",
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
            "name": "destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "port",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_ucs1_configuration_sum_order_by",
        "inputFields": [
          {
            "name": "destination_chain_id",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_ucs1_configuration_var_pop_order_by",
        "inputFields": [
          {
            "name": "destination_chain_id",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_ucs1_configuration_var_samp_order_by",
        "inputFields": [
          {
            "name": "destination_chain_id",
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v0_ucs1_configuration_variance_order_by",
        "inputFields": [
          {
            "name": "destination_chain_id",
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