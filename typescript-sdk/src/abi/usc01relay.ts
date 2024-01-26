export const usc01relayAbi = [
  {
    inputs: [
      {
        internalType: 'contract IBCHandler',
        name: '_ibcHandler',
        type: 'address'
      }
    ],
    stateMutability: 'nonpayable',
    type: 'constructor'
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: false,
        internalType: 'string',
        name: 'denom',
        type: 'string'
      },
      {
        indexed: false,
        internalType: 'address',
        name: 'token',
        type: 'address'
      }
    ],
    name: 'DenomCreated',
    type: 'event'
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: false,
        internalType: 'string',
        name: 'sender',
        type: 'string'
      },
      {
        indexed: false,
        internalType: 'address',
        name: 'receiver',
        type: 'address'
      },
      {
        indexed: false,
        internalType: 'string',
        name: 'denom',
        type: 'string'
      },
      {
        indexed: false,
        internalType: 'address',
        name: 'token',
        type: 'address'
      },
      {
        indexed: false,
        internalType: 'uint256',
        name: 'amount',
        type: 'uint256'
      }
    ],
    name: 'Received',
    type: 'event'
  },
  {
    anonymous: false,
    inputs: [
      {
        indexed: false,
        internalType: 'address',
        name: 'sender',
        type: 'address'
      },
      {
        indexed: false,
        internalType: 'string',
        name: 'receiver',
        type: 'string'
      },
      {
        indexed: false,
        internalType: 'string',
        name: 'denom',
        type: 'string'
      },
      {
        indexed: false,
        internalType: 'address',
        name: 'token',
        type: 'address'
      },
      {
        indexed: false,
        internalType: 'uint256',
        name: 'amount',
        type: 'uint256'
      }
    ],
    name: 'Sent',
    type: 'event'
  },
  {
    inputs: [
      {
        internalType: 'address',
        name: '',
        type: 'address'
      }
    ],
    name: 'addressToDenom',
    outputs: [
      {
        internalType: 'string',
        name: '',
        type: 'string'
      }
    ],
    stateMutability: 'view',
    type: 'function'
  },
  {
    inputs: [
      {
        internalType: 'string',
        name: '',
        type: 'string'
      },
      {
        internalType: 'string',
        name: '',
        type: 'string'
      }
    ],
    name: 'counterpartyEndpoints',
    outputs: [
      {
        internalType: 'string',
        name: 'port_id',
        type: 'string'
      },
      {
        internalType: 'string',
        name: 'channel_id',
        type: 'string'
      }
    ],
    stateMutability: 'view',
    type: 'function'
  },
  {
    inputs: [
      {
        internalType: 'string',
        name: '',
        type: 'string'
      }
    ],
    name: 'denomToAddress',
    outputs: [
      {
        internalType: 'address',
        name: '',
        type: 'address'
      }
    ],
    stateMutability: 'view',
    type: 'function'
  },
  {
    inputs: [],
    name: 'ibcAddress',
    outputs: [
      {
        internalType: 'address',
        name: '',
        type: 'address'
      }
    ],
    stateMutability: 'view',
    type: 'function'
  },
  {
    inputs: [
      {
        components: [
          {
            internalType: 'uint64',
            name: 'sequence',
            type: 'uint64'
          },
          {
            internalType: 'string',
            name: 'source_port',
            type: 'string'
          },
          {
            internalType: 'string',
            name: 'source_channel',
            type: 'string'
          },
          {
            internalType: 'string',
            name: 'destination_port',
            type: 'string'
          },
          {
            internalType: 'string',
            name: 'destination_channel',
            type: 'string'
          },
          {
            internalType: 'bytes',
            name: 'data',
            type: 'bytes'
          },
          {
            components: [
              {
                internalType: 'uint64',
                name: 'revision_number',
                type: 'uint64'
              },
              {
                internalType: 'uint64',
                name: 'revision_height',
                type: 'uint64'
              }
            ],
            internalType: 'struct IbcCoreClientV1Height.Data',
            name: 'timeout_height',
            type: 'tuple'
          },
          {
            internalType: 'uint64',
            name: 'timeout_timestamp',
            type: 'uint64'
          }
        ],
        internalType: 'struct IbcCoreChannelV1Packet.Data',
        name: 'ibcPacket',
        type: 'tuple'
      },
      {
        internalType: 'bytes',
        name: 'acknowledgement',
        type: 'bytes'
      },
      {
        internalType: 'address',
        name: '_relayer',
        type: 'address'
      }
    ],
    name: 'onAcknowledgementPacket',
    outputs: [],
    stateMutability: 'nonpayable',
    type: 'function'
  },
  {
    inputs: [
      {
        internalType: 'string',
        name: '_portId',
        type: 'string'
      },
      {
        internalType: 'string',
        name: '_channelId',
        type: 'string'
      }
    ],
    name: 'onChanCloseConfirm',
    outputs: [],
    stateMutability: 'nonpayable',
    type: 'function'
  },
  {
    inputs: [
      {
        internalType: 'string',
        name: '_portId',
        type: 'string'
      },
      {
        internalType: 'string',
        name: '_channelId',
        type: 'string'
      }
    ],
    name: 'onChanCloseInit',
    outputs: [],
    stateMutability: 'nonpayable',
    type: 'function'
  },
  {
    inputs: [
      {
        internalType: 'string',
        name: 'portId',
        type: 'string'
      },
      {
        internalType: 'string',
        name: 'channelId',
        type: 'string'
      },
      {
        internalType: 'string',
        name: 'counterpartyChannelId',
        type: 'string'
      },
      {
        internalType: 'string',
        name: '_counterpartyVersion',
        type: 'string'
      }
    ],
    name: 'onChanOpenAck',
    outputs: [],
    stateMutability: 'nonpayable',
    type: 'function'
  },
  {
    inputs: [
      {
        internalType: 'string',
        name: '_portId',
        type: 'string'
      },
      {
        internalType: 'string',
        name: '_channelId',
        type: 'string'
      }
    ],
    name: 'onChanOpenConfirm',
    outputs: [],
    stateMutability: 'nonpayable',
    type: 'function'
  },
  {
    inputs: [
      {
        internalType: 'enum IbcCoreChannelV1GlobalEnums.Order',
        name: '_order',
        type: 'uint8'
      },
      {
        internalType: 'string[]',
        name: '_connectionHops',
        type: 'string[]'
      },
      {
        internalType: 'string',
        name: 'portId',
        type: 'string'
      },
      {
        internalType: 'string',
        name: 'channelId',
        type: 'string'
      },
      {
        components: [
          {
            internalType: 'string',
            name: 'port_id',
            type: 'string'
          },
          {
            internalType: 'string',
            name: 'channel_id',
            type: 'string'
          }
        ],
        internalType: 'struct IbcCoreChannelV1Counterparty.Data',
        name: 'counterpartyEndpoint',
        type: 'tuple'
      },
      {
        internalType: 'string',
        name: '_version',
        type: 'string'
      }
    ],
    name: 'onChanOpenInit',
    outputs: [],
    stateMutability: 'nonpayable',
    type: 'function'
  },
  {
    inputs: [
      {
        internalType: 'enum IbcCoreChannelV1GlobalEnums.Order',
        name: '_order',
        type: 'uint8'
      },
      {
        internalType: 'string[]',
        name: '_connectionHops',
        type: 'string[]'
      },
      {
        internalType: 'string',
        name: 'portId',
        type: 'string'
      },
      {
        internalType: 'string',
        name: 'channelId',
        type: 'string'
      },
      {
        components: [
          {
            internalType: 'string',
            name: 'port_id',
            type: 'string'
          },
          {
            internalType: 'string',
            name: 'channel_id',
            type: 'string'
          }
        ],
        internalType: 'struct IbcCoreChannelV1Counterparty.Data',
        name: 'counterpartyEndpoint',
        type: 'tuple'
      },
      {
        internalType: 'string',
        name: '_version',
        type: 'string'
      },
      {
        internalType: 'string',
        name: '_counterpartyVersion',
        type: 'string'
      }
    ],
    name: 'onChanOpenTry',
    outputs: [],
    stateMutability: 'nonpayable',
    type: 'function'
  },
  {
    inputs: [
      {
        components: [
          {
            internalType: 'uint64',
            name: 'sequence',
            type: 'uint64'
          },
          {
            internalType: 'string',
            name: 'source_port',
            type: 'string'
          },
          {
            internalType: 'string',
            name: 'source_channel',
            type: 'string'
          },
          {
            internalType: 'string',
            name: 'destination_port',
            type: 'string'
          },
          {
            internalType: 'string',
            name: 'destination_channel',
            type: 'string'
          },
          {
            internalType: 'bytes',
            name: 'data',
            type: 'bytes'
          },
          {
            components: [
              {
                internalType: 'uint64',
                name: 'revision_number',
                type: 'uint64'
              },
              {
                internalType: 'uint64',
                name: 'revision_height',
                type: 'uint64'
              }
            ],
            internalType: 'struct IbcCoreClientV1Height.Data',
            name: 'timeout_height',
            type: 'tuple'
          },
          {
            internalType: 'uint64',
            name: 'timeout_timestamp',
            type: 'uint64'
          }
        ],
        internalType: 'struct IbcCoreChannelV1Packet.Data',
        name: 'ibcPacket',
        type: 'tuple'
      },
      {
        internalType: 'address',
        name: 'relayer',
        type: 'address'
      }
    ],
    name: 'onRecvPacket',
    outputs: [
      {
        internalType: 'bytes',
        name: 'acknowledgement',
        type: 'bytes'
      }
    ],
    stateMutability: 'nonpayable',
    type: 'function'
  },
  {
    inputs: [
      {
        components: [
          {
            internalType: 'uint64',
            name: 'sequence',
            type: 'uint64'
          },
          {
            internalType: 'string',
            name: 'source_port',
            type: 'string'
          },
          {
            internalType: 'string',
            name: 'source_channel',
            type: 'string'
          },
          {
            internalType: 'string',
            name: 'destination_port',
            type: 'string'
          },
          {
            internalType: 'string',
            name: 'destination_channel',
            type: 'string'
          },
          {
            internalType: 'bytes',
            name: 'data',
            type: 'bytes'
          },
          {
            components: [
              {
                internalType: 'uint64',
                name: 'revision_number',
                type: 'uint64'
              },
              {
                internalType: 'uint64',
                name: 'revision_height',
                type: 'uint64'
              }
            ],
            internalType: 'struct IbcCoreClientV1Height.Data',
            name: 'timeout_height',
            type: 'tuple'
          },
          {
            internalType: 'uint64',
            name: 'timeout_timestamp',
            type: 'uint64'
          }
        ],
        internalType: 'struct IbcCoreChannelV1Packet.Data',
        name: 'ibcPacket',
        type: 'tuple'
      },
      {
        internalType: 'address',
        name: 'relayer',
        type: 'address'
      }
    ],
    name: 'onRecvPacketProcessing',
    outputs: [],
    stateMutability: 'nonpayable',
    type: 'function'
  },
  {
    inputs: [
      {
        components: [
          {
            internalType: 'uint64',
            name: 'sequence',
            type: 'uint64'
          },
          {
            internalType: 'string',
            name: 'source_port',
            type: 'string'
          },
          {
            internalType: 'string',
            name: 'source_channel',
            type: 'string'
          },
          {
            internalType: 'string',
            name: 'destination_port',
            type: 'string'
          },
          {
            internalType: 'string',
            name: 'destination_channel',
            type: 'string'
          },
          {
            internalType: 'bytes',
            name: 'data',
            type: 'bytes'
          },
          {
            components: [
              {
                internalType: 'uint64',
                name: 'revision_number',
                type: 'uint64'
              },
              {
                internalType: 'uint64',
                name: 'revision_height',
                type: 'uint64'
              }
            ],
            internalType: 'struct IbcCoreClientV1Height.Data',
            name: 'timeout_height',
            type: 'tuple'
          },
          {
            internalType: 'uint64',
            name: 'timeout_timestamp',
            type: 'uint64'
          }
        ],
        internalType: 'struct IbcCoreChannelV1Packet.Data',
        name: 'ibcPacket',
        type: 'tuple'
      },
      {
        internalType: 'address',
        name: '_relayer',
        type: 'address'
      }
    ],
    name: 'onTimeoutPacket',
    outputs: [],
    stateMutability: 'nonpayable',
    type: 'function'
  },
  {
    inputs: [
      {
        internalType: 'string',
        name: '',
        type: 'string'
      },
      {
        internalType: 'string',
        name: '',
        type: 'string'
      },
      {
        internalType: 'address',
        name: '',
        type: 'address'
      }
    ],
    name: 'outstanding',
    outputs: [
      {
        internalType: 'uint256',
        name: '',
        type: 'uint256'
      }
    ],
    stateMutability: 'view',
    type: 'function'
  },
  {
    inputs: [
      {
        internalType: 'string',
        name: 'portId',
        type: 'string'
      },
      {
        internalType: 'string',
        name: 'channelId',
        type: 'string'
      },
      {
        internalType: 'bytes',
        name: 'receiver',
        type: 'bytes'
      },
      {
        components: [
          {
            internalType: 'address',
            name: 'denom',
            type: 'address'
          },
          {
            internalType: 'uint128',
            name: 'amount',
            type: 'uint128'
          }
        ],
        internalType: 'struct LocalToken[]',
        name: 'tokens',
        type: 'tuple[]'
      },
      {
        internalType: 'uint64',
        name: 'counterpartyTimeoutRevisionNumber',
        type: 'uint64'
      },
      {
        internalType: 'uint64',
        name: 'counterpartyTimeoutRevisionHeight',
        type: 'uint64'
      }
    ],
    name: 'send',
    outputs: [],
    stateMutability: 'nonpayable',
    type: 'function'
  }
] as const
