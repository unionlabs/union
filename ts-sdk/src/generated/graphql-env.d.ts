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
        "name": "String_array_comparison_exp",
        "inputFields": [
          {
            "name": "_contained_in",
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
            "name": "_contains",
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
            "name": "_eq",
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
            "name": "_gt",
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
            "name": "_gte",
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
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "SCALAR",
                      "name": "String"
                    }
                  }
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
            "name": "_lte",
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
            "name": "_neq",
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
            "name": "_nin",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "LIST",
                  "ofType": {
                    "kind": "NON_NULL",
                    "ofType": {
                      "kind": "SCALAR",
                      "name": "String"
                    }
                  }
                }
              }
            }
          }
        ],
        "isOneOf": false
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
        "name": "dashboard_count_by_chain_type",
        "fields": [
          {
            "name": "count",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "phase",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "universal_chain_id",
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
        "name": "dashboard_count_by_chain_type_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "dashboard_count_by_chain_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "dashboard_count_by_chain_type_bool_exp"
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
                  "name": "dashboard_count_by_chain_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "count",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "phase",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "universal_chain_id",
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
        "name": "dashboard_count_by_chain_type_order_by",
        "inputFields": [
          {
            "name": "count",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "phase",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "universal_chain_id",
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
        "name": "dashboard_count_by_chain_type_select_column",
        "enumValues": [
          {
            "name": "count",
            "isDeprecated": false
          },
          {
            "name": "phase",
            "isDeprecated": false
          },
          {
            "name": "universal_chain_id",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "OBJECT",
        "name": "dashboard_days_by_chain_type",
        "fields": [
          {
            "name": "day_count",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "universal_chain_id",
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
        "name": "dashboard_days_by_chain_type_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "dashboard_days_by_chain_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "dashboard_days_by_chain_type_bool_exp"
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
                  "name": "dashboard_days_by_chain_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "day_count",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "universal_chain_id",
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
        "name": "dashboard_days_by_chain_type_order_by",
        "inputFields": [
          {
            "name": "day_count",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "universal_chain_id",
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
        "name": "dashboard_days_by_chain_type_select_column",
        "enumValues": [
          {
            "name": "day_count",
            "isDeprecated": false
          },
          {
            "name": "universal_chain_id",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "dashboard_transfer_count_by_chain_args",
        "inputFields": [
          {
            "name": "p_addresses_dashboard",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "p_phase",
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
        "name": "dashboard_transfer_days_count_by_chain_args",
        "inputFields": [
          {
            "name": "p_addresses_dashboard",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          }
        ],
        "isOneOf": false
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
        "name": "drip_dropMutation",
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
                "name": "chainId",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "SCALAR",
                    "name": "String"
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
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "drip_dropQuery",
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
        "name": "latency_percentiles_scalar"
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "latency_percentiles_scalar_comparison_exp",
        "inputFields": [
          {
            "name": "_eq",
            "type": {
              "kind": "SCALAR",
              "name": "latency_percentiles_scalar"
            }
          },
          {
            "name": "_gt",
            "type": {
              "kind": "SCALAR",
              "name": "latency_percentiles_scalar"
            }
          },
          {
            "name": "_gte",
            "type": {
              "kind": "SCALAR",
              "name": "latency_percentiles_scalar"
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
                  "name": "latency_percentiles_scalar"
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
              "name": "latency_percentiles_scalar"
            }
          },
          {
            "name": "_lte",
            "type": {
              "kind": "SCALAR",
              "name": "latency_percentiles_scalar"
            }
          },
          {
            "name": "_neq",
            "type": {
              "kind": "SCALAR",
              "name": "latency_percentiles_scalar"
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
                  "name": "latency_percentiles_scalar"
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
            "name": "drip_drop",
            "type": {
              "kind": "OBJECT",
              "name": "drip_dropMutation"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
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
            "name": "dashboard_transfer_count_by_chain",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "dashboard_count_by_chain_type"
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
                    "name": "dashboard_transfer_count_by_chain_args"
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
                      "name": "dashboard_count_by_chain_type_select_column"
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
                      "name": "dashboard_count_by_chain_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "dashboard_count_by_chain_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "dashboard_transfer_days_count_by_chain",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "dashboard_days_by_chain_type"
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
                    "name": "dashboard_transfer_days_count_by_chain_args"
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
                      "name": "dashboard_days_by_chain_type_select_column"
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
                      "name": "dashboard_days_by_chain_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "dashboard_days_by_chain_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "drip_drop",
            "type": {
              "kind": "OBJECT",
              "name": "drip_dropQuery"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "v2_chains",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_chain_type"
                  }
                }
              }
            },
            "args": [
              {
                "name": "args",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_chains_args"
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
                      "name": "v2_chain_type_select_column"
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
                      "name": "v2_chain_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_chain_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_channels",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_channel_type"
                  }
                }
              }
            },
            "args": [
              {
                "name": "args",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_channels_args"
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
                      "name": "v2_channel_type_select_column"
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
                      "name": "v2_channel_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_channel_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_clients",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_client_type"
                  }
                }
              }
            },
            "args": [
              {
                "name": "args",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_clients_args"
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
                      "name": "v2_client_type_select_column"
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
                      "name": "v2_client_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_client_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_connections",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_connection_type"
                  }
                }
              }
            },
            "args": [
              {
                "name": "args",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_connections_args"
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
                      "name": "v2_connection_type_select_column"
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
                      "name": "v2_connection_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_connection_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_error_type",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_error_type"
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
                      "name": "v2_error_type_select_column"
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
                      "name": "v2_error_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_error_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_errors",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_error_type"
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
                      "name": "v2_error_type_select_column"
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
                      "name": "v2_error_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_error_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_health_check",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_health_check_type"
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
                      "name": "v2_health_check_type_select_column"
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
                      "name": "v2_health_check_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_health_check_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_packets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_packet_type"
                  }
                }
              }
            },
            "args": [
              {
                "name": "args",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_packets_args"
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
                      "name": "v2_packet_type_select_column"
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
                      "name": "v2_packet_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_packet_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_stats_count",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_stats_type"
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
                      "name": "v2_stats_type_select_column"
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
                      "name": "v2_stats_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_stats_latency",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_stats_latency_type"
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
                    "name": "v2_stats_latency_args"
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
                      "name": "v2_stats_latency_type_select_column"
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
                      "name": "v2_stats_latency_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_latency_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_stats_packets_chain",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_stats_packets_chain_type"
                  }
                }
              }
            },
            "args": [
              {
                "name": "args",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_packets_chain_args"
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
                      "name": "v2_stats_packets_chain_type_select_column"
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
                      "name": "v2_stats_packets_chain_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_packets_chain_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_stats_packets_daily_count",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_stats_daily_count_type"
                  }
                }
              }
            },
            "args": [
              {
                "name": "args",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_packets_daily_count_args"
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
                      "name": "v2_stats_daily_count_type_select_column"
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
                      "name": "v2_stats_daily_count_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_daily_count_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_stats_transfers_address",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_stats_transfers_address_type"
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
                    "name": "v2_stats_transfers_address_args"
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
                      "name": "v2_stats_transfers_address_type_select_column"
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
                      "name": "v2_stats_transfers_address_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_transfers_address_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_stats_transfers_chain",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_stats_transfers_chain_type"
                  }
                }
              }
            },
            "args": [
              {
                "name": "args",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_transfers_chain_args"
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
                      "name": "v2_stats_transfers_chain_type_select_column"
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
                      "name": "v2_stats_transfers_chain_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_transfers_chain_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_stats_transfers_daily_count",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_stats_daily_count_type"
                  }
                }
              }
            },
            "args": [
              {
                "name": "args",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_transfers_daily_count_args"
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
                      "name": "v2_stats_daily_count_type_select_column"
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
                      "name": "v2_stats_daily_count_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_daily_count_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_tokens",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_token_meta"
                  }
                }
              }
            },
            "args": [
              {
                "name": "args",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_tokens_args"
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
                      "name": "v2_token_meta_select_column"
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
                      "name": "v2_token_meta_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_token_meta_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_transfers",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_transfer_type"
                  }
                }
              }
            },
            "args": [
              {
                "name": "args",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_transfers_args"
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
                      "name": "v2_transfer_type_select_column"
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
                      "name": "v2_transfer_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_transfer_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_util_get_address_types_for_display_address",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_util_get_address_types_for_display_address_type"
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
                    "name": "v2_util_get_address_types_for_display_address_args"
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
                      "name": "v2_util_get_address_types_for_display_address_type_select_column"
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
                      "name": "v2_util_get_address_types_for_display_address_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_util_get_address_types_for_display_address_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_util_get_transfer_request_details",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_util_get_transfer_request_details_type"
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
                    "name": "v2_util_get_transfer_request_details_args"
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
                      "name": "v2_util_get_transfer_request_details_type_select_column"
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
                      "name": "v2_util_get_transfer_request_details_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_util_get_transfer_request_details_type_bool_exp"
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
            "name": "dashboard_transfer_count_by_chain",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "dashboard_count_by_chain_type"
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
                    "name": "dashboard_transfer_count_by_chain_args"
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
                      "name": "dashboard_count_by_chain_type_select_column"
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
                      "name": "dashboard_count_by_chain_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "dashboard_count_by_chain_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "dashboard_transfer_days_count_by_chain",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "dashboard_days_by_chain_type"
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
                    "name": "dashboard_transfer_days_count_by_chain_args"
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
                      "name": "dashboard_days_by_chain_type_select_column"
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
                      "name": "dashboard_days_by_chain_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "dashboard_days_by_chain_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_chains",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_chain_type"
                  }
                }
              }
            },
            "args": [
              {
                "name": "args",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_chains_args"
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
                      "name": "v2_chain_type_select_column"
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
                      "name": "v2_chain_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_chain_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_channels",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_channel_type"
                  }
                }
              }
            },
            "args": [
              {
                "name": "args",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_channels_args"
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
                      "name": "v2_channel_type_select_column"
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
                      "name": "v2_channel_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_channel_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_clients",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_client_type"
                  }
                }
              }
            },
            "args": [
              {
                "name": "args",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_clients_args"
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
                      "name": "v2_client_type_select_column"
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
                      "name": "v2_client_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_client_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_connections",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_connection_type"
                  }
                }
              }
            },
            "args": [
              {
                "name": "args",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_connections_args"
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
                      "name": "v2_connection_type_select_column"
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
                      "name": "v2_connection_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_connection_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_error_type",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_error_type"
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
                      "name": "v2_error_type_select_column"
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
                      "name": "v2_error_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_error_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_error_type_stream",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_error_type"
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
                      "name": "v2_error_type_stream_cursor_input"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_error_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_errors",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_error_type"
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
                      "name": "v2_error_type_select_column"
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
                      "name": "v2_error_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_error_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_health_check",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_health_check_type"
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
                      "name": "v2_health_check_type_select_column"
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
                      "name": "v2_health_check_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_health_check_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_packets",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_packet_type"
                  }
                }
              }
            },
            "args": [
              {
                "name": "args",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_packets_args"
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
                      "name": "v2_packet_type_select_column"
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
                      "name": "v2_packet_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_packet_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_stats_count",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_stats_type"
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
                      "name": "v2_stats_type_select_column"
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
                      "name": "v2_stats_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_stats_latency",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_stats_latency_type"
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
                    "name": "v2_stats_latency_args"
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
                      "name": "v2_stats_latency_type_select_column"
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
                      "name": "v2_stats_latency_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_latency_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_stats_packets_chain",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_stats_packets_chain_type"
                  }
                }
              }
            },
            "args": [
              {
                "name": "args",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_packets_chain_args"
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
                      "name": "v2_stats_packets_chain_type_select_column"
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
                      "name": "v2_stats_packets_chain_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_packets_chain_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_stats_packets_daily_count",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_stats_daily_count_type"
                  }
                }
              }
            },
            "args": [
              {
                "name": "args",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_packets_daily_count_args"
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
                      "name": "v2_stats_daily_count_type_select_column"
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
                      "name": "v2_stats_daily_count_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_daily_count_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_stats_transfers_address",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_stats_transfers_address_type"
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
                    "name": "v2_stats_transfers_address_args"
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
                      "name": "v2_stats_transfers_address_type_select_column"
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
                      "name": "v2_stats_transfers_address_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_transfers_address_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_stats_transfers_chain",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_stats_transfers_chain_type"
                  }
                }
              }
            },
            "args": [
              {
                "name": "args",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_transfers_chain_args"
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
                      "name": "v2_stats_transfers_chain_type_select_column"
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
                      "name": "v2_stats_transfers_chain_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_transfers_chain_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_stats_transfers_daily_count",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_stats_daily_count_type"
                  }
                }
              }
            },
            "args": [
              {
                "name": "args",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_transfers_daily_count_args"
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
                      "name": "v2_stats_daily_count_type_select_column"
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
                      "name": "v2_stats_daily_count_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_daily_count_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_tokens",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_token_meta"
                  }
                }
              }
            },
            "args": [
              {
                "name": "args",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_tokens_args"
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
                      "name": "v2_token_meta_select_column"
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
                      "name": "v2_token_meta_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_token_meta_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_transfers",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_transfer_type"
                  }
                }
              }
            },
            "args": [
              {
                "name": "args",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_transfers_args"
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
                      "name": "v2_transfer_type_select_column"
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
                      "name": "v2_transfer_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_transfer_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_util_get_address_types_for_display_address",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_util_get_address_types_for_display_address_type"
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
                    "name": "v2_util_get_address_types_for_display_address_args"
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
                      "name": "v2_util_get_address_types_for_display_address_type_select_column"
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
                      "name": "v2_util_get_address_types_for_display_address_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_util_get_address_types_for_display_address_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "v2_util_get_transfer_request_details",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_util_get_transfer_request_details_type"
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
                    "name": "v2_util_get_transfer_request_details_args"
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
                      "name": "v2_util_get_transfer_request_details_type_select_column"
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
                      "name": "v2_util_get_transfer_request_details_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_util_get_transfer_request_details_type_bool_exp"
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
        "name": "v2_chain_features",
        "fields": [
          {
            "name": "channel_list",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "connection_list",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "environment",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "index_status",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_list",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transfer_list",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transfer_submission",
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
        "name": "v2_chain_features_aggregate_order_by",
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
              "name": "v2_chain_features_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chain_features_min_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_chain_features_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_chain_features_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chain_features_bool_exp"
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
                  "name": "v2_chain_features_bool_exp"
                }
              }
            }
          },
          {
            "name": "channel_list",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Boolean_comparison_exp"
            }
          },
          {
            "name": "connection_list",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Boolean_comparison_exp"
            }
          },
          {
            "name": "environment",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "index_status",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Boolean_comparison_exp"
            }
          },
          {
            "name": "packet_list",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Boolean_comparison_exp"
            }
          },
          {
            "name": "transfer_list",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Boolean_comparison_exp"
            }
          },
          {
            "name": "transfer_submission",
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
        "name": "v2_chain_features_max_order_by",
        "inputFields": [
          {
            "name": "environment",
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
        "name": "v2_chain_features_min_order_by",
        "inputFields": [
          {
            "name": "environment",
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
        "name": "v2_chain_features_order_by",
        "inputFields": [
          {
            "name": "channel_list",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "connection_list",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "environment",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "index_status",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_list",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "transfer_list",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "transfer_submission",
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
        "name": "v2_chain_features_select_column",
        "enumValues": [
          {
            "name": "channel_list",
            "isDeprecated": false
          },
          {
            "name": "connection_list",
            "isDeprecated": false
          },
          {
            "name": "environment",
            "isDeprecated": false
          },
          {
            "name": "index_status",
            "isDeprecated": false
          },
          {
            "name": "packet_list",
            "isDeprecated": false
          },
          {
            "name": "transfer_list",
            "isDeprecated": false
          },
          {
            "name": "transfer_submission",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "OBJECT",
        "name": "v2_chain_status_type",
        "fields": [
          {
            "name": "height",
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
        "name": "v2_chain_status_type_aggregate_order_by",
        "inputFields": [
          {
            "name": "avg",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chain_status_type_avg_order_by"
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
              "name": "v2_chain_status_type_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chain_status_type_min_order_by"
            }
          },
          {
            "name": "stddev",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chain_status_type_stddev_order_by"
            }
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chain_status_type_stddev_pop_order_by"
            }
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chain_status_type_stddev_samp_order_by"
            }
          },
          {
            "name": "sum",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chain_status_type_sum_order_by"
            }
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chain_status_type_var_pop_order_by"
            }
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chain_status_type_var_samp_order_by"
            }
          },
          {
            "name": "variance",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chain_status_type_variance_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_chain_status_type_avg_order_by",
        "inputFields": [
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
        "kind": "INPUT_OBJECT",
        "name": "v2_chain_status_type_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_chain_status_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chain_status_type_bool_exp"
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
                  "name": "v2_chain_status_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "height",
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
              "name": "Int_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_chain_status_type_max_order_by",
        "inputFields": [
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
        "kind": "INPUT_OBJECT",
        "name": "v2_chain_status_type_min_order_by",
        "inputFields": [
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
        "kind": "INPUT_OBJECT",
        "name": "v2_chain_status_type_order_by",
        "inputFields": [
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
        "name": "v2_chain_status_type_select_column",
        "enumValues": [
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
        "name": "v2_chain_status_type_stddev_order_by",
        "inputFields": [
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
        "kind": "INPUT_OBJECT",
        "name": "v2_chain_status_type_stddev_pop_order_by",
        "inputFields": [
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
        "kind": "INPUT_OBJECT",
        "name": "v2_chain_status_type_stddev_samp_order_by",
        "inputFields": [
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
        "kind": "INPUT_OBJECT",
        "name": "v2_chain_status_type_sum_order_by",
        "inputFields": [
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
        "kind": "INPUT_OBJECT",
        "name": "v2_chain_status_type_var_pop_order_by",
        "inputFields": [
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
        "kind": "INPUT_OBJECT",
        "name": "v2_chain_status_type_var_samp_order_by",
        "inputFields": [
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
        "kind": "INPUT_OBJECT",
        "name": "v2_chain_status_type_variance_order_by",
        "inputFields": [
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
        "kind": "OBJECT",
        "name": "v2_chain_type",
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
            "name": "explorers",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_explorers"
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
                      "name": "v2_explorers_select_column"
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
                      "name": "v2_explorers_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_explorers_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "features",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_chain_features"
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
                      "name": "v2_chain_features_select_column"
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
                      "name": "v2_chain_features_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_chain_features_bool_exp"
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
                    "name": "v2_rpcs"
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
                      "name": "v2_rpcs_select_column"
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
                      "name": "v2_rpcs_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_rpcs_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "status",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "OBJECT",
                  "name": "v2_chain_status_type"
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
                      "name": "v2_chain_status_type_select_column"
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
                      "name": "v2_chain_status_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_chain_status_type_bool_exp"
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
            "name": "universal_chain_id",
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
        "name": "v2_chain_type_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_chain_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chain_type_bool_exp"
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
                  "name": "v2_chain_type_bool_exp"
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
            "name": "explorers",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_explorers_bool_exp"
            }
          },
          {
            "name": "features",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chain_features_bool_exp"
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
              "name": "v2_rpcs_bool_exp"
            }
          },
          {
            "name": "status",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chain_status_type_bool_exp"
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
            "name": "universal_chain_id",
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
        "name": "v2_chain_type_order_by",
        "inputFields": [
          {
            "name": "addr_prefix",
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
            "name": "display_name",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "explorers_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_explorers_aggregate_order_by"
            }
          },
          {
            "name": "features_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chain_features_aggregate_order_by"
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
              "name": "v2_rpcs_aggregate_order_by"
            }
          },
          {
            "name": "status_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chain_status_type_aggregate_order_by"
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
            "name": "universal_chain_id",
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
        "name": "v2_chain_type_select_column",
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
          },
          {
            "name": "universal_chain_id",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_chains_args",
        "inputFields": [
          {
            "name": "p_comparison",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_limit",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "p_sort_order",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_universal_chain_id",
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
        "name": "v2_chains_view",
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
            "name": "testnet",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "universal_chain_id",
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
        "name": "v2_chains_view_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_chains_view_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_bool_exp"
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
                  "name": "v2_chains_view_bool_exp"
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
            "name": "testnet",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Boolean_comparison_exp"
            }
          },
          {
            "name": "universal_chain_id",
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
        "name": "v2_chains_view_order_by",
        "inputFields": [
          {
            "name": "addr_prefix",
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
            "name": "display_name",
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
            "name": "testnet",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "universal_chain_id",
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
        "name": "v2_channel_type",
        "fields": [
          {
            "name": "destination_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v2_chains_view"
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
            "name": "destination_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sort_order",
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
              "name": "v2_chains_view"
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
            "name": "source_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "tags",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "SCALAR",
                  "name": "String"
                }
              }
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
        "name": "v2_channel_type_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_channel_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_channel_type_bool_exp"
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
                  "name": "v2_channel_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_bool_exp"
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
            "name": "destination_port_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_universal_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "sort_order",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_bool_exp"
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
            "name": "source_port_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_universal_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "tags",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_array_comparison_exp"
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
        "name": "v2_channel_type_order_by",
        "inputFields": [
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_order_by"
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
            "name": "destination_port_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_universal_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "sort_order",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_order_by"
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
            "name": "source_port_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_universal_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "tags",
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
        "name": "v2_channel_type_select_column",
        "enumValues": [
          {
            "name": "destination_channel_id",
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
            "name": "destination_port_id",
            "isDeprecated": false
          },
          {
            "name": "destination_universal_chain_id",
            "isDeprecated": false
          },
          {
            "name": "sort_order",
            "isDeprecated": false
          },
          {
            "name": "source_channel_id",
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
            "name": "source_port_id",
            "isDeprecated": false
          },
          {
            "name": "source_universal_chain_id",
            "isDeprecated": false
          },
          {
            "name": "tags",
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
        "name": "v2_channels_args",
        "inputFields": [
          {
            "name": "p_comparison",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_destination_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "p_destination_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "p_destination_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "p_destination_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_destination_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_limit",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "p_recommended",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            }
          },
          {
            "name": "p_sort_order",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_source_channel_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "p_source_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "p_source_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "p_source_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_source_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_tags",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "p_version",
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
        "name": "v2_client_type",
        "fields": [
          {
            "name": "client_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "counterparty_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sort_order",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "universal_chain_id",
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
        "name": "v2_client_type_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_client_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_client_type_bool_exp"
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
                  "name": "v2_client_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "client_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "counterparty_universal_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "sort_order",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "universal_chain_id",
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
        "name": "v2_client_type_order_by",
        "inputFields": [
          {
            "name": "client_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "counterparty_universal_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "sort_order",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "universal_chain_id",
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
        "name": "v2_client_type_select_column",
        "enumValues": [
          {
            "name": "client_id",
            "isDeprecated": false
          },
          {
            "name": "counterparty_universal_chain_id",
            "isDeprecated": false
          },
          {
            "name": "sort_order",
            "isDeprecated": false
          },
          {
            "name": "universal_chain_id",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_clients_args",
        "inputFields": [
          {
            "name": "p_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "p_comparison",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_counterparty_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_limit",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "p_sort_order",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_universal_chain_id",
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
        "name": "v2_connection_type",
        "fields": [
          {
            "name": "destination_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v2_chains_view"
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
            "name": "destination_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sort_order",
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
              "name": "v2_chains_view"
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
            "name": "source_universal_chain_id",
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
        "name": "v2_connection_type_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_connection_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_connection_type_bool_exp"
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
                  "name": "v2_connection_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_bool_exp"
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
            "name": "destination_universal_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "sort_order",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_bool_exp"
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
            "name": "source_universal_chain_id",
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
        "name": "v2_connection_type_order_by",
        "inputFields": [
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_order_by"
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
            "name": "destination_universal_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "sort_order",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_order_by"
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
            "name": "source_universal_chain_id",
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
        "name": "v2_connection_type_select_column",
        "enumValues": [
          {
            "name": "destination_client_id",
            "isDeprecated": false
          },
          {
            "name": "destination_connection_id",
            "isDeprecated": false
          },
          {
            "name": "destination_universal_chain_id",
            "isDeprecated": false
          },
          {
            "name": "sort_order",
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
            "name": "source_universal_chain_id",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_connections_args",
        "inputFields": [
          {
            "name": "p_comparison",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_destination_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "p_destination_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "p_destination_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_limit",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "p_sort_order",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_source_client_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "p_source_connection_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "p_source_universal_chain_id",
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
        "name": "v2_error_type",
        "fields": [
          {
            "name": "detail",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "hint",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
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
            "name": "union_error_code",
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
        "name": "v2_error_type_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_error_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_error_type_bool_exp"
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
                  "name": "v2_error_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "detail",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "hint",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
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
            "name": "union_error_code",
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
        "name": "v2_error_type_order_by",
        "inputFields": [
          {
            "name": "detail",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "hint",
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
            "name": "union_error_code",
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
        "name": "v2_error_type_select_column",
        "enumValues": [
          {
            "name": "detail",
            "isDeprecated": false
          },
          {
            "name": "hint",
            "isDeprecated": false
          },
          {
            "name": "message",
            "isDeprecated": false
          },
          {
            "name": "union_error_code",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_error_type_stream_cursor_input",
        "inputFields": [
          {
            "name": "initial_value",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "INPUT_OBJECT",
                "name": "v2_error_type_stream_cursor_value_input"
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
        "name": "v2_error_type_stream_cursor_value_input",
        "inputFields": [
          {
            "name": "detail",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "hint",
            "type": {
              "kind": "SCALAR",
              "name": "String"
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
            "name": "union_error_code",
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
        "name": "v2_explorers",
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
        "name": "v2_explorers_aggregate_order_by",
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
              "name": "v2_explorers_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_explorers_min_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_explorers_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_explorers_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_explorers_bool_exp"
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
                  "name": "v2_explorers_bool_exp"
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
        "name": "v2_explorers_max_order_by",
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
        "name": "v2_explorers_min_order_by",
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
        "name": "v2_explorers_order_by",
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
        "kind": "ENUM",
        "name": "v2_explorers_select_column",
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
        "kind": "OBJECT",
        "name": "v2_health_check_type",
        "fields": [
          {
            "name": "environment",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "last_update",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
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
        "name": "v2_health_check_type_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_health_check_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_health_check_type_bool_exp"
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
                  "name": "v2_health_check_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "environment",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "last_update",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
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
        "name": "v2_health_check_type_order_by",
        "inputFields": [
          {
            "name": "environment",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "last_update",
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
        "name": "v2_health_check_type_select_column",
        "enumValues": [
          {
            "name": "environment",
            "isDeprecated": false
          },
          {
            "name": "last_update",
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
        "name": "v2_packet_type",
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
            "name": "decoded",
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
            "name": "decoded_flattened",
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
              "name": "v2_chains_view"
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
            "name": "destination_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
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
            "name": "packet_ack_height",
            "type": {
              "kind": "SCALAR",
              "name": "String"
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
            "name": "packet_ack_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
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
            "name": "packet_recv_height",
            "type": {
              "kind": "SCALAR",
              "name": "String"
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
            "name": "packet_recv_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
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
            "name": "packet_send_height",
            "type": {
              "kind": "SCALAR",
              "name": "String"
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
            "name": "packet_send_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_timeout_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_timeout_height",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_timeout_maker",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_timeout_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_timeout_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sort_order",
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
              "name": "v2_chains_view"
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
            "name": "source_port_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_universal_chain_id",
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
            "name": "success",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "timeout_height",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "timeout_timestamp",
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
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "OBJECT",
                  "name": "v2_traces_type"
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
                      "name": "v2_traces_type_select_column"
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
                      "name": "v2_traces_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_traces_type_bool_exp"
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
            "name": "write_ack_height",
            "type": {
              "kind": "SCALAR",
              "name": "String"
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
            "name": "write_ack_transaction_hash",
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
        "name": "v2_packet_type_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_packet_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_packet_type_bool_exp"
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
                  "name": "v2_packet_type_bool_exp"
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
            "name": "decoded",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "decoded_flattened",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "jsonb_comparison_exp"
            }
          },
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_bool_exp"
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
            "name": "destination_port_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "destination_universal_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
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
            "name": "packet_ack_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
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
            "name": "packet_ack_transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
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
            "name": "packet_recv_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
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
            "name": "packet_recv_transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
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
            "name": "packet_send_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
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
            "name": "packet_send_transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet_timeout_block_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet_timeout_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet_timeout_maker",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet_timeout_timestamp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "packet_timeout_transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "sort_order",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_bool_exp"
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
            "name": "source_port_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_universal_chain_id",
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
            "name": "success",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Boolean_comparison_exp"
            }
          },
          {
            "name": "timeout_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "timeout_timestamp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "traces",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_traces_type_bool_exp"
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
            "name": "write_ack_height",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
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
            "name": "write_ack_transaction_hash",
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
        "name": "v2_packet_type_order_by",
        "inputFields": [
          {
            "name": "acknowledgement",
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
            "name": "decoded",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "decoded_flattened",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_order_by"
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
            "name": "destination_port_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_universal_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
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
            "name": "packet_ack_transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_hash",
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
            "name": "packet_recv_transaction_hash",
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
            "name": "packet_send_transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_timeout_block_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_timeout_height",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_timeout_maker",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_timeout_timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_timeout_transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "sort_order",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_order_by"
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
            "name": "source_port_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_universal_chain_id",
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
            "name": "success",
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
              "name": "v2_traces_type_aggregate_order_by"
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
            "name": "write_ack_transaction_hash",
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
        "name": "v2_packet_type_select_column",
        "enumValues": [
          {
            "name": "acknowledgement",
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
            "name": "decoded",
            "isDeprecated": false
          },
          {
            "name": "decoded_flattened",
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
            "name": "destination_client_id",
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
            "name": "destination_universal_chain_id",
            "isDeprecated": false
          },
          {
            "name": "packet_ack_block_hash",
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
            "name": "packet_ack_transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "packet_hash",
            "isDeprecated": false
          },
          {
            "name": "packet_recv_block_hash",
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
            "name": "packet_recv_transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "packet_send_block_hash",
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
            "name": "packet_send_transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "packet_timeout_block_hash",
            "isDeprecated": false
          },
          {
            "name": "packet_timeout_height",
            "isDeprecated": false
          },
          {
            "name": "packet_timeout_maker",
            "isDeprecated": false
          },
          {
            "name": "packet_timeout_timestamp",
            "isDeprecated": false
          },
          {
            "name": "packet_timeout_transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "sort_order",
            "isDeprecated": false
          },
          {
            "name": "source_channel_id",
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
            "name": "source_port_id",
            "isDeprecated": false
          },
          {
            "name": "source_universal_chain_id",
            "isDeprecated": false
          },
          {
            "name": "status",
            "isDeprecated": false
          },
          {
            "name": "success",
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
            "name": "write_ack_height",
            "isDeprecated": false
          },
          {
            "name": "write_ack_timestamp",
            "isDeprecated": false
          },
          {
            "name": "write_ack_transaction_hash",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_packets_args",
        "inputFields": [
          {
            "name": "p_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_comparison",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_destination_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_limit",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "p_packet_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_packet_send_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            }
          },
          {
            "name": "p_sort_order",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_source_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_transaction_hash",
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
        "name": "v2_rpcs",
        "fields": [
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
        "name": "v2_rpcs_aggregate_order_by",
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
              "name": "v2_rpcs_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_rpcs_min_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_rpcs_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_rpcs_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_rpcs_bool_exp"
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
                  "name": "v2_rpcs_bool_exp"
                }
              }
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
        "name": "v2_rpcs_max_order_by",
        "inputFields": [
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
        "name": "v2_rpcs_min_order_by",
        "inputFields": [
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
        "name": "v2_rpcs_order_by",
        "inputFields": [
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
        "name": "v2_rpcs_select_column",
        "enumValues": [
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
        "kind": "OBJECT",
        "name": "v2_stats_daily_count_type",
        "fields": [
          {
            "name": "count",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "day_date",
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
        "name": "v2_stats_daily_count_type_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_daily_count_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_stats_daily_count_type_bool_exp"
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
                  "name": "v2_stats_daily_count_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "count",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "day_date",
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
        "name": "v2_stats_daily_count_type_order_by",
        "inputFields": [
          {
            "name": "count",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "day_date",
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
        "name": "v2_stats_daily_count_type_select_column",
        "enumValues": [
          {
            "name": "count",
            "isDeprecated": false
          },
          {
            "name": "day_date",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_stats_latency_args",
        "inputFields": [
          {
            "name": "p_destination_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_phase",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_source_universal_chain_id",
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
        "name": "v2_stats_latency_type",
        "fields": [
          {
            "name": "secs_until_packet_ack",
            "type": {
              "kind": "SCALAR",
              "name": "latency_percentiles_scalar"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "secs_until_packet_recv",
            "type": {
              "kind": "SCALAR",
              "name": "latency_percentiles_scalar"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "secs_until_write_ack",
            "type": {
              "kind": "SCALAR",
              "name": "latency_percentiles_scalar"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_stats_latency_type_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_latency_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_stats_latency_type_bool_exp"
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
                  "name": "v2_stats_latency_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "secs_until_packet_ack",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "latency_percentiles_scalar_comparison_exp"
            }
          },
          {
            "name": "secs_until_packet_recv",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "latency_percentiles_scalar_comparison_exp"
            }
          },
          {
            "name": "secs_until_write_ack",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "latency_percentiles_scalar_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_stats_latency_type_order_by",
        "inputFields": [
          {
            "name": "secs_until_packet_ack",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "secs_until_packet_recv",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "secs_until_write_ack",
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
        "name": "v2_stats_latency_type_select_column",
        "enumValues": [
          {
            "name": "secs_until_packet_ack",
            "isDeprecated": false
          },
          {
            "name": "secs_until_packet_recv",
            "isDeprecated": false
          },
          {
            "name": "secs_until_write_ack",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_stats_packets_chain_args",
        "inputFields": [
          {
            "name": "p_days_back",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "p_destination_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_source_universal_chain_id",
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
        "name": "v2_stats_packets_chain_type",
        "fields": [
          {
            "name": "day_date",
            "type": {
              "kind": "SCALAR",
              "name": "date"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "total_packets",
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
        "name": "v2_stats_packets_chain_type_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_packets_chain_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_stats_packets_chain_type_bool_exp"
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
                  "name": "v2_stats_packets_chain_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "day_date",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "date_comparison_exp"
            }
          },
          {
            "name": "destination_universal_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_universal_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "total_packets",
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
        "name": "v2_stats_packets_chain_type_order_by",
        "inputFields": [
          {
            "name": "day_date",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_universal_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_universal_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "total_packets",
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
        "name": "v2_stats_packets_chain_type_select_column",
        "enumValues": [
          {
            "name": "day_date",
            "isDeprecated": false
          },
          {
            "name": "destination_universal_chain_id",
            "isDeprecated": false
          },
          {
            "name": "source_universal_chain_id",
            "isDeprecated": false
          },
          {
            "name": "total_packets",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_stats_packets_daily_count_args",
        "inputFields": [
          {
            "name": "p_days_back",
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
        "name": "v2_stats_transfers_address_args",
        "inputFields": [
          {
            "name": "p_addresses_canonical",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "p_days_back",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "p_destination_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_source_universal_chain_id",
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
        "name": "v2_stats_transfers_address_type",
        "fields": [
          {
            "name": "canonical_address",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "day_date",
            "type": {
              "kind": "SCALAR",
              "name": "date"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transfer_count",
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
        "name": "v2_stats_transfers_address_type_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_transfers_address_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_stats_transfers_address_type_bool_exp"
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
                  "name": "v2_stats_transfers_address_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "canonical_address",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "day_date",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "date_comparison_exp"
            }
          },
          {
            "name": "destination_universal_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_universal_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "transfer_count",
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
        "name": "v2_stats_transfers_address_type_order_by",
        "inputFields": [
          {
            "name": "canonical_address",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "day_date",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_universal_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_universal_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "transfer_count",
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
        "name": "v2_stats_transfers_address_type_select_column",
        "enumValues": [
          {
            "name": "canonical_address",
            "isDeprecated": false
          },
          {
            "name": "day_date",
            "isDeprecated": false
          },
          {
            "name": "destination_universal_chain_id",
            "isDeprecated": false
          },
          {
            "name": "source_universal_chain_id",
            "isDeprecated": false
          },
          {
            "name": "transfer_count",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_stats_transfers_chain_args",
        "inputFields": [
          {
            "name": "p_days_back",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "p_destination_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_source_universal_chain_id",
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
        "name": "v2_stats_transfers_chain_type",
        "fields": [
          {
            "name": "day_date",
            "type": {
              "kind": "SCALAR",
              "name": "date"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "total_transfers",
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
        "name": "v2_stats_transfers_chain_type_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_transfers_chain_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_stats_transfers_chain_type_bool_exp"
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
                  "name": "v2_stats_transfers_chain_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "day_date",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "date_comparison_exp"
            }
          },
          {
            "name": "destination_universal_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_universal_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "total_transfers",
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
        "name": "v2_stats_transfers_chain_type_order_by",
        "inputFields": [
          {
            "name": "day_date",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "destination_universal_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_universal_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "total_transfers",
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
        "name": "v2_stats_transfers_chain_type_select_column",
        "enumValues": [
          {
            "name": "day_date",
            "isDeprecated": false
          },
          {
            "name": "destination_universal_chain_id",
            "isDeprecated": false
          },
          {
            "name": "source_universal_chain_id",
            "isDeprecated": false
          },
          {
            "name": "total_transfers",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_stats_transfers_daily_count_args",
        "inputFields": [
          {
            "name": "p_days_back",
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
        "name": "v2_stats_type",
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
        "name": "v2_stats_type_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_stats_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_stats_type_bool_exp"
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
                  "name": "v2_stats_type_bool_exp"
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
              "name": "String_comparison_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_stats_type_order_by",
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
        "name": "v2_stats_type_select_column",
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
        "kind": "OBJECT",
        "name": "v2_token_meta",
        "fields": [
          {
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v2_chains_view"
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
            "name": "rank",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "representations",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_token_meta_representations"
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
                      "name": "v2_token_meta_representations_select_column"
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
                      "name": "v2_token_meta_representations_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_token_meta_representations_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "wrapping",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_token_meta_wrapping"
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
                      "name": "v2_token_meta_wrapping_select_column"
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
                      "name": "v2_token_meta_wrapping_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_token_meta_wrapping_bool_exp"
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
        "name": "v2_token_meta_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_token_meta_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_bool_exp"
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
                  "name": "v2_token_meta_bool_exp"
                }
              }
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_bool_exp"
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
            "name": "rank",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "representations",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representations_bool_exp"
            }
          },
          {
            "name": "wrapping",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_wrapping_bool_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_token_meta_order_by",
        "inputFields": [
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_order_by"
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
            "name": "rank",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "representations_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representations_aggregate_order_by"
            }
          },
          {
            "name": "wrapping_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_wrapping_aggregate_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "v2_token_meta_representation_sources",
        "fields": [
          {
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v2_chains_view"
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
            "name": "name",
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
              "name": "v2_token_meta_sources"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "symbol",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "update_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "wrapping",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_token_meta_representation_sources"
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
                      "name": "v2_token_meta_representation_sources_select_column"
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
                      "name": "v2_token_meta_representation_sources_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_token_meta_representation_sources_bool_exp"
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
        "name": "v2_token_meta_representation_sources_aggregate_order_by",
        "inputFields": [
          {
            "name": "avg",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representation_sources_avg_order_by"
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
              "name": "v2_token_meta_representation_sources_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representation_sources_min_order_by"
            }
          },
          {
            "name": "stddev",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representation_sources_stddev_order_by"
            }
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representation_sources_stddev_pop_order_by"
            }
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representation_sources_stddev_samp_order_by"
            }
          },
          {
            "name": "sum",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representation_sources_sum_order_by"
            }
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representation_sources_var_pop_order_by"
            }
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representation_sources_var_samp_order_by"
            }
          },
          {
            "name": "variance",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representation_sources_variance_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_token_meta_representation_sources_avg_order_by",
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
        "name": "v2_token_meta_representation_sources_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_token_meta_representation_sources_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representation_sources_bool_exp"
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
                  "name": "v2_token_meta_representation_sources_bool_exp"
                }
              }
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_bool_exp"
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
            "name": "name",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_sources_bool_exp"
            }
          },
          {
            "name": "symbol",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "update_timestamp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "wrapping",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representation_sources_bool_exp"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_token_meta_representation_sources_max_order_by",
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
            "name": "name",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "symbol",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "update_timestamp",
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
        "name": "v2_token_meta_representation_sources_min_order_by",
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
            "name": "name",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "symbol",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "update_timestamp",
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
        "name": "v2_token_meta_representation_sources_order_by",
        "inputFields": [
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_order_by"
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
            "name": "name",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_sources_order_by"
            }
          },
          {
            "name": "symbol",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "update_timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "wrapping_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representation_sources_aggregate_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "ENUM",
        "name": "v2_token_meta_representation_sources_select_column",
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
            "name": "name",
            "isDeprecated": false
          },
          {
            "name": "symbol",
            "isDeprecated": false
          },
          {
            "name": "update_timestamp",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_token_meta_representation_sources_stddev_order_by",
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
        "name": "v2_token_meta_representation_sources_stddev_pop_order_by",
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
        "name": "v2_token_meta_representation_sources_stddev_samp_order_by",
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
        "name": "v2_token_meta_representation_sources_sum_order_by",
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
        "name": "v2_token_meta_representation_sources_var_pop_order_by",
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
        "name": "v2_token_meta_representation_sources_var_samp_order_by",
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
        "name": "v2_token_meta_representation_sources_variance_order_by",
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
        "name": "v2_token_meta_representations",
        "fields": [
          {
            "name": "chain",
            "type": {
              "kind": "OBJECT",
              "name": "v2_chains_view"
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
            "name": "sources",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_token_meta_representation_sources"
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
                      "name": "v2_token_meta_representation_sources_select_column"
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
                      "name": "v2_token_meta_representation_sources_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_token_meta_representation_sources_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "symbol",
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
        "name": "v2_token_meta_representations_aggregate_order_by",
        "inputFields": [
          {
            "name": "avg",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representations_avg_order_by"
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
              "name": "v2_token_meta_representations_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representations_min_order_by"
            }
          },
          {
            "name": "stddev",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representations_stddev_order_by"
            }
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representations_stddev_pop_order_by"
            }
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representations_stddev_samp_order_by"
            }
          },
          {
            "name": "sum",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representations_sum_order_by"
            }
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representations_var_pop_order_by"
            }
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representations_var_samp_order_by"
            }
          },
          {
            "name": "variance",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representations_variance_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_token_meta_representations_avg_order_by",
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
        "name": "v2_token_meta_representations_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_token_meta_representations_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representations_bool_exp"
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
                  "name": "v2_token_meta_representations_bool_exp"
                }
              }
            }
          },
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_bool_exp"
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
            "name": "sources",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representation_sources_bool_exp"
            }
          },
          {
            "name": "symbol",
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
        "name": "v2_token_meta_representations_max_order_by",
        "inputFields": [
          {
            "name": "decimals",
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
            "name": "symbol",
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
        "name": "v2_token_meta_representations_min_order_by",
        "inputFields": [
          {
            "name": "decimals",
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
            "name": "symbol",
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
        "name": "v2_token_meta_representations_order_by",
        "inputFields": [
          {
            "name": "chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_order_by"
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
            "name": "sources_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_representation_sources_aggregate_order_by"
            }
          },
          {
            "name": "symbol",
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
        "name": "v2_token_meta_representations_select_column",
        "enumValues": [
          {
            "name": "decimals",
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
            "name": "symbol",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_token_meta_representations_stddev_order_by",
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
        "name": "v2_token_meta_representations_stddev_pop_order_by",
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
        "name": "v2_token_meta_representations_stddev_samp_order_by",
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
        "name": "v2_token_meta_representations_sum_order_by",
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
        "name": "v2_token_meta_representations_var_pop_order_by",
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
        "name": "v2_token_meta_representations_var_samp_order_by",
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
        "name": "v2_token_meta_representations_variance_order_by",
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
        "kind": "ENUM",
        "name": "v2_token_meta_select_column",
        "enumValues": [
          {
            "name": "denom",
            "isDeprecated": false
          },
          {
            "name": "rank",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "OBJECT",
        "name": "v2_token_meta_sources",
        "fields": [
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
            "name": "source_uri",
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
        "name": "v2_token_meta_sources_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_token_meta_sources_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_sources_bool_exp"
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
                  "name": "v2_token_meta_sources_bool_exp"
                }
              }
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
            "name": "source_uri",
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
        "name": "v2_token_meta_sources_order_by",
        "inputFields": [
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
            "name": "source_uri",
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
        "name": "v2_token_meta_wrapping",
        "fields": [
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
            "name": "unwrapped_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v2_chains_view"
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
          },
          {
            "name": "wrapped_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v2_chains_view"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "wrapper",
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
        "name": "v2_token_meta_wrapping_aggregate_order_by",
        "inputFields": [
          {
            "name": "avg",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_wrapping_avg_order_by"
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
              "name": "v2_token_meta_wrapping_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_wrapping_min_order_by"
            }
          },
          {
            "name": "stddev",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_wrapping_stddev_order_by"
            }
          },
          {
            "name": "stddev_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_wrapping_stddev_pop_order_by"
            }
          },
          {
            "name": "stddev_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_wrapping_stddev_samp_order_by"
            }
          },
          {
            "name": "sum",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_wrapping_sum_order_by"
            }
          },
          {
            "name": "var_pop",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_wrapping_var_pop_order_by"
            }
          },
          {
            "name": "var_samp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_wrapping_var_samp_order_by"
            }
          },
          {
            "name": "variance",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_wrapping_variance_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_token_meta_wrapping_avg_order_by",
        "inputFields": [
          {
            "name": "destination_channel_id",
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
        "name": "v2_token_meta_wrapping_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_token_meta_wrapping_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_wrapping_bool_exp"
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
                  "name": "v2_token_meta_wrapping_bool_exp"
                }
              }
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
            "name": "unwrapped_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_bool_exp"
            }
          },
          {
            "name": "unwrapped_denom",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "wrapped_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_bool_exp"
            }
          },
          {
            "name": "wrapper",
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
        "name": "v2_token_meta_wrapping_max_order_by",
        "inputFields": [
          {
            "name": "destination_channel_id",
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
          },
          {
            "name": "wrapper",
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
        "name": "v2_token_meta_wrapping_min_order_by",
        "inputFields": [
          {
            "name": "destination_channel_id",
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
          },
          {
            "name": "wrapper",
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
        "name": "v2_token_meta_wrapping_order_by",
        "inputFields": [
          {
            "name": "destination_channel_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "unwrapped_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_order_by"
            }
          },
          {
            "name": "unwrapped_denom",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "wrapped_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_order_by"
            }
          },
          {
            "name": "wrapper",
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
        "name": "v2_token_meta_wrapping_select_column",
        "enumValues": [
          {
            "name": "destination_channel_id",
            "isDeprecated": false
          },
          {
            "name": "unwrapped_denom",
            "isDeprecated": false
          },
          {
            "name": "wrapper",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_token_meta_wrapping_stddev_order_by",
        "inputFields": [
          {
            "name": "destination_channel_id",
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
        "name": "v2_token_meta_wrapping_stddev_pop_order_by",
        "inputFields": [
          {
            "name": "destination_channel_id",
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
        "name": "v2_token_meta_wrapping_stddev_samp_order_by",
        "inputFields": [
          {
            "name": "destination_channel_id",
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
        "name": "v2_token_meta_wrapping_sum_order_by",
        "inputFields": [
          {
            "name": "destination_channel_id",
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
        "name": "v2_token_meta_wrapping_var_pop_order_by",
        "inputFields": [
          {
            "name": "destination_channel_id",
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
        "name": "v2_token_meta_wrapping_var_samp_order_by",
        "inputFields": [
          {
            "name": "destination_channel_id",
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
        "name": "v2_token_meta_wrapping_variance_order_by",
        "inputFields": [
          {
            "name": "destination_channel_id",
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
        "name": "v2_tokens_args",
        "inputFields": [
          {
            "name": "p_denom",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_universal_chain_id",
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
        "name": "v2_traces_type",
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
              "name": "v2_chains_view"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "event_index",
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
          },
          {
            "name": "universal_chain_id",
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
        "name": "v2_traces_type_aggregate_order_by",
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
              "name": "v2_traces_type_max_order_by"
            }
          },
          {
            "name": "min",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_traces_type_min_order_by"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_traces_type_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_traces_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_traces_type_bool_exp"
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
                  "name": "v2_traces_type_bool_exp"
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
              "name": "v2_chains_view_bool_exp"
            }
          },
          {
            "name": "event_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "height",
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
          },
          {
            "name": "universal_chain_id",
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
        "name": "v2_traces_type_max_order_by",
        "inputFields": [
          {
            "name": "block_hash",
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
            "name": "type",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "universal_chain_id",
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
        "name": "v2_traces_type_min_order_by",
        "inputFields": [
          {
            "name": "block_hash",
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
            "name": "type",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "universal_chain_id",
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
        "name": "v2_traces_type_order_by",
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
              "name": "v2_chains_view_order_by"
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
          },
          {
            "name": "universal_chain_id",
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
        "name": "v2_traces_type_select_column",
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
          },
          {
            "name": "universal_chain_id",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "OBJECT",
        "name": "v2_transfer_type",
        "fields": [
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
            "name": "base_token_decimals",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "base_token_meta",
            "type": {
              "kind": "OBJECT",
              "name": "v2_token_meta"
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
            "name": "destination_chain",
            "type": {
              "kind": "OBJECT",
              "name": "v2_chains_view"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "destination_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "fee_amount",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "fee_token",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "fee_token_meta",
            "type": {
              "kind": "OBJECT",
              "name": "v2_token_meta"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "fee_type",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "packet_shape",
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
            "name": "quote_token_meta",
            "type": {
              "kind": "OBJECT",
              "name": "v2_token_meta"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "receiver_canonical",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "receiver_display",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "receiver_zkgm",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sender_canonical",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sender_display",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sender_zkgm",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sort_order",
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
              "name": "v2_chains_view"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "source_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "success",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "traces",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "OBJECT",
                  "name": "v2_traces_type"
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
                      "name": "v2_traces_type_select_column"
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
                      "name": "v2_traces_type_order_by"
                    }
                  }
                }
              },
              {
                "name": "where",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_traces_type_bool_exp"
                }
              }
            ],
            "isDeprecated": false
          },
          {
            "name": "transfer_index",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transfer_recv_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transfer_recv_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transfer_send_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transfer_send_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transfer_timeout_timestamp",
            "type": {
              "kind": "SCALAR",
              "name": "timestamptz"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "transfer_timeout_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "wrap_direction",
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
        "name": "v2_transfer_type_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_transfer_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_transfer_type_bool_exp"
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
                  "name": "v2_transfer_type_bool_exp"
                }
              }
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
            "name": "base_token_decimals",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "base_token_meta",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_bool_exp"
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
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_bool_exp"
            }
          },
          {
            "name": "destination_universal_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "fee_amount",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "fee_token",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "fee_token_meta",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_bool_exp"
            }
          },
          {
            "name": "fee_type",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "packet_shape",
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
            "name": "quote_token_meta",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_bool_exp"
            }
          },
          {
            "name": "receiver_canonical",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "receiver_display",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "receiver_zkgm",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "sender_canonical",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "sender_display",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "sender_zkgm",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "sort_order",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_bool_exp"
            }
          },
          {
            "name": "source_universal_chain_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "success",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Boolean_comparison_exp"
            }
          },
          {
            "name": "traces",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_traces_type_bool_exp"
            }
          },
          {
            "name": "transfer_index",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            }
          },
          {
            "name": "transfer_recv_timestamp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "transfer_recv_transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "transfer_send_timestamp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "transfer_send_transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "transfer_timeout_timestamp",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "timestamptz_comparison_exp"
            }
          },
          {
            "name": "transfer_timeout_transaction_hash",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "wrap_direction",
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
        "name": "v2_transfer_type_order_by",
        "inputFields": [
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
            "name": "base_token_decimals",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "base_token_meta",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_order_by"
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
            "name": "destination_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_order_by"
            }
          },
          {
            "name": "destination_universal_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "fee_amount",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "fee_token",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "fee_token_meta",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_order_by"
            }
          },
          {
            "name": "fee_type",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "packet_shape",
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
            "name": "quote_token_meta",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_token_meta_order_by"
            }
          },
          {
            "name": "receiver_canonical",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "receiver_display",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "receiver_zkgm",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "sender_canonical",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "sender_display",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "sender_zkgm",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "sort_order",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "source_chain",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_chains_view_order_by"
            }
          },
          {
            "name": "source_universal_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "success",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "traces_aggregate",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_traces_type_aggregate_order_by"
            }
          },
          {
            "name": "transfer_index",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "transfer_recv_timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "transfer_recv_transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "transfer_send_timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "transfer_send_transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "transfer_timeout_timestamp",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "transfer_timeout_transaction_hash",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "wrap_direction",
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
        "name": "v2_transfer_type_select_column",
        "enumValues": [
          {
            "name": "base_amount",
            "isDeprecated": false
          },
          {
            "name": "base_token",
            "isDeprecated": false
          },
          {
            "name": "base_token_decimals",
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
            "name": "destination_universal_chain_id",
            "isDeprecated": false
          },
          {
            "name": "fee_amount",
            "isDeprecated": false
          },
          {
            "name": "fee_token",
            "isDeprecated": false
          },
          {
            "name": "fee_type",
            "isDeprecated": false
          },
          {
            "name": "packet_hash",
            "isDeprecated": false
          },
          {
            "name": "packet_shape",
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
            "name": "receiver_canonical",
            "isDeprecated": false
          },
          {
            "name": "receiver_display",
            "isDeprecated": false
          },
          {
            "name": "receiver_zkgm",
            "isDeprecated": false
          },
          {
            "name": "sender_canonical",
            "isDeprecated": false
          },
          {
            "name": "sender_display",
            "isDeprecated": false
          },
          {
            "name": "sender_zkgm",
            "isDeprecated": false
          },
          {
            "name": "sort_order",
            "isDeprecated": false
          },
          {
            "name": "source_universal_chain_id",
            "isDeprecated": false
          },
          {
            "name": "success",
            "isDeprecated": false
          },
          {
            "name": "transfer_index",
            "isDeprecated": false
          },
          {
            "name": "transfer_recv_timestamp",
            "isDeprecated": false
          },
          {
            "name": "transfer_recv_transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "transfer_send_timestamp",
            "isDeprecated": false
          },
          {
            "name": "transfer_send_transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "transfer_timeout_timestamp",
            "isDeprecated": false
          },
          {
            "name": "transfer_timeout_transaction_hash",
            "isDeprecated": false
          },
          {
            "name": "wrap_direction",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_transfers_args",
        "inputFields": [
          {
            "name": "p_addresses_canonical",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          },
          {
            "name": "p_block_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_comparison",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_limit",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "p_packet_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_sort_order",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_source_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_transaction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_transfer_index",
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
        "name": "v2_util_get_address_types_for_display_address_args",
        "inputFields": [
          {
            "name": "p_chain_type",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_display_address",
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
        "name": "v2_util_get_address_types_for_display_address_type",
        "fields": [
          {
            "name": "canonical",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "display",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "zkgm",
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
        "name": "v2_util_get_address_types_for_display_address_type_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_util_get_address_types_for_display_address_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_util_get_address_types_for_display_address_type_bool_exp"
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
                  "name": "v2_util_get_address_types_for_display_address_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "canonical",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "display",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            }
          },
          {
            "name": "zkgm",
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
        "name": "v2_util_get_address_types_for_display_address_type_order_by",
        "inputFields": [
          {
            "name": "canonical",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "display",
            "type": {
              "kind": "ENUM",
              "name": "order_by"
            }
          },
          {
            "name": "zkgm",
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
        "name": "v2_util_get_address_types_for_display_address_type_select_column",
        "enumValues": [
          {
            "name": "canonical",
            "isDeprecated": false
          },
          {
            "name": "display",
            "isDeprecated": false
          },
          {
            "name": "zkgm",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "v2_util_get_transfer_request_details_args",
        "inputFields": [
          {
            "name": "p_base_token",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_destination_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_source_universal_chain_id",
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
        "name": "v2_util_get_transfer_request_details_type",
        "fields": [
          {
            "name": "already_exists",
            "type": {
              "kind": "SCALAR",
              "name": "Boolean"
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
            "name": "quote_token",
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
            "name": "wrap_direction",
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
        "name": "v2_util_get_transfer_request_details_type_bool_exp",
        "inputFields": [
          {
            "name": "_and",
            "type": {
              "kind": "LIST",
              "ofType": {
                "kind": "NON_NULL",
                "ofType": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_util_get_transfer_request_details_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "_not",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "v2_util_get_transfer_request_details_type_bool_exp"
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
                  "name": "v2_util_get_transfer_request_details_type_bool_exp"
                }
              }
            }
          },
          {
            "name": "already_exists",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Boolean_comparison_exp"
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
            "name": "quote_token",
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
            "name": "wrap_direction",
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
        "name": "v2_util_get_transfer_request_details_type_order_by",
        "inputFields": [
          {
            "name": "already_exists",
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
            "name": "quote_token",
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
            "name": "wrap_direction",
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
        "name": "v2_util_get_transfer_request_details_type_select_column",
        "enumValues": [
          {
            "name": "already_exists",
            "isDeprecated": false
          },
          {
            "name": "destination_channel_id",
            "isDeprecated": false
          },
          {
            "name": "quote_token",
            "isDeprecated": false
          },
          {
            "name": "source_channel_id",
            "isDeprecated": false
          },
          {
            "name": "wrap_direction",
            "isDeprecated": false
          }
        ]
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