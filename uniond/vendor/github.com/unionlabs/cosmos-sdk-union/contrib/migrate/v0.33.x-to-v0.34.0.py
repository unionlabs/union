#!/usr/bin/env python3

import lib


def process_raw_genesis(genesis, parsed_args):
    # update genesis with breaking changes
    if 'block_size' in genesis['consensus_params']:
        genesis['consensus_params']['block'] = genesis['consensus_params']['block_size']
        del genesis['consensus_params']['block_size']

    genesis['app_state']['crisis'] = {
        'constant_fee': {
            'amount': '1333000000',  # ~$5,000 worth of uatoms
            'denom': 'uatom',
        },
    }

    # migrate governance state as the internal structure of proposals has changed
    migrate_gov_data(genesis['app_state']['gov'])

    # default Tendermint block time (ms)
    genesis['consensus_params']['block']['time_iota_ms'] = '1000'

    # proposal #1 updates
    genesis['app_state']['mint']['params']['blocks_per_year'] = '4855015'

    # proposal #2 updates
    genesis['consensus_params']['block']['max_gas'] = '2000000'
    genesis['consensus_params']['block']['max_bytes'] = '200000'

    # enable transfers
    genesis['app_state']['bank']['send_enabled'] = True
    genesis['app_state']['distr']['withdraw_addr_enabled'] = True

    # Set new chain ID and genesis start time
    genesis['chain_id'] = parsed_args.chain_id.strip()
    genesis['genesis_time'] = parsed_args.start_time

    return genesis


def migrate_gov_data(gov_data):
    for p in gov_data['proposals']:
        # get Amino type and value
        t = p['type']
        v = p['value']

        del p['type']
        del p['value']

        assert t == 'gov/TextProposal', 'invalid proposal type: {t}'
        assert p == {}, 'expected proposal to be empty after deleting contents'

        p['proposal_content'] = {
            'type': t,
            'value': {
                'title': v['title'],
                'description': v['description']
            }
        }

        p['proposal_id'] = v['proposal_id']
        p['proposal_status'] = v['proposal_status']
        p['final_tally_result'] = v['final_tally_result']
        p['submit_time'] = v['submit_time']
        p['deposit_end_time'] = v['deposit_end_time']
        p['total_deposit'] = v['total_deposit']
        p['voting_start_time'] = v['voting_start_time']
        p['voting_end_time'] = v['voting_end_time']


if __name__ == '__main__':
    parser = lib.init_default_argument_parser(
        prog_desc='Convert genesis.json from v0.33.x to v0.34.0',
        default_chain_id='cosmoshub-n',
        default_start_time='2019-02-11T12:00:00Z',
    )
    lib.main(parser, process_raw_genesis)
