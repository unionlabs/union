use convert_case::{Case, Casing};
use std::collections::HashMap;
use std::fmt::Write;
use std::fs;
use std::path::{Component, Path, PathBuf};
use walkdir::{DirEntry, Error, WalkDir};

type TestIndex = HashMap<
    String,
    HashMap<String, HashMap<String, HashMap<String, HashMap<String, HashMap<String, String>>>>>,
>;

fn component_to_string(c: Component) -> String {
    let c = c.as_os_str().to_str().unwrap().to_string();
    c.to_case(Case::Snake)
}

fn insert_test(tests: &mut TestIndex, path: &Path) -> bool {
    let mut components = path.components().skip(2);

    let config = component_to_string(components.next().unwrap());
    let tests = tests.entry(config).or_default();

    let fork = component_to_string(components.next().unwrap());
    let tests = tests.entry(fork).or_default();

    let runner = component_to_string(components.next().unwrap());

    // NOTE: these tests are handled elsewhere outside this repo.
    if runner == "ssz_generic" {
        return false;
    }

    let tests = tests.entry(runner).or_default();

    let handler = component_to_string(components.next().unwrap());
    let tests = tests.entry(handler).or_default();

    let suite = component_to_string(components.next().unwrap());
    let tests = tests.entry(suite).or_default();

    let test_case = component_to_string(components.next().unwrap());
    tests.insert(test_case, path.display().to_string());

    true
}

fn collect_test_cases(root: &str) -> Result<(TestIndex, usize), Error> {
    let mut total_test_case_count = 0;
    let mut tests = TestIndex::new();
    for entry in leaf_dirs(&PathBuf::from(root)) {
        let inserted = insert_test(&mut tests, entry.path());
        if inserted {
            total_test_case_count += 1;
        }
    }
    Ok((tests, total_test_case_count))
}

fn prefix_if_starts_with_reserved(s: &str) -> String {
    if s.as_bytes()[0].is_ascii_digit() || s == "true" || s == "false" {
        String::from("mod_") + s
    } else {
        s.to_string()
    }
}

fn generate_module_decl_src<'a>(modules: impl Iterator<Item = &'a String>) -> String {
    let mut src = String::new();
    for module in modules {
        let module = prefix_if_starts_with_reserved(module);
        let module = if module == "phase_0" {
            "phase0".to_string()
        } else {
            module
        };
        writeln!(src, "mod {module};").unwrap();
    }
    src
}

fn is_leaf(entry: &DirEntry) -> bool {
    let mut has_children = false;
    let mut is_empty = true;

    for child in WalkDir::new(entry.path()).min_depth(1).max_depth(2) {
        is_empty = false;
        let child = child.unwrap();
        if child.metadata().unwrap().is_dir() {
            has_children = true;
        }
    }

    !has_children && !is_empty
}

fn leaf_dirs(path: &PathBuf) -> impl Iterator<Item = DirEntry> {
    WalkDir::new(path)
        .into_iter()
        .filter(|e| is_leaf(e.as_ref().unwrap()))
        .map(|e| e.unwrap())
}

fn generate_suite_src(
    tests: &HashMap<String, String>,
    fork: &str,
    config: &str,
    runner: &str,
    handler: &str,
    auxilliary_data: &HashMap<&str, HashMap<&str, Auxillary>>,
) -> String {
    let mut test_case_type = handler.to_case(Case::UpperCamel) + "TestCase";
    let mut src = r#"// WARNING!
// This file was generated with `gen-tests`. Do NOT edit manually.

"#
    .to_string();

    writeln!(
        src,
        "use crate::spec_test_runners::{runner}::{test_case_type};",
    )
    .unwrap();

    let needs_trait_import = matches!(runner, "bls");

    if needs_trait_import {
        src += "use crate::test_utils::TestCase;\n";
    }

    let mut needs_spec_import = false;
    let mut execution_handler = String::new();
    let mut mut_decl = String::new();
    let mut preamble = String::new();
    let spec = match fork {
        "phase0" => Spec::Phase0,
        "altair" => Spec::Altair,
        "bellatrix" => Spec::Bellatrix,
        _ => todo!("support other forks"),
    };
    if let Some(handler_data) = auxilliary_data.get(runner) {
        if let Some(data) = handler_data.get(handler) {
            let mut generics = data.test_case_type_generics.to_string();
            // NOTE: special case for this handler...
            if runner == "genesis" && handler == "initialization" {
                if spec == Spec::Bellatrix {
                    generics += ", spec::ExecutionPayloadHeader"
                } else {
                    // "placeholder" type, can be whatever
                    // ideally would  use `()` but it does not implement `ssz_rs::Deserialize`
                    generics += ", usize"
                }
            }
            test_case_type = format!("{test_case_type}::<{generics}>");
            if data.execution_handler.len() == 1 && data.execution_handler.contains_key(&Spec::All)
            {
                execution_handler += &data.execution_handler[&Spec::All];
            } else {
                assert!(data.execution_handler.len() > 1);
                let handler = &data.execution_handler[&spec];
                execution_handler += handler;
            }
            if !matches!(runner, "ssz_static" | "rewards" | "fork") {
                mut_decl += "mut";
            }
            if let Some(s) = data.preamble.get(&spec) {
                preamble += s;
            }
            if let Some(s) = data.preamble.get(&Spec::All) {
                preamble += s;
            }
            needs_spec_import = true;
        }
    }

    if execution_handler.is_empty() {
        execution_handler += "execute()";
    }

    if needs_spec_import {
        writeln!(src, "use ethereum_consensus::{fork}::{config} as spec;").unwrap();
        writeln!(src, "use ssz_rs::prelude::*;").unwrap();
    }

    // special case imports here...
    if matches!(runner, "transition") {
        writeln!(
            src,
            "use ethereum_consensus::state_transition::{config}::{{Executor, BeaconState}};",
        )
        .unwrap();
        let pre_fork = match spec {
            Spec::Altair => "phase0",
            Spec::Bellatrix => "altair",
            _ => unimplemented!("support other forks"),
        };
        writeln!(
            src,
            "use ethereum_consensus::{pre_fork}::{config} as pre_spec;",
        )
        .unwrap();
        writeln!(
            src,
            "use ethereum_consensus::bellatrix::{config}::NoOpExecutionEngine;",
        )
        .unwrap();
    }
    if matches!(runner, "fork") {
        match spec {
            Spec::Altair => {
                writeln!(src, "use ethereum_consensus::phase0::{config} as phase0;").unwrap();
            }
            Spec::Bellatrix => {
                writeln!(src, "use ethereum_consensus::altair::{config} as altair;").unwrap();
            }
            _ => todo!("support other forks"),
        }
    }
    let mut test_cases = tests.keys().cloned().collect::<Vec<_>>();
    test_cases.sort();
    for test_case in &test_cases {
        let source_path = &tests[test_case];
        let test_src = format!(
            r#"
#[test]
fn test_{test_case}() {{
    let {mut_decl} test_case = {test_case_type}::from("{source_path}");
    {preamble}
    test_case.{execution_handler};
}}
"#
        );
        src += &test_src;
    }

    src
}

struct Auxillary {
    test_case_type_generics: String,
    preamble: HashMap<Spec, String>,
    execution_handler: HashMap<Spec, String>,
}

#[derive(Hash, PartialEq, Eq)]
enum Spec {
    Phase0,
    Altair,
    Bellatrix,
    All,
}

fn main() {
    let auxilliary_data = HashMap::from([
        ("operations",
        HashMap::from([
            (
                "attestation",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState, spec::Attestation".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|state, operation, context| {
                    spec::process_attestation(state, operation, context)
                })"
                    .to_string())]),
                },
            ),
            (
                "attester_slashing",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState, spec::AttesterSlashing".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|state, operation, context| {
                    spec::process_attester_slashing(state, operation, context)
                })"
                    .to_string())]),
                },
            ),
            (
                "block_header",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState, spec::BeaconBlock".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|state, operation, context| {
                    spec::process_block_header(state, operation, context)
                })"
                    .to_string())]),
                },
            ),
            (
                "deposit",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState, spec::Deposit".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|state, operation, context| {
                    spec::process_deposit(state, operation, context)
                })"
                    .to_string())]),
                },
            ),
            (
                "proposer_slashing",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState, spec::ProposerSlashing".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|state, operation, context| {
                    spec::process_proposer_slashing(state, operation, context)
                })"
                    .to_string())]),
                },
            ),
            (
                "voluntary_exit",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState, spec::SignedVoluntaryExit".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|state, operation, context| {
                    spec::process_voluntary_exit(state, operation, context)
                })"
                    .to_string())]),
                },
            ),
            (
                "sync_aggregate",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState, spec::SyncAggregate".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|state, operation, context| {
                    spec::process_sync_aggregate(state, operation, context)
                })"
                    .to_string())]),
                },
            ),
            (
                "execution_payload",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState, spec::ExecutionPayload".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|state, operation, context, execution_valid| {
                    use ethereum_consensus::state_transition::{Error, InvalidBlock, InvalidOperation, InvalidExecutionPayload};
                    let execution_engine = spec::MockExecutionEngine::new(|_| if execution_valid {
                        Ok(())
                    } else {
                        // NOTE: exact error is not specified for this test
                        Err(Error::InvalidBlock(Box::new(InvalidBlock::InvalidOperation(InvalidOperation::ExecutionPayload(InvalidExecutionPayload::InvalidTimestamp { provided: 0, expected: 0 })))))
                    });
                    spec::process_execution_payload(state, operation, execution_engine, context)
                })"
                    .to_string())]),
                },
            ),
        ])),
        ("genesis",
        HashMap::from([
            (
                "validity",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|state, context| {
                    spec::is_valid_genesis_state(state, context)
                })"
                    .to_string())]),
                },
            ),
            (
                "initialization",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState, spec::Deposit".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([
                        (Spec::Phase0, "execute(|eth1_block_hash, eth1_timestamp, deposits, _execution_payload_header, context| {
                    spec::initialize_beacon_state_from_eth1::<
                        {spec::SLOTS_PER_HISTORICAL_ROOT},
                        {spec::HISTORICAL_ROOTS_LIMIT},
                        {spec::ETH1_DATA_VOTES_BOUND},
                        {spec::VALIDATOR_REGISTRY_LIMIT},
                        {spec::EPOCHS_PER_HISTORICAL_VECTOR},
                        {spec::EPOCHS_PER_SLASHINGS_VECTOR},
                        {spec::MAX_VALIDATORS_PER_COMMITTEE},
                        {spec::PENDING_ATTESTATIONS_BOUND},
                        {spec::MAX_PROPOSER_SLASHINGS},
                        {spec::MAX_ATTESTER_SLASHINGS},
                        {spec::MAX_ATTESTATIONS},
                        {spec::MAX_DEPOSITS},
                        {spec::MAX_VOLUNTARY_EXITS},
                    >(eth1_block_hash, eth1_timestamp, deposits, context)
                })"
                    .to_string()),
                        (Spec::Altair, "execute(|eth1_block_hash, eth1_timestamp, deposits, _execution_payload_header, context| {
                    spec::initialize_beacon_state_from_eth1::<
                        {spec::SLOTS_PER_HISTORICAL_ROOT},
                        {spec::HISTORICAL_ROOTS_LIMIT},
                        {spec::ETH1_DATA_VOTES_BOUND},
                        {spec::VALIDATOR_REGISTRY_LIMIT},
                        {spec::EPOCHS_PER_HISTORICAL_VECTOR},
                        {spec::EPOCHS_PER_SLASHINGS_VECTOR},
                        {spec::MAX_VALIDATORS_PER_COMMITTEE},
                        {spec::SYNC_COMMITTEE_SIZE},
                        {spec::MAX_PROPOSER_SLASHINGS},
                        {spec::MAX_ATTESTER_SLASHINGS},
                        {spec::MAX_ATTESTATIONS},
                        {spec::MAX_DEPOSITS},
                        {spec::MAX_VOLUNTARY_EXITS},
                        >(eth1_block_hash, eth1_timestamp, deposits, context)
                })"
                    .to_string()),
                        (Spec::Bellatrix, "execute(|eth1_block_hash, eth1_timestamp, deposits, execution_payload_header, context| {
                    spec::initialize_beacon_state_from_eth1::<
                        {spec::SLOTS_PER_HISTORICAL_ROOT},
                        {spec::HISTORICAL_ROOTS_LIMIT},
                        {spec::ETH1_DATA_VOTES_BOUND},
                        {spec::VALIDATOR_REGISTRY_LIMIT},
                        {spec::EPOCHS_PER_HISTORICAL_VECTOR},
                        {spec::EPOCHS_PER_SLASHINGS_VECTOR},
                        {spec::MAX_VALIDATORS_PER_COMMITTEE},
                        {spec::SYNC_COMMITTEE_SIZE},
                        {spec::MAX_PROPOSER_SLASHINGS},
                        {spec::MAX_ATTESTER_SLASHINGS},
                        {spec::MAX_ATTESTATIONS},
                        {spec::MAX_DEPOSITS},
                        {spec::MAX_VOLUNTARY_EXITS},
                        {spec::BYTES_PER_LOGS_BLOOM},
                        {spec::MAX_EXTRA_DATA_BYTES},
                        {spec::MAX_BYTES_PER_TRANSACTION},
                        {spec::MAX_TRANSACTIONS_PER_PAYLOAD},
                        >(eth1_block_hash, eth1_timestamp, deposits, execution_payload_header, context)
                })"
                    .to_string())
                    ]),
                },
            ),
        ])),
        ("transition",
        HashMap::from([
            (
                "core",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([
                        (Spec::Altair, "execute(|state: pre_spec::BeaconState, pre_blocks: Vec<pre_spec::SignedBeaconBlock>, blocks: Vec<spec::SignedBeaconBlock>, context| {
                    let mut executor = Executor::new(state.into(), NoOpExecutionEngine, context);
                    for block in pre_blocks.into_iter() {
                        let mut block = block.into();
                        executor.apply_block(&mut block)?;
                    }
                    for block in blocks.into_iter() {
                        let mut block = block.into();
                        executor.apply_block(&mut block)?;
                    }
                    match executor.state {
                        BeaconState::Altair(inner) => Ok(*inner),
                        _ => unreachable!(),
                    }
                })"
                    .to_string()),
                        (Spec::Bellatrix, "execute(|state: pre_spec::BeaconState, pre_blocks: Vec<pre_spec::SignedBeaconBlock>, blocks: Vec<spec::SignedBeaconBlock>, context| {
                    let mut executor = Executor::new(state.into(), NoOpExecutionEngine, context);
                    for block in pre_blocks.into_iter() {
                        let mut block = block.into();
                        executor.apply_block(&mut block)?;
                    }
                    for block in blocks.into_iter() {
                        let mut block = block.into();
                        executor.apply_block(&mut block)?;
                    }
                    match executor.state {
                        BeaconState::Bellatrix(inner) => Ok(*inner),
                        _ => unreachable!(),
                    }
                })"
                    .to_string())]),
                },
            ),
        ])),
        ("fork",
        HashMap::from([
            (
                "fork",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([
                        (Spec::Altair, "execute(|state: &phase0::BeaconState, context| -> spec::BeaconState {
                    spec::upgrade_to_altair(state, context).unwrap()
                })"
                    .to_string()),
                        (Spec::Bellatrix, "execute(|state: &altair::BeaconState, context| -> spec::BeaconState {
                    spec::upgrade_to_bellatrix(state, context)
                })"
                    .to_string())
                    ]),
                },
            ),
        ])),
        ("sanity",
        HashMap::from([
            (
                "slots",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|state, offset, context| {
                    let target_slot = state.slot + offset;
                    spec::process_slots(state, target_slot, context).unwrap();
                })"
                    .to_string())]),
                },
            ),
            (
                "blocks",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState, spec::SignedBeaconBlock"
                        .to_string(),
                    preamble: HashMap::from_iter([(Spec::Bellatrix, "let execution_engine = spec::NoOpExecutionEngine;".to_string())]),
                    execution_handler: HashMap::from_iter([(Spec::Phase0, "execute(|state, blocks, validation, context| {
                        for block in blocks.iter_mut() {
                            spec::state_transition(state, block, validation, context)?;
                        }
                        Ok(())
                    })".to_string()), (Spec::Altair, "execute(|state, blocks, validation, context| {
                        for block in blocks.iter_mut() {
                            spec::state_transition(state, block, validation, context)?;
                        }
                        Ok(())
                    })".to_string()), (Spec::Bellatrix, "execute(|state, blocks, validation, context| {
                        for block in blocks.iter_mut() {
                            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
                        }
                        Ok(())
                    })".to_string())]),
                },
            ),
        ])),
        ("rewards",
        HashMap::from([
            (
                "basic",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([
                        (Spec::Phase0, "execute(|state, context| {
                    let source_deltas = spec::get_source_deltas(state, context).unwrap();
                    let target_deltas = spec::get_target_deltas(state, context).unwrap();
                    let head_deltas = spec::get_head_deltas(state, context).unwrap();
                    let inclusion_penalty_deltas = spec::get_inclusion_delay_deltas(state, context).unwrap();
                    let inactivity_penalty_deltas = spec::get_inactivity_penalty_deltas(state, context).unwrap();
                    (source_deltas, target_deltas, head_deltas, Some(inclusion_penalty_deltas), inactivity_penalty_deltas)
                })"
                    .to_string()),
                        (Spec::Altair, "execute(|state, context| {
                    let flag_index = spec::TIMELY_SOURCE_FLAG_INDEX;
                    let source_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
                    let flag_index = spec::TIMELY_TARGET_FLAG_INDEX;
                    let target_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
                    let flag_index = spec::TIMELY_HEAD_FLAG_INDEX;
                    let head_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
                    let inactivity_penalty_deltas = spec::get_inactivity_penalty_deltas(state, context).unwrap();
                    (source_deltas, target_deltas, head_deltas, None, inactivity_penalty_deltas)
                })"
                    .to_string()),
                        (Spec::Bellatrix, "execute(|state, context| {
                    let flag_index = spec::TIMELY_SOURCE_FLAG_INDEX;
                    let source_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
                    let flag_index = spec::TIMELY_TARGET_FLAG_INDEX;
                    let target_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
                    let flag_index = spec::TIMELY_HEAD_FLAG_INDEX;
                    let head_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
                    let inactivity_penalty_deltas = spec::get_inactivity_penalty_deltas(state, context).unwrap();
                    (source_deltas, target_deltas, head_deltas, None, inactivity_penalty_deltas)
                })"
                    .to_string()),
                    ]),
                },
            ),
            (
                "leak",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([
                        (Spec::Phase0, "execute(|state, context| {
                    let source_deltas = spec::get_source_deltas(state, context).unwrap();
                    let target_deltas = spec::get_target_deltas(state, context).unwrap();
                    let head_deltas = spec::get_head_deltas(state, context).unwrap();
                    let inclusion_penalty_deltas = spec::get_inclusion_delay_deltas(state, context).unwrap();
                    let inactivity_penalty_deltas = spec::get_inactivity_penalty_deltas(state, context).unwrap();
                    (source_deltas, target_deltas, head_deltas, Some(inclusion_penalty_deltas), inactivity_penalty_deltas)
                })"
                    .to_string()),
                        (Spec::Altair, "execute(|state, context| {
                    let flag_index = spec::TIMELY_SOURCE_FLAG_INDEX;
                    let source_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
                    let flag_index = spec::TIMELY_TARGET_FLAG_INDEX;
                    let target_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
                    let flag_index = spec::TIMELY_HEAD_FLAG_INDEX;
                    let head_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
                    let inactivity_penalty_deltas = spec::get_inactivity_penalty_deltas(state, context).unwrap();
                    (source_deltas, target_deltas, head_deltas, None, inactivity_penalty_deltas)
                })"
                    .to_string()),
                        (Spec::Bellatrix, "execute(|state, context| {
                    let flag_index = spec::TIMELY_SOURCE_FLAG_INDEX;
                    let source_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
                    let flag_index = spec::TIMELY_TARGET_FLAG_INDEX;
                    let target_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
                    let flag_index = spec::TIMELY_HEAD_FLAG_INDEX;
                    let head_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
                    let inactivity_penalty_deltas = spec::get_inactivity_penalty_deltas(state, context).unwrap();
                    (source_deltas, target_deltas, head_deltas, None, inactivity_penalty_deltas)
                })"
                    .to_string()),
                    ]),
                },
            ),
            (
                "random",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([
                        (Spec::Phase0, "execute(|state, context| {
                    let source_deltas = spec::get_source_deltas(state, context).unwrap();
                    let target_deltas = spec::get_target_deltas(state, context).unwrap();
                    let head_deltas = spec::get_head_deltas(state, context).unwrap();
                    let inclusion_penalty_deltas = spec::get_inclusion_delay_deltas(state, context).unwrap();
                    let inactivity_penalty_deltas = spec::get_inactivity_penalty_deltas(state, context).unwrap();
                    (source_deltas, target_deltas, head_deltas, Some(inclusion_penalty_deltas), inactivity_penalty_deltas)
                })"
                    .to_string()),
                        (Spec::Altair, "execute(|state, context| {
                    let flag_index = spec::TIMELY_SOURCE_FLAG_INDEX;
                    let source_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
                    let flag_index = spec::TIMELY_TARGET_FLAG_INDEX;
                    let target_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
                    let flag_index = spec::TIMELY_HEAD_FLAG_INDEX;
                    let head_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
                    let inactivity_penalty_deltas = spec::get_inactivity_penalty_deltas(state, context).unwrap();
                    (source_deltas, target_deltas, head_deltas, None, inactivity_penalty_deltas)
                })"
                    .to_string()),
                        (Spec::Bellatrix, "execute(|state, context| {
                    let flag_index = spec::TIMELY_SOURCE_FLAG_INDEX;
                    let source_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
                    let flag_index = spec::TIMELY_TARGET_FLAG_INDEX;
                    let target_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
                    let flag_index = spec::TIMELY_HEAD_FLAG_INDEX;
                    let head_deltas = spec::get_flag_index_deltas(state, flag_index, context).unwrap();
                    let inactivity_penalty_deltas = spec::get_inactivity_penalty_deltas(state, context).unwrap();
                    (source_deltas, target_deltas, head_deltas, None, inactivity_penalty_deltas)
                })"
                    .to_string()),
                    ]),
                },
            ),
        ])),
        ("finality",
        HashMap::from([
            (
                "finality",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState, spec::SignedBeaconBlock"
                        .to_string(),
                    preamble: HashMap::from_iter([(Spec::Bellatrix, "let execution_engine = spec::NoOpExecutionEngine;".to_string())]),
                    execution_handler: HashMap::from_iter([(Spec::Phase0, "execute(|state, blocks, validation, context| {
                        for block in blocks.iter_mut() {
                            spec::state_transition(state, block, validation, context)?;
                        }
                        Ok(())
                    })".to_string()), (Spec::Altair, "execute(|state, blocks, validation, context| {
                        for block in blocks.iter_mut() {
                            spec::state_transition(state, block, validation, context)?;
                        }
                        Ok(())
                    })".to_string()), (Spec::Bellatrix, "execute(|state, blocks, validation, context| {
                        for block in blocks.iter_mut() {
                            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
                        }
                        Ok(())
                    })".to_string())]),
                },
            ),
        ])),
        ("ssz_static",
        HashMap::from([
            (
                "aggregate_and_proof",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::AggregateAndProof = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "attestation",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::Attestation = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "attestation_data",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::AttestationData = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "attester_slashing",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::AttesterSlashing = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "beacon_block",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::BeaconBlock = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "beacon_block_body",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::BeaconBlockBody = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "beacon_block_header",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::BeaconBlockHeader = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "beacon_state",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::BeaconState = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "checkpoint",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::Checkpoint = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "contribution_and_proof",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::ContributionAndProof = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "deposit",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::Deposit = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "deposit_data",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::DepositData = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "deposit_message",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::DepositMessage = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "eth_1_block",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::Eth1Block = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "eth_1_data",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::Eth1Data = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "execution_payload",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::ExecutionPayload = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "execution_payload_header",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::ExecutionPayloadHeader = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "fork",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::Fork = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "fork_data",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::ForkData = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "historical_batch",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::HistoricalBatch = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "indexed_attestation",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::IndexedAttestation = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "light_client_update",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::LightClientUpdate = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "pending_attestation",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::PendingAttestation = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "pow_block",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::PowBlock = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "proposer_slashing",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::ProposerSlashing = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "signed_aggregate_and_proof",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::SignedAggregateAndProof = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "signed_beacon_block",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::SignedBeaconBlock = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "signed_beacon_block_header",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::SignedBeaconBlockHeader = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "signed_contribution_and_proof",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::SignedContributionAndProof = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "signed_voluntary_exit",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::SignedVoluntaryExit = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "signing_data",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::SigningData = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "sync_aggregate",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::SyncAggregate = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "sync_aggregator_selection_data",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::SyncAggregatorSelectionData = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "sync_committee",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::SyncCommittee = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "sync_committee_contribution",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::SyncCommitteeContribution = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "sync_committee_message",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::SyncCommitteeMessage = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "validator",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::Validator = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
            (
                "voluntary_exit",
                Auxillary {
                    test_case_type_generics: Default::default(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|encoding| {
                        let mut data: spec::VoluntaryExit = ssz_rs::deserialize(encoding).unwrap();
                        let serialized = ssz_rs::serialize(&data).unwrap();
                        let root = data.hash_tree_root().unwrap();
                        (serialized, root)
                })"
                    .to_string())]),
                },
            ),
        ])),
        ("random",
        HashMap::from([
            (
                "random",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState, spec::SignedBeaconBlock"
                        .to_string(),
                    preamble: HashMap::from_iter([(Spec::Bellatrix, "let execution_engine = spec::NoOpExecutionEngine;".to_string())]),
                    execution_handler: HashMap::from_iter([(Spec::Phase0, "execute(|state, blocks, validation, context| {
                        for block in blocks.iter_mut() {
                            spec::state_transition(state, block, validation, context)?;
                        }
                        Ok(())
                    })".to_string()), (Spec::Altair, "execute(|state, blocks, validation, context| {
                        for block in blocks.iter_mut() {
                            spec::state_transition(state, block, validation, context)?;
                        }
                        Ok(())
                    })".to_string()), (Spec::Bellatrix, "execute(|state, blocks, validation, context| {
                        for block in blocks.iter_mut() {
                            spec::state_transition(state, block, execution_engine.clone(), validation, context)?;
                        }
                        Ok(())
                    })".to_string())]),
                },
            ),
        ])),
        ("epoch_processing",
        HashMap::from([
            (
                "effective_balance_updates",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|state, context| {
                        spec::process_effective_balance_updates(state, context);
                        Ok(())
                })"
                    .to_string())]),
                },
            ),
            (
                "eth_1_data_reset",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|state, context| {
                        spec::process_eth1_data_reset(state, context);
                        Ok(())
                })"
                    .to_string())]),
                },
            ),
            (
                "historical_roots_update",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|state, context| {
                        spec::process_historical_roots_update(state, context)
                })"
                    .to_string())]),
                },
            ),
            (
                "inactivity_updates",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|state, context| {
                        spec::process_inactivity_updates(state, context)
                })"
                    .to_string())]),
                },
            ),
            (
                "justification_and_finalization",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|state, context| {
                        spec::process_justification_and_finalization(state, context)
                })"
                    .to_string())]),
                },
            ),
            (
                "participation_record_updates",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|state, context| {
                        spec::process_participation_record_updates(state);
                        Ok(())
                })"
                    .to_string())]),
                },
            ),
            (
                "participation_flag_updates",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|state, context| {
                        spec::process_participation_flag_updates(state)
                })"
                    .to_string())]),
                },
            ),
            (
                "randao_mixes_reset",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|state, context| {
                        spec::process_randao_mixes_reset(state, context);
                        Ok(())
                })"
                    .to_string())]),
                },
            ),
            (
                "registry_updates",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|state, context| {
                        spec::process_registry_updates(state, context);
                        Ok(())
                })"
                    .to_string())]),
                },
            ),
            (
                "rewards_and_penalties",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|state, context| {
                        spec::process_rewards_and_penalties(state, context)
                })"
                    .to_string())]),
                },
            ),
            (
                "slashings",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|state, context| {
                        spec::process_slashings(state, context)
                })"
                    .to_string())]),
                },
            ),
            (
                "slashings_reset",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|state, context| {
                        spec::process_slashings_reset(state, context);
                        Ok(())
                })"
                    .to_string())]),
                },
            ),
            (
                "sync_committee_updates",
                Auxillary {
                    test_case_type_generics: "spec::BeaconState".to_string(),
                    preamble: Default::default(),
                    execution_handler: HashMap::from_iter([(Spec::All, "execute(|state, context| {
                        spec::process_sync_committee_updates(state, context)
                })"
                    .to_string())]),
                },
            ),
        ])),
    ]);
    let test_root = "consensus-spec-tests/tests";
    let (all_test_cases, collected_test_case_count) = collect_test_cases(test_root).unwrap();

    println!("collected {collected_test_case_count} tests");

    let generated_test_root = PathBuf::from("tests/consensus_spec_tests");
    fs::remove_dir_all(&generated_test_root).unwrap();

    fs::create_dir_all(&generated_test_root).unwrap();
    let module_decl_src = generate_module_decl_src(all_test_cases.keys());
    let mut mod_path = generated_test_root.clone();
    mod_path.push("mod.rs");
    fs::write(mod_path, module_decl_src).unwrap();
    for (config, tests) in all_test_cases {
        let config = config.to_case(Case::Snake);
        let target = generated_test_root.join(&config);
        fs::create_dir_all(&target).unwrap();
        let module_decl_src = generate_module_decl_src(tests.keys());
        let mut mod_path = target.clone();
        mod_path.push("mod.rs");
        fs::write(mod_path, module_decl_src).unwrap();
        for (fork, tests) in tests {
            let fork = fork.to_case(Case::Camel);
            let target = target.join(&fork);
            fs::create_dir(&target).unwrap();
            let module_decl_src = generate_module_decl_src(tests.keys());
            let mut mod_path = target.clone();
            mod_path.push("mod.rs");
            fs::write(mod_path, module_decl_src).unwrap();
            for (runner, tests) in tests {
                let runner = runner.to_case(Case::Snake);
                let target = target.join(&runner);
                fs::create_dir(&target).unwrap();
                let module_decl_src = generate_module_decl_src(tests.keys());
                let mut mod_path = target.clone();
                mod_path.push("mod.rs");
                fs::write(mod_path, module_decl_src).unwrap();
                for (handler, tests) in tests {
                    let handler = handler.to_case(Case::Snake);
                    let target = target.join(&handler);
                    fs::create_dir(&target).unwrap();
                    let module_decl_src = generate_module_decl_src(tests.keys());
                    let mut mod_path = target.clone();
                    mod_path.push("mod.rs");
                    fs::write(mod_path, module_decl_src).unwrap();
                    for (suite, tests) in tests {
                        let suite = suite.to_case(Case::Snake);
                        let target = target.join(suite);
                        fs::create_dir(&target).unwrap();
                        let mut mod_path = target.clone();
                        mod_path.push("mod.rs");
                        // NOTE: batch test cases to reduce overall file count
                        // which strains git, IDEs, etc.
                        let suite_src = generate_suite_src(
                            &tests,
                            &fork,
                            &config,
                            &runner,
                            &handler,
                            &auxilliary_data,
                        );
                        fs::write(mod_path, suite_src).unwrap();
                    }
                }
            }
        }
    }
}
