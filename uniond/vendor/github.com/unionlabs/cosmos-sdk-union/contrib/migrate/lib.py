#!/usr/bin/env python3

import argparse
import json
import sys


def init_default_argument_parser(prog_desc, default_chain_id, default_start_time):
    parser = argparse.ArgumentParser(description=prog_desc)
    parser.add_argument(
        'exported_genesis',
        help='exported genesis.json file',
        type=argparse.FileType('r'), default=sys.stdin,
    )
    parser.add_argument('--chain-id', type=str, default=default_chain_id)
    parser.add_argument('--start-time', type=str, default=default_start_time)
    return parser


def main(argument_parser, process_genesis_func):
    args = argument_parser.parse_args()
    if args.chain_id.strip() == '':
        sys.exit('chain-id required')

    genesis = json.loads(args.exported_genesis.read())

    print(json.dumps(process_genesis_func(
        genesis=genesis, parsed_args=args,), indent=True))
