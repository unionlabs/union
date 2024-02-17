export const ibcHandlerAbi = <const>[
  {
    type: 'function',
    name: 'COMMITMENT_PREFIX',
    inputs: [],
    outputs: [
      {
        name: '',
        type: 'string',
        internalType: 'string'
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'acknowledgePacket',
    inputs: [
      {
        name: 'msg_',
        type: 'tuple',
        internalType: 'struct IBCMsgs.MsgPacketAcknowledgement',
        components: [
          {
            name: 'packet',
            type: 'tuple',
            internalType: 'struct IbcCoreChannelV1Packet.Data',
            components: [
              {
                name: 'sequence',
                type: 'uint64',
                internalType: 'uint64'
              },
              {
                name: 'source_port',
                type: 'string',
                internalType: 'string'
              },
              {
                name: 'source_channel',
                type: 'string',
                internalType: 'string'
              },
              {
                name: 'destination_port',
                type: 'string',
                internalType: 'string'
              },
              {
                name: 'destination_channel',
                type: 'string',
                internalType: 'string'
              },
              {
                name: 'data',
                type: 'bytes',
                internalType: 'bytes'
              },
              {
                name: 'timeout_height',
                type: 'tuple',
                internalType: 'struct IbcCoreClientV1Height.Data',
                components: [
                  {
                    name: 'revision_number',
                    type: 'uint64',
                    internalType: 'uint64'
                  },
                  {
                    name: 'revision_height',
                    type: 'uint64',
                    internalType: 'uint64'
                  }
                ]
              },
              {
                name: 'timeout_timestamp',
                type: 'uint64',
                internalType: 'uint64'
              }
            ]
          },
          {
            name: 'acknowledgement',
            type: 'bytes',
            internalType: 'bytes'
          },
          {
            name: 'proof',
            type: 'bytes',
            internalType: 'bytes'
          },
          {
            name: 'proofHeight',
            type: 'tuple',
            internalType: 'struct IbcCoreClientV1Height.Data',
            components: [
              {
                name: 'revision_number',
                type: 'uint64',
                internalType: 'uint64'
              },
              {
                name: 'revision_height',
                type: 'uint64',
                internalType: 'uint64'
              }
            ]
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'bindPort',
    inputs: [
      {
        name: 'portId',
        type: 'string',
        internalType: 'string'
      },
      {
        name: 'moduleAddress',
        type: 'address',
        internalType: 'address'
      }
    ],
    outputs: [],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'capabilities',
    inputs: [
      {
        name: '',
        type: 'string',
        internalType: 'string'
      }
    ],
    outputs: [
      {
        name: '',
        type: 'address',
        internalType: 'address'
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'channelCapabilityPath',
    inputs: [
      {
        name: 'portId',
        type: 'string',
        internalType: 'string'
      },
      {
        name: 'channelId',
        type: 'string',
        internalType: 'string'
      }
    ],
    outputs: [
      {
        name: '',
        type: 'string',
        internalType: 'string'
      }
    ],
    stateMutability: 'pure'
  },
  {
    type: 'function',
    name: 'channelCloseConfirm',
    inputs: [
      {
        name: 'msg_',
        type: 'tuple',
        internalType: 'struct IBCMsgs.MsgChannelCloseConfirm',
        components: [
          {
            name: 'portId',
            type: 'string',
            internalType: 'string'
          },
          {
            name: 'channelId',
            type: 'string',
            internalType: 'string'
          },
          {
            name: 'proofInit',
            type: 'bytes',
            internalType: 'bytes'
          },
          {
            name: 'proofHeight',
            type: 'tuple',
            internalType: 'struct IbcCoreClientV1Height.Data',
            components: [
              {
                name: 'revision_number',
                type: 'uint64',
                internalType: 'uint64'
              },
              {
                name: 'revision_height',
                type: 'uint64',
                internalType: 'uint64'
              }
            ]
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'channelCloseInit',
    inputs: [
      {
        name: 'msg_',
        type: 'tuple',
        internalType: 'struct IBCMsgs.MsgChannelCloseInit',
        components: [
          {
            name: 'portId',
            type: 'string',
            internalType: 'string'
          },
          {
            name: 'channelId',
            type: 'string',
            internalType: 'string'
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'channelOpenAck',
    inputs: [
      {
        name: 'msg_',
        type: 'tuple',
        internalType: 'struct IBCMsgs.MsgChannelOpenAck',
        components: [
          {
            name: 'portId',
            type: 'string',
            internalType: 'string'
          },
          {
            name: 'channelId',
            type: 'string',
            internalType: 'string'
          },
          {
            name: 'counterpartyVersion',
            type: 'string',
            internalType: 'string'
          },
          {
            name: 'counterpartyChannelId',
            type: 'string',
            internalType: 'string'
          },
          {
            name: 'proofTry',
            type: 'bytes',
            internalType: 'bytes'
          },
          {
            name: 'proofHeight',
            type: 'tuple',
            internalType: 'struct IbcCoreClientV1Height.Data',
            components: [
              {
                name: 'revision_number',
                type: 'uint64',
                internalType: 'uint64'
              },
              {
                name: 'revision_height',
                type: 'uint64',
                internalType: 'uint64'
              }
            ]
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'channelOpenConfirm',
    inputs: [
      {
        name: 'msg_',
        type: 'tuple',
        internalType: 'struct IBCMsgs.MsgChannelOpenConfirm',
        components: [
          {
            name: 'portId',
            type: 'string',
            internalType: 'string'
          },
          {
            name: 'channelId',
            type: 'string',
            internalType: 'string'
          },
          {
            name: 'proofAck',
            type: 'bytes',
            internalType: 'bytes'
          },
          {
            name: 'proofHeight',
            type: 'tuple',
            internalType: 'struct IbcCoreClientV1Height.Data',
            components: [
              {
                name: 'revision_number',
                type: 'uint64',
                internalType: 'uint64'
              },
              {
                name: 'revision_height',
                type: 'uint64',
                internalType: 'uint64'
              }
            ]
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'channelOpenInit',
    inputs: [
      {
        name: 'msg_',
        type: 'tuple',
        internalType: 'struct IBCMsgs.MsgChannelOpenInit',
        components: [
          {
            name: 'portId',
            type: 'string',
            internalType: 'string'
          },
          {
            name: 'channel',
            type: 'tuple',
            internalType: 'struct IbcCoreChannelV1Channel.Data',
            components: [
              {
                name: 'state',
                type: 'uint8',
                internalType: 'enum IbcCoreChannelV1GlobalEnums.State'
              },
              {
                name: 'ordering',
                type: 'uint8',
                internalType: 'enum IbcCoreChannelV1GlobalEnums.Order'
              },
              {
                name: 'counterparty',
                type: 'tuple',
                internalType: 'struct IbcCoreChannelV1Counterparty.Data',
                components: [
                  {
                    name: 'port_id',
                    type: 'string',
                    internalType: 'string'
                  },
                  {
                    name: 'channel_id',
                    type: 'string',
                    internalType: 'string'
                  }
                ]
              },
              {
                name: 'connection_hops',
                type: 'string[]',
                internalType: 'string[]'
              },
              {
                name: 'version',
                type: 'string',
                internalType: 'string'
              }
            ]
          }
        ]
      }
    ],
    outputs: [
      {
        name: '',
        type: 'string',
        internalType: 'string'
      }
    ],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'channelOpenTry',
    inputs: [
      {
        name: 'msg_',
        type: 'tuple',
        internalType: 'struct IBCMsgs.MsgChannelOpenTry',
        components: [
          {
            name: 'portId',
            type: 'string',
            internalType: 'string'
          },
          {
            name: 'channel',
            type: 'tuple',
            internalType: 'struct IbcCoreChannelV1Channel.Data',
            components: [
              {
                name: 'state',
                type: 'uint8',
                internalType: 'enum IbcCoreChannelV1GlobalEnums.State'
              },
              {
                name: 'ordering',
                type: 'uint8',
                internalType: 'enum IbcCoreChannelV1GlobalEnums.Order'
              },
              {
                name: 'counterparty',
                type: 'tuple',
                internalType: 'struct IbcCoreChannelV1Counterparty.Data',
                components: [
                  {
                    name: 'port_id',
                    type: 'string',
                    internalType: 'string'
                  },
                  {
                    name: 'channel_id',
                    type: 'string',
                    internalType: 'string'
                  }
                ]
              },
              {
                name: 'connection_hops',
                type: 'string[]',
                internalType: 'string[]'
              },
              {
                name: 'version',
                type: 'string',
                internalType: 'string'
              }
            ]
          },
          {
            name: 'counterpartyVersion',
            type: 'string',
            internalType: 'string'
          },
          {
            name: 'proofInit',
            type: 'bytes',
            internalType: 'bytes'
          },
          {
            name: 'proofHeight',
            type: 'tuple',
            internalType: 'struct IbcCoreClientV1Height.Data',
            components: [
              {
                name: 'revision_number',
                type: 'uint64',
                internalType: 'uint64'
              },
              {
                name: 'revision_height',
                type: 'uint64',
                internalType: 'uint64'
              }
            ]
          }
        ]
      }
    ],
    outputs: [
      {
        name: '',
        type: 'string',
        internalType: 'string'
      }
    ],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'channels',
    inputs: [
      {
        name: '',
        type: 'string',
        internalType: 'string'
      },
      {
        name: '',
        type: 'string',
        internalType: 'string'
      }
    ],
    outputs: [
      {
        name: 'state',
        type: 'uint8',
        internalType: 'enum IbcCoreChannelV1GlobalEnums.State'
      },
      {
        name: 'ordering',
        type: 'uint8',
        internalType: 'enum IbcCoreChannelV1GlobalEnums.Order'
      },
      {
        name: 'counterparty',
        type: 'tuple',
        internalType: 'struct IbcCoreChannelV1Counterparty.Data',
        components: [
          {
            name: 'port_id',
            type: 'string',
            internalType: 'string'
          },
          {
            name: 'channel_id',
            type: 'string',
            internalType: 'string'
          }
        ]
      },
      {
        name: 'version',
        type: 'string',
        internalType: 'string'
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'clientImpls',
    inputs: [
      {
        name: '',
        type: 'string',
        internalType: 'string'
      }
    ],
    outputs: [
      {
        name: '',
        type: 'address',
        internalType: 'address'
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'clientRegistry',
    inputs: [
      {
        name: '',
        type: 'string',
        internalType: 'string'
      }
    ],
    outputs: [
      {
        name: '',
        type: 'address',
        internalType: 'address'
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'clientTypes',
    inputs: [
      {
        name: '',
        type: 'string',
        internalType: 'string'
      }
    ],
    outputs: [
      {
        name: '',
        type: 'string',
        internalType: 'string'
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'commitments',
    inputs: [
      {
        name: '',
        type: 'bytes32',
        internalType: 'bytes32'
      }
    ],
    outputs: [
      {
        name: '',
        type: 'bytes32',
        internalType: 'bytes32'
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'connectionOpenAck',
    inputs: [
      {
        name: 'msg_',
        type: 'tuple',
        internalType: 'struct IBCMsgs.MsgConnectionOpenAck',
        components: [
          {
            name: 'connectionId',
            type: 'string',
            internalType: 'string'
          },
          {
            name: 'clientStateBytes',
            type: 'bytes',
            internalType: 'bytes'
          },
          {
            name: 'version',
            type: 'tuple',
            internalType: 'struct IbcCoreConnectionV1Version.Data',
            components: [
              {
                name: 'identifier',
                type: 'string',
                internalType: 'string'
              },
              {
                name: 'features',
                type: 'string[]',
                internalType: 'string[]'
              }
            ]
          },
          {
            name: 'counterpartyConnectionID',
            type: 'string',
            internalType: 'string'
          },
          {
            name: 'proofTry',
            type: 'bytes',
            internalType: 'bytes'
          },
          {
            name: 'proofClient',
            type: 'bytes',
            internalType: 'bytes'
          },
          {
            name: 'proofConsensus',
            type: 'bytes',
            internalType: 'bytes'
          },
          {
            name: 'proofHeight',
            type: 'tuple',
            internalType: 'struct IbcCoreClientV1Height.Data',
            components: [
              {
                name: 'revision_number',
                type: 'uint64',
                internalType: 'uint64'
              },
              {
                name: 'revision_height',
                type: 'uint64',
                internalType: 'uint64'
              }
            ]
          },
          {
            name: 'consensusHeight',
            type: 'tuple',
            internalType: 'struct IbcCoreClientV1Height.Data',
            components: [
              {
                name: 'revision_number',
                type: 'uint64',
                internalType: 'uint64'
              },
              {
                name: 'revision_height',
                type: 'uint64',
                internalType: 'uint64'
              }
            ]
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'connectionOpenConfirm',
    inputs: [
      {
        name: 'msg_',
        type: 'tuple',
        internalType: 'struct IBCMsgs.MsgConnectionOpenConfirm',
        components: [
          {
            name: 'connectionId',
            type: 'string',
            internalType: 'string'
          },
          {
            name: 'proofAck',
            type: 'bytes',
            internalType: 'bytes'
          },
          {
            name: 'proofHeight',
            type: 'tuple',
            internalType: 'struct IbcCoreClientV1Height.Data',
            components: [
              {
                name: 'revision_number',
                type: 'uint64',
                internalType: 'uint64'
              },
              {
                name: 'revision_height',
                type: 'uint64',
                internalType: 'uint64'
              }
            ]
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'connectionOpenInit',
    inputs: [
      {
        name: 'msg_',
        type: 'tuple',
        internalType: 'struct IBCMsgs.MsgConnectionOpenInit',
        components: [
          {
            name: 'clientId',
            type: 'string',
            internalType: 'string'
          },
          {
            name: 'counterparty',
            type: 'tuple',
            internalType: 'struct IbcCoreConnectionV1Counterparty.Data',
            components: [
              {
                name: 'client_id',
                type: 'string',
                internalType: 'string'
              },
              {
                name: 'connection_id',
                type: 'string',
                internalType: 'string'
              },
              {
                name: 'prefix',
                type: 'tuple',
                internalType: 'struct IbcCoreCommitmentV1MerklePrefix.Data',
                components: [
                  {
                    name: 'key_prefix',
                    type: 'bytes',
                    internalType: 'bytes'
                  }
                ]
              }
            ]
          },
          {
            name: 'delayPeriod',
            type: 'uint64',
            internalType: 'uint64'
          }
        ]
      }
    ],
    outputs: [
      {
        name: '',
        type: 'string',
        internalType: 'string'
      }
    ],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'connectionOpenTry',
    inputs: [
      {
        name: 'msg_',
        type: 'tuple',
        internalType: 'struct IBCMsgs.MsgConnectionOpenTry',
        components: [
          {
            name: 'counterparty',
            type: 'tuple',
            internalType: 'struct IbcCoreConnectionV1Counterparty.Data',
            components: [
              {
                name: 'client_id',
                type: 'string',
                internalType: 'string'
              },
              {
                name: 'connection_id',
                type: 'string',
                internalType: 'string'
              },
              {
                name: 'prefix',
                type: 'tuple',
                internalType: 'struct IbcCoreCommitmentV1MerklePrefix.Data',
                components: [
                  {
                    name: 'key_prefix',
                    type: 'bytes',
                    internalType: 'bytes'
                  }
                ]
              }
            ]
          },
          {
            name: 'delayPeriod',
            type: 'uint64',
            internalType: 'uint64'
          },
          {
            name: 'clientId',
            type: 'string',
            internalType: 'string'
          },
          {
            name: 'clientStateBytes',
            type: 'bytes',
            internalType: 'bytes'
          },
          {
            name: 'counterpartyVersions',
            type: 'tuple[]',
            internalType: 'struct IbcCoreConnectionV1Version.Data[]',
            components: [
              {
                name: 'identifier',
                type: 'string',
                internalType: 'string'
              },
              {
                name: 'features',
                type: 'string[]',
                internalType: 'string[]'
              }
            ]
          },
          {
            name: 'proofInit',
            type: 'bytes',
            internalType: 'bytes'
          },
          {
            name: 'proofClient',
            type: 'bytes',
            internalType: 'bytes'
          },
          {
            name: 'proofConsensus',
            type: 'bytes',
            internalType: 'bytes'
          },
          {
            name: 'proofHeight',
            type: 'tuple',
            internalType: 'struct IbcCoreClientV1Height.Data',
            components: [
              {
                name: 'revision_number',
                type: 'uint64',
                internalType: 'uint64'
              },
              {
                name: 'revision_height',
                type: 'uint64',
                internalType: 'uint64'
              }
            ]
          },
          {
            name: 'consensusHeight',
            type: 'tuple',
            internalType: 'struct IbcCoreClientV1Height.Data',
            components: [
              {
                name: 'revision_number',
                type: 'uint64',
                internalType: 'uint64'
              },
              {
                name: 'revision_height',
                type: 'uint64',
                internalType: 'uint64'
              }
            ]
          }
        ]
      }
    ],
    outputs: [
      {
        name: '',
        type: 'string',
        internalType: 'string'
      }
    ],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'connections',
    inputs: [
      {
        name: '',
        type: 'string',
        internalType: 'string'
      }
    ],
    outputs: [
      {
        name: 'client_id',
        type: 'string',
        internalType: 'string'
      },
      {
        name: 'state',
        type: 'uint8',
        internalType: 'enum IbcCoreConnectionV1GlobalEnums.State'
      },
      {
        name: 'counterparty',
        type: 'tuple',
        internalType: 'struct IbcCoreConnectionV1Counterparty.Data',
        components: [
          {
            name: 'client_id',
            type: 'string',
            internalType: 'string'
          },
          {
            name: 'connection_id',
            type: 'string',
            internalType: 'string'
          },
          {
            name: 'prefix',
            type: 'tuple',
            internalType: 'struct IbcCoreCommitmentV1MerklePrefix.Data',
            components: [
              {
                name: 'key_prefix',
                type: 'bytes',
                internalType: 'bytes'
              }
            ]
          }
        ]
      },
      {
        name: 'delay_period',
        type: 'uint64',
        internalType: 'uint64'
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'createClient',
    inputs: [
      {
        name: 'msg_',
        type: 'tuple',
        internalType: 'struct IBCMsgs.MsgCreateClient',
        components: [
          {
            name: 'clientType',
            type: 'string',
            internalType: 'string'
          },
          {
            name: 'clientStateBytes',
            type: 'bytes',
            internalType: 'bytes'
          },
          {
            name: 'consensusStateBytes',
            type: 'bytes',
            internalType: 'bytes'
          }
        ]
      }
    ],
    outputs: [
      {
        name: '',
        type: 'string',
        internalType: 'string'
      }
    ],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'getChannel',
    inputs: [
      {
        name: 'portId',
        type: 'string',
        internalType: 'string'
      },
      {
        name: 'channelId',
        type: 'string',
        internalType: 'string'
      }
    ],
    outputs: [
      {
        name: '',
        type: 'tuple',
        internalType: 'struct IbcCoreChannelV1Channel.Data',
        components: [
          {
            name: 'state',
            type: 'uint8',
            internalType: 'enum IbcCoreChannelV1GlobalEnums.State'
          },
          {
            name: 'ordering',
            type: 'uint8',
            internalType: 'enum IbcCoreChannelV1GlobalEnums.Order'
          },
          {
            name: 'counterparty',
            type: 'tuple',
            internalType: 'struct IbcCoreChannelV1Counterparty.Data',
            components: [
              {
                name: 'port_id',
                type: 'string',
                internalType: 'string'
              },
              {
                name: 'channel_id',
                type: 'string',
                internalType: 'string'
              }
            ]
          },
          {
            name: 'connection_hops',
            type: 'string[]',
            internalType: 'string[]'
          },
          {
            name: 'version',
            type: 'string',
            internalType: 'string'
          }
        ]
      },
      {
        name: '',
        type: 'bool',
        internalType: 'bool'
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'getClient',
    inputs: [
      {
        name: 'clientId',
        type: 'string',
        internalType: 'string'
      }
    ],
    outputs: [
      {
        name: '',
        type: 'address',
        internalType: 'contract ILightClient'
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'getClientState',
    inputs: [
      {
        name: 'clientId',
        type: 'string',
        internalType: 'string'
      }
    ],
    outputs: [
      {
        name: '',
        type: 'bytes',
        internalType: 'bytes'
      },
      {
        name: '',
        type: 'bool',
        internalType: 'bool'
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'getConnection',
    inputs: [
      {
        name: 'connectionId',
        type: 'string',
        internalType: 'string'
      }
    ],
    outputs: [
      {
        name: '',
        type: 'tuple',
        internalType: 'struct IbcCoreConnectionV1ConnectionEnd.Data',
        components: [
          {
            name: 'client_id',
            type: 'string',
            internalType: 'string'
          },
          {
            name: 'versions',
            type: 'tuple[]',
            internalType: 'struct IbcCoreConnectionV1Version.Data[]',
            components: [
              {
                name: 'identifier',
                type: 'string',
                internalType: 'string'
              },
              {
                name: 'features',
                type: 'string[]',
                internalType: 'string[]'
              }
            ]
          },
          {
            name: 'state',
            type: 'uint8',
            internalType: 'enum IbcCoreConnectionV1GlobalEnums.State'
          },
          {
            name: 'counterparty',
            type: 'tuple',
            internalType: 'struct IbcCoreConnectionV1Counterparty.Data',
            components: [
              {
                name: 'client_id',
                type: 'string',
                internalType: 'string'
              },
              {
                name: 'connection_id',
                type: 'string',
                internalType: 'string'
              },
              {
                name: 'prefix',
                type: 'tuple',
                internalType: 'struct IbcCoreCommitmentV1MerklePrefix.Data',
                components: [
                  {
                    name: 'key_prefix',
                    type: 'bytes',
                    internalType: 'bytes'
                  }
                ]
              }
            ]
          },
          {
            name: 'delay_period',
            type: 'uint64',
            internalType: 'uint64'
          }
        ]
      },
      {
        name: '',
        type: 'bool',
        internalType: 'bool'
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'getConsensusState',
    inputs: [
      {
        name: 'clientId',
        type: 'string',
        internalType: 'string'
      },
      {
        name: 'height',
        type: 'tuple',
        internalType: 'struct IbcCoreClientV1Height.Data',
        components: [
          {
            name: 'revision_number',
            type: 'uint64',
            internalType: 'uint64'
          },
          {
            name: 'revision_height',
            type: 'uint64',
            internalType: 'uint64'
          }
        ]
      }
    ],
    outputs: [
      {
        name: 'consensusStateBytes',
        type: 'bytes',
        internalType: 'bytes'
      },
      {
        name: '',
        type: 'bool',
        internalType: 'bool'
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'getHashedPacketAcknowledgementCommitment',
    inputs: [
      {
        name: 'portId',
        type: 'string',
        internalType: 'string'
      },
      {
        name: 'channelId',
        type: 'string',
        internalType: 'string'
      },
      {
        name: 'sequence',
        type: 'uint64',
        internalType: 'uint64'
      }
    ],
    outputs: [
      {
        name: '',
        type: 'bytes32',
        internalType: 'bytes32'
      },
      {
        name: '',
        type: 'bool',
        internalType: 'bool'
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'getHashedPacketCommitment',
    inputs: [
      {
        name: 'portId',
        type: 'string',
        internalType: 'string'
      },
      {
        name: 'channelId',
        type: 'string',
        internalType: 'string'
      },
      {
        name: 'sequence',
        type: 'uint64',
        internalType: 'uint64'
      }
    ],
    outputs: [
      {
        name: '',
        type: 'bytes32',
        internalType: 'bytes32'
      },
      {
        name: '',
        type: 'bool',
        internalType: 'bool'
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'getNextSequenceSend',
    inputs: [
      {
        name: 'portId',
        type: 'string',
        internalType: 'string'
      },
      {
        name: 'channelId',
        type: 'string',
        internalType: 'string'
      }
    ],
    outputs: [
      {
        name: '',
        type: 'uint64',
        internalType: 'uint64'
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'hasPacketReceipt',
    inputs: [
      {
        name: 'portId',
        type: 'string',
        internalType: 'string'
      },
      {
        name: 'channelId',
        type: 'string',
        internalType: 'string'
      },
      {
        name: 'sequence',
        type: 'uint64',
        internalType: 'uint64'
      }
    ],
    outputs: [
      {
        name: '',
        type: 'bool',
        internalType: 'bool'
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'nextChannelSequence',
    inputs: [],
    outputs: [
      {
        name: '',
        type: 'uint64',
        internalType: 'uint64'
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'nextClientSequence',
    inputs: [],
    outputs: [
      {
        name: '',
        type: 'uint64',
        internalType: 'uint64'
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'nextConnectionSequence',
    inputs: [],
    outputs: [
      {
        name: '',
        type: 'uint64',
        internalType: 'uint64'
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'nextSequenceAcks',
    inputs: [
      {
        name: '',
        type: 'string',
        internalType: 'string'
      },
      {
        name: '',
        type: 'string',
        internalType: 'string'
      }
    ],
    outputs: [
      {
        name: '',
        type: 'uint64',
        internalType: 'uint64'
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'nextSequenceRecvs',
    inputs: [
      {
        name: '',
        type: 'string',
        internalType: 'string'
      },
      {
        name: '',
        type: 'string',
        internalType: 'string'
      }
    ],
    outputs: [
      {
        name: '',
        type: 'uint64',
        internalType: 'uint64'
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'nextSequenceSends',
    inputs: [
      {
        name: '',
        type: 'string',
        internalType: 'string'
      },
      {
        name: '',
        type: 'string',
        internalType: 'string'
      }
    ],
    outputs: [
      {
        name: '',
        type: 'uint64',
        internalType: 'uint64'
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'packetReceipts',
    inputs: [
      {
        name: '',
        type: 'string',
        internalType: 'string'
      },
      {
        name: '',
        type: 'string',
        internalType: 'string'
      },
      {
        name: '',
        type: 'uint64',
        internalType: 'uint64'
      }
    ],
    outputs: [
      {
        name: '',
        type: 'uint8',
        internalType: 'uint8'
      }
    ],
    stateMutability: 'view'
  },
  {
    type: 'function',
    name: 'portCapabilityPath',
    inputs: [
      {
        name: 'portId',
        type: 'string',
        internalType: 'string'
      }
    ],
    outputs: [
      {
        name: '',
        type: 'string',
        internalType: 'string'
      }
    ],
    stateMutability: 'pure'
  },
  {
    type: 'function',
    name: 'recvPacket',
    inputs: [
      {
        name: 'msg_',
        type: 'tuple',
        internalType: 'struct IBCMsgs.MsgPacketRecv',
        components: [
          {
            name: 'packet',
            type: 'tuple',
            internalType: 'struct IbcCoreChannelV1Packet.Data',
            components: [
              {
                name: 'sequence',
                type: 'uint64',
                internalType: 'uint64'
              },
              {
                name: 'source_port',
                type: 'string',
                internalType: 'string'
              },
              {
                name: 'source_channel',
                type: 'string',
                internalType: 'string'
              },
              {
                name: 'destination_port',
                type: 'string',
                internalType: 'string'
              },
              {
                name: 'destination_channel',
                type: 'string',
                internalType: 'string'
              },
              {
                name: 'data',
                type: 'bytes',
                internalType: 'bytes'
              },
              {
                name: 'timeout_height',
                type: 'tuple',
                internalType: 'struct IbcCoreClientV1Height.Data',
                components: [
                  {
                    name: 'revision_number',
                    type: 'uint64',
                    internalType: 'uint64'
                  },
                  {
                    name: 'revision_height',
                    type: 'uint64',
                    internalType: 'uint64'
                  }
                ]
              },
              {
                name: 'timeout_timestamp',
                type: 'uint64',
                internalType: 'uint64'
              }
            ]
          },
          {
            name: 'proof',
            type: 'bytes',
            internalType: 'bytes'
          },
          {
            name: 'proofHeight',
            type: 'tuple',
            internalType: 'struct IbcCoreClientV1Height.Data',
            components: [
              {
                name: 'revision_number',
                type: 'uint64',
                internalType: 'uint64'
              },
              {
                name: 'revision_height',
                type: 'uint64',
                internalType: 'uint64'
              }
            ]
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'registerClient',
    inputs: [
      {
        name: 'clientType',
        type: 'string',
        internalType: 'string'
      },
      {
        name: 'client',
        type: 'address',
        internalType: 'contract ILightClient'
      }
    ],
    outputs: [],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'sendPacket',
    inputs: [
      {
        name: 'sourcePort',
        type: 'string',
        internalType: 'string'
      },
      {
        name: 'sourceChannel',
        type: 'string',
        internalType: 'string'
      },
      {
        name: 'timeoutHeight',
        type: 'tuple',
        internalType: 'struct IbcCoreClientV1Height.Data',
        components: [
          {
            name: 'revision_number',
            type: 'uint64',
            internalType: 'uint64'
          },
          {
            name: 'revision_height',
            type: 'uint64',
            internalType: 'uint64'
          }
        ]
      },
      {
        name: 'timeoutTimestamp',
        type: 'uint64',
        internalType: 'uint64'
      },
      {
        name: 'data',
        type: 'bytes',
        internalType: 'bytes'
      }
    ],
    outputs: [
      {
        name: '',
        type: 'uint64',
        internalType: 'uint64'
      }
    ],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'timeoutPacket',
    inputs: [
      {
        name: 'msg_',
        type: 'tuple',
        internalType: 'struct IBCMsgs.MsgPacketTimeout',
        components: [
          {
            name: 'packet',
            type: 'tuple',
            internalType: 'struct IbcCoreChannelV1Packet.Data',
            components: [
              {
                name: 'sequence',
                type: 'uint64',
                internalType: 'uint64'
              },
              {
                name: 'source_port',
                type: 'string',
                internalType: 'string'
              },
              {
                name: 'source_channel',
                type: 'string',
                internalType: 'string'
              },
              {
                name: 'destination_port',
                type: 'string',
                internalType: 'string'
              },
              {
                name: 'destination_channel',
                type: 'string',
                internalType: 'string'
              },
              {
                name: 'data',
                type: 'bytes',
                internalType: 'bytes'
              },
              {
                name: 'timeout_height',
                type: 'tuple',
                internalType: 'struct IbcCoreClientV1Height.Data',
                components: [
                  {
                    name: 'revision_number',
                    type: 'uint64',
                    internalType: 'uint64'
                  },
                  {
                    name: 'revision_height',
                    type: 'uint64',
                    internalType: 'uint64'
                  }
                ]
              },
              {
                name: 'timeout_timestamp',
                type: 'uint64',
                internalType: 'uint64'
              }
            ]
          },
          {
            name: 'proof',
            type: 'bytes',
            internalType: 'bytes'
          },
          {
            name: 'proofHeight',
            type: 'tuple',
            internalType: 'struct IbcCoreClientV1Height.Data',
            components: [
              {
                name: 'revision_number',
                type: 'uint64',
                internalType: 'uint64'
              },
              {
                name: 'revision_height',
                type: 'uint64',
                internalType: 'uint64'
              }
            ]
          },
          {
            name: 'nextSequenceRecv',
            type: 'uint64',
            internalType: 'uint64'
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'updateClient',
    inputs: [
      {
        name: 'msg_',
        type: 'tuple',
        internalType: 'struct IBCMsgs.MsgUpdateClient',
        components: [
          {
            name: 'clientId',
            type: 'string',
            internalType: 'string'
          },
          {
            name: 'clientMessage',
            type: 'bytes',
            internalType: 'bytes'
          }
        ]
      }
    ],
    outputs: [],
    stateMutability: 'nonpayable'
  },
  {
    type: 'function',
    name: 'writeAcknowledgement',
    inputs: [
      {
        name: 'destinationPortId',
        type: 'string',
        internalType: 'string'
      },
      {
        name: 'destinationChannel',
        type: 'string',
        internalType: 'string'
      },
      {
        name: 'sequence',
        type: 'uint64',
        internalType: 'uint64'
      },
      {
        name: 'acknowledgement',
        type: 'bytes',
        internalType: 'bytes'
      }
    ],
    outputs: [],
    stateMutability: 'nonpayable'
  },
  {
    type: 'error',
    name: 'ErrCapabilityAlreadyClaimed',
    inputs: []
  },
  {
    type: 'error',
    name: 'ErrClientNotFound',
    inputs: []
  }
]
