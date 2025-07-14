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
      "name": "Query"
    },
    "mutationType": null,
    "subscriptionType": null,
    "types": [
      {
        "kind": "SCALAR",
        "name": "Boolean"
      },
      {
        "kind": "ENUM",
        "name": "ComparisonOp",
        "enumValues": [
          {
            "name": "lt",
            "isDeprecated": false
          },
          {
            "name": "gt",
            "isDeprecated": false
          }
        ]
      },
      {
        "kind": "SCALAR",
        "name": "DateTime"
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
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Int"
              }
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "SCALAR",
        "name": "JSON"
      },
      {
        "kind": "OBJECT",
        "name": "LatencyPercentiles",
        "fields": [
          {
            "name": "median",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Float"
              }
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "p5",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Float"
              }
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "p95",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "SCALAR",
                "name": "Float"
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
        "name": "NaiveDate"
      },
      {
        "kind": "ENUM",
        "name": "OrderBy",
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
        "name": "Query",
        "fields": [
          {
            "name": "dashboard_balance_current",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "dashboard_balance_current_type"
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
                    "name": "dashboard_balance_current_args"
                  }
                },
                "defaultValue": "{p_contract_address_canonical: null, p_phase: null, p_universal_chain_id: null, p_wallet_addresses_canonical: null}"
              }
            ],
            "isDeprecated": false
          },
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
                    "name": "dashboard_count_by_chain_args"
                  }
                },
                "defaultValue": "{p_addresses_dashboard: null, p_phase: null}"
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "dashboard_count_by_chain_type_order_by"
                },
                "defaultValue": "null"
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
                    "name": "dashboard_days_by_chain_args"
                  }
                },
                "defaultValue": "{p_addresses_dashboard: null}"
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "dashboard_days_by_chain_type_order_by"
                },
                "defaultValue": "null"
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
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "INPUT_OBJECT",
                    "name": "v2_chains_args"
                  }
                },
                "defaultValue": "{p_comparison: lt, p_limit: null, p_sort_order: null, p_universal_chain_id: null}"
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
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "INPUT_OBJECT",
                    "name": "v2_channels_args"
                  }
                },
                "defaultValue": "{p_comparison: lt, p_destination_channel_id: null, p_destination_client_id: null, p_destination_connection_id: null, p_destination_port_id: null, p_destination_universal_chain_id: null, p_limit: null, p_recommended: null, p_sort_order: null, p_source_channel_id: null, p_source_client_id: null, p_source_connection_id: null, p_source_port_id: null, p_source_universal_chain_id: null, p_tags: null, p_version: null}"
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
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "INPUT_OBJECT",
                    "name": "v2_clients_args"
                  }
                },
                "defaultValue": "{p_client_id: null, p_comparison: lt, p_counterparty_universal_chain_id: null, p_limit: null, p_sort_order: null, p_universal_chain_id: null}"
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
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "INPUT_OBJECT",
                    "name": "v2_connections_args"
                  }
                },
                "defaultValue": "{p_comparison: lt, p_destination_client_id: null, p_destination_connection_id: null, p_destination_universal_chain_id: null, p_limit: null, p_sort_order: null, p_source_client_id: null, p_source_connection_id: null, p_source_universal_chain_id: null}"
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
            "args": [],
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
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "v2_instructions",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_instruction_type"
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
                    "name": "v2_instructions_args"
                  }
                },
                "defaultValue": "{p_block_hash: null, p_comparison: lt, p_limit: null, p_multiplex_contract_address: null, p_multiplex_sender: null, p_network: null, p_packet_hash: null, p_sort_order: null, p_transaction_hash: null}"
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
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "INPUT_OBJECT",
                    "name": "v2_packets_args"
                  }
                },
                "defaultValue": "{p_block_hash: null, p_comparison: lt, p_destination_universal_chain_id: null, p_exceeding_sla: null, p_limit: null, p_network: null, p_packet_hash: null, p_packet_send_timestamp: null, p_sort_order: null, p_source_universal_chain_id: null, p_transaction_hash: null}"
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
            "args": [],
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
                },
                "defaultValue": "{p_destination_universal_chain_id: null, p_phase: null, p_source_universal_chain_id: null}"
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
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "INPUT_OBJECT",
                    "name": "v2_stats_packets_chain_args"
                  }
                },
                "defaultValue": "{p_days_back: null, p_destination_universal_chain_id: null, p_source_universal_chain_id: null}"
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
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "INPUT_OBJECT",
                    "name": "v2_stats_packets_daily_count_args"
                  }
                },
                "defaultValue": "{p_days_back: null}"
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
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "INPUT_OBJECT",
                    "name": "v2_stats_transfers_chain_args"
                  }
                },
                "defaultValue": "{p_days_back: null, p_destination_universal_chain_id: null, p_source_universal_chain_id: null}"
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
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "INPUT_OBJECT",
                    "name": "v2_stats_transfers_daily_count_args"
                  }
                },
                "defaultValue": "{p_days_back: null}"
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
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "INPUT_OBJECT",
                    "name": "v2_tokens_args"
                  }
                },
                "defaultValue": "{p_denom: null, p_universal_chain_id: null, p_whitelist: null}"
              },
              {
                "name": "order_by",
                "type": {
                  "kind": "INPUT_OBJECT",
                  "name": "v2_token_meta_order_by"
                },
                "defaultValue": "null"
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
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "INPUT_OBJECT",
                    "name": "v2_transfers_args"
                  }
                },
                "defaultValue": "{p_addresses_canonical: null, p_block_hash: null, p_comparison: lt, p_destination_universal_chain_id: null, p_limit: null, p_network: null, p_packet_hash: null, p_sort_order: null, p_source_universal_chain_id: null, p_transaction_hash: null, p_transfer_index: null}"
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
                },
                "defaultValue": "{p_display_address: null, p_chain_type: null}"
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
                },
                "defaultValue": "{p_source_universal_chain_id: null, p_destination_universal_chain_id: null, p_base_token: null}"
              }
            ],
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
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "dashboard_balance_current_args",
        "inputFields": [
          {
            "name": "p_contract_address_canonical",
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
            "name": "p_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_wallet_addresses_canonical",
            "type": {
              "kind": "SCALAR",
              "name": "jsonb"
            }
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "OBJECT",
        "name": "dashboard_balance_current_type",
        "fields": [
          {
            "name": "balance",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "balance_usd",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "contract_address_canonical",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "token",
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
          },
          {
            "name": "wallet_address_canonical",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "weighted_balance",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "weighted_balance_usd",
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
        "name": "dashboard_count_by_chain_args",
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
        "name": "dashboard_count_by_chain_type_order_by",
        "inputFields": [
          {
            "name": "phase",
            "type": {
              "kind": "ENUM",
              "name": "OrderBy"
            },
            "defaultValue": "null"
          },
          {
            "name": "universal_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "OrderBy"
            },
            "defaultValue": "null"
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "INPUT_OBJECT",
        "name": "dashboard_days_by_chain_args",
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
        "name": "dashboard_days_by_chain_type_order_by",
        "inputFields": [
          {
            "name": "universal_chain_id",
            "type": {
              "kind": "ENUM",
              "name": "OrderBy"
            },
            "defaultValue": "null"
          }
        ],
        "isOneOf": false
      },
      {
        "kind": "SCALAR",
        "name": "jsonb"
      },
      {
        "kind": "OBJECT",
        "name": "v2_chain_edition",
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
            "name": "name",
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
        "name": "v2_chain_features_bool_exp",
        "inputFields": [
          {
            "name": "environment",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "String_comparison_exp"
            },
            "defaultValue": "null"
          }
        ],
        "isOneOf": false
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
              "name": "DateTime"
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
            "name": "editions",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_chain_edition"
                  }
                }
              }
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
            "args": [],
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
                "name": "where",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "INPUT_OBJECT",
                    "name": "v2_chain_features_bool_exp"
                  }
                },
                "defaultValue": "{environment: null}"
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
            "name": "minter_address_display",
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
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "status",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v2_chain_status_type"
              }
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
        "name": "v2_chains_args",
        "inputFields": [
          {
            "name": "p_comparison",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "ENUM",
                "name": "ComparisonOp"
              }
            },
            "defaultValue": "lt"
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
        "name": "v2_channel_fee",
        "fields": [
          {
            "name": "action",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "fee",
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
        "name": "v2_channel_type",
        "fields": [
          {
            "name": "destination_chain",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v2_chain_type"
              }
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
            "name": "fees",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_channel_fee"
                  }
                }
              }
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "sla",
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
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v2_chain_type"
              }
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
        "name": "v2_channels_args",
        "inputFields": [
          {
            "name": "p_comparison",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "ENUM",
                "name": "ComparisonOp"
              }
            },
            "defaultValue": "lt"
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
        "name": "v2_client_status_type",
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
            "name": "counterparty_height",
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
              "name": "DateTime"
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
          }
        ],
        "interfaces": []
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
            "name": "status",
            "type": {
              "kind": "OBJECT",
              "name": "v2_client_status_type"
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
              "kind": "NON_NULL",
              "ofType": {
                "kind": "ENUM",
                "name": "ComparisonOp"
              }
            },
            "defaultValue": "lt"
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
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v2_chain_type"
              }
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
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v2_chain_type"
              }
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
        "name": "v2_connections_args",
        "inputFields": [
          {
            "name": "p_comparison",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "ENUM",
                "name": "ComparisonOp"
              }
            },
            "defaultValue": "lt"
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
              "name": "DateTime"
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
        "name": "v2_instruction_type",
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
              "name": "JSON"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "decoded_flattened",
            "type": {
              "kind": "SCALAR",
              "name": "JSON"
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
                "name": "v2_chain_type"
              }
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
            "name": "instruction",
            "type": {
              "kind": "SCALAR",
              "name": "JSON"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "instruction_hash",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "instruction_index",
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
            "name": "instruction_type",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "internal_destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
            "name": "opcode",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "operand",
            "type": {
              "kind": "SCALAR",
              "name": "JSON"
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
              "name": "DateTime"
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
              "name": "DateTime"
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
              "name": "DateTime"
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
              "name": "DateTime"
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
            "name": "path",
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
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v2_chain_type"
              }
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
            "name": "structure",
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
            "name": "version",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            },
            "args": [],
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
              "name": "DateTime"
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
        "name": "v2_instructions_args",
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
              "kind": "NON_NULL",
              "ofType": {
                "kind": "ENUM",
                "name": "ComparisonOp"
              }
            },
            "defaultValue": "lt"
          },
          {
            "name": "p_limit",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
            }
          },
          {
            "name": "p_multiplex_contract_address",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_multiplex_sender",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_network",
            "type": {
              "kind": "SCALAR",
              "name": "String"
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
              "name": "JSON"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "decoded_flattened",
            "type": {
              "kind": "SCALAR",
              "name": "JSON"
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
                "name": "v2_chain_type"
              }
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
            "name": "internal_destination_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "Int"
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
              "name": "DateTime"
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
              "name": "DateTime"
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
              "name": "DateTime"
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
              "name": "DateTime"
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
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v2_chain_type"
              }
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
            "name": "structure",
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
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_traces_type"
                  }
                }
              }
            },
            "args": [],
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
              "name": "DateTime"
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
              "kind": "NON_NULL",
              "ofType": {
                "kind": "ENUM",
                "name": "ComparisonOp"
              }
            },
            "defaultValue": "lt"
          },
          {
            "name": "p_destination_universal_chain_id",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_exceeding_sla",
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
            "name": "p_network",
            "type": {
              "kind": "SCALAR",
              "name": "String"
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
              "name": "DateTime"
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
              "name": "NaiveDate"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
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
              "kind": "OBJECT",
              "name": "LatencyPercentiles"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "secs_until_packet_recv",
            "type": {
              "kind": "OBJECT",
              "name": "LatencyPercentiles"
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "secs_until_write_ack",
            "type": {
              "kind": "OBJECT",
              "name": "LatencyPercentiles"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
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
              "name": "NaiveDate"
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
              "name": "NaiveDate"
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
        "kind": "OBJECT",
        "name": "v2_token_meta",
        "fields": [
          {
            "name": "bucket",
            "type": {
              "kind": "OBJECT",
              "name": "v2_token_meta_bucket"
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
                "name": "v2_chain_type"
              }
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
                "name": "where",
                "type": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "INPUT_OBJECT",
                    "name": "v2_token_meta_wrapping_bool_exp"
                  }
                },
                "defaultValue": "{destination_channel_id: null}"
              }
            ],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v2_token_meta_bucket",
        "fields": [
          {
            "name": "capacity",
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
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v2_chain_type"
              }
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
            "name": "refill_rate",
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
        "name": "v2_token_meta_order_by",
        "inputFields": [
          {
            "name": "rank",
            "type": {
              "kind": "ENUM",
              "name": "OrderBy"
            },
            "defaultValue": "null"
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
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v2_chain_type"
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
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v2_token_meta_sources"
              }
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
              "name": "DateTime"
            },
            "args": [],
            "isDeprecated": false
          }
        ],
        "interfaces": []
      },
      {
        "kind": "OBJECT",
        "name": "v2_token_meta_representations",
        "fields": [
          {
            "name": "chain",
            "type": {
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v2_chain_type"
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
          }
        ],
        "interfaces": []
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
            "name": "index",
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
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v2_chain_type"
              }
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
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v2_chain_type"
              }
            },
            "args": [],
            "isDeprecated": false
          },
          {
            "name": "wrapped_denom",
            "type": {
              "kind": "SCALAR",
              "name": "String"
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
        "name": "v2_token_meta_wrapping_bool_exp",
        "inputFields": [
          {
            "name": "destination_channel_id",
            "type": {
              "kind": "INPUT_OBJECT",
              "name": "Int_comparison_exp"
            },
            "defaultValue": "null"
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
          },
          {
            "name": "p_whitelist",
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
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v2_chain_type"
              }
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
              "name": "DateTime"
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
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v2_chain_type"
              }
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
              "kind": "NON_NULL",
              "ofType": {
                "kind": "OBJECT",
                "name": "v2_chain_type"
              }
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
              "kind": "NON_NULL",
              "ofType": {
                "kind": "LIST",
                "ofType": {
                  "kind": "NON_NULL",
                  "ofType": {
                    "kind": "OBJECT",
                    "name": "v2_traces_type"
                  }
                }
              }
            },
            "args": [],
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
              "name": "DateTime"
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
              "name": "DateTime"
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
              "name": "DateTime"
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
              "kind": "NON_NULL",
              "ofType": {
                "kind": "ENUM",
                "name": "ComparisonOp"
              }
            },
            "defaultValue": "lt"
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
            "name": "p_network",
            "type": {
              "kind": "SCALAR",
              "name": "String"
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
            "name": "p_display_address",
            "type": {
              "kind": "SCALAR",
              "name": "String"
            }
          },
          {
            "name": "p_chain_type",
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
        "name": "v2_util_get_transfer_request_details_args",
        "inputFields": [
          {
            "name": "p_source_universal_chain_id",
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
            "name": "p_base_token",
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