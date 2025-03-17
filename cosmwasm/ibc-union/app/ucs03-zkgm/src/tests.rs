#[cfg(test)]
mod tests {
    use alloy::primitives::U256;
    use cosmwasm_std::{
        testing::{message_info, mock_dependencies},
        Addr, Response,
    };
    use ibc_union_spec::ChannelId;
    use unionlabs::primitives::H256;

    use crate::{
        com::{Instruction, Multiplex, FORWARD_SALT_MAGIC, INSTR_VERSION_0, OP_MULTIPLEX},
        contract::{
            dequeue_channel_from_path, is_forwarded_packet, pop_channel_from_path,
            reverse_channel_path, tint_forward_salt, update_channel_path, verify_internal,
            verify_multiplex,
        },
        ContractError,
    };

    #[test]
    fn test_verify_multiplex_sender() {
        let sender = Addr::unchecked("sender");
        // Test with matching sender
        let multiplex = Multiplex {
            sender: sender.as_bytes().to_vec().into(),
            eureka: false,
            contract_address: Addr::unchecked("contract").as_bytes().to_vec().into(),
            contract_calldata: vec![].into(),
        };
        let mut response = Response::new();
        let result = verify_multiplex(&multiplex, sender.clone(), &mut response);
        assert_eq!(result, Ok(()));
        assert_eq!(response, Response::new());
        // Test with non-matching sender
        let wrong_sender = Addr::unchecked("wrong_sender");
        let result = verify_multiplex(&multiplex, wrong_sender, &mut response);
        assert!(matches!(result, Err(ContractError::InvalidMultiplexSender)));
    }

    #[test]
    fn test_verify_internal_unsupported_version() {
        let deps = mock_dependencies();
        let info = message_info(&Addr::unchecked("sender"), &[]);

        let instruction = Instruction {
            version: 99, // Unsupported version
            opcode: OP_MULTIPLEX,
            operand: vec![].into(),
        };

        let mut response = Response::new();
        let result = verify_internal(
            deps.as_ref(),
            info,
            ChannelId!(1),
            U256::ZERO,
            &instruction,
            &mut response,
        );
        assert!(matches!(
            result,
            Err(ContractError::UnsupportedVersion { version: 99 })
        ));
    }

    #[test]
    fn test_verify_internal_unknown_opcode() {
        let deps = mock_dependencies();
        let info = message_info(&Addr::unchecked("sender"), &[]);

        let instruction = Instruction {
            version: INSTR_VERSION_0,
            opcode: 99, // Unknown opcode
            operand: vec![].into(),
        };

        let mut response = Response::new();
        let result = verify_internal(
            deps.as_ref(),
            info,
            ChannelId!(1),
            U256::ZERO,
            &instruction,
            &mut response,
        );
        assert!(matches!(
            result,
            Err(ContractError::UnknownOpcode { opcode: 99 })
        ));
    }

    #[test]
    fn test_dequeue_channel_from_path_ok_1() {
        let a = ChannelId!(42);
        let path = update_channel_path(U256::ZERO, a).unwrap();
        assert_eq!(dequeue_channel_from_path(path).1.unwrap(), a);
    }

    #[test]
    fn test_dequeue_channel_from_path_ok_2() {
        let a = ChannelId!(42);
        let b = ChannelId!(123);
        let path1 = update_channel_path(U256::ZERO, a).unwrap();
        let path2 = update_channel_path(path1, b).unwrap();
        let (remaining_path, last_channel) = dequeue_channel_from_path(path2);
        assert_eq!(last_channel.unwrap(), a);
        assert_eq!(dequeue_channel_from_path(remaining_path).1.unwrap(), b);
    }

    #[test]
    fn test_dequeue_channel_from_path_ok_3() {
        let a = ChannelId!(0xDEAD);
        let b = ChannelId!(0xC0DE);
        let c = ChannelId!(0xBEEF);
        let path1 = update_channel_path(U256::ZERO, a).unwrap();
        let path2 = update_channel_path(path1, b).unwrap();
        let path3 = update_channel_path(path2, c).unwrap();

        let (remaining_path1, last_channel1) = dequeue_channel_from_path(path3);
        assert_eq!(last_channel1.unwrap(), a);

        let (remaining_path2, last_channel2) = dequeue_channel_from_path(remaining_path1);
        assert_eq!(last_channel2.unwrap(), b);

        let (remaining_path3, last_channel3) = dequeue_channel_from_path(remaining_path2);
        assert_eq!(last_channel3.unwrap(), c);
        assert_eq!(remaining_path3, U256::ZERO);
    }

    #[test]
    fn test_channel_path_ok() {
        let a = ChannelId!(1);
        let b = ChannelId!(2);
        let c = ChannelId!(3);
        let d = ChannelId!(4);
        let e = ChannelId!(5);
        let f = ChannelId!(6);
        let g = ChannelId!(7);
        let h = ChannelId!(8);

        let path1 = update_channel_path(U256::ZERO, a).unwrap();
        let path2 = update_channel_path(path1, b).unwrap();
        let path3 = update_channel_path(path2, c).unwrap();
        let path4 = update_channel_path(path3, d).unwrap();
        let path5 = update_channel_path(path4, e).unwrap();
        let path6 = update_channel_path(path5, f).unwrap();
        let path7 = update_channel_path(path6, g).unwrap();
        let path8 = update_channel_path(path7, h).unwrap();

        let expected = U256::from(a.raw())
            | U256::from(b.raw()) << 32
            | U256::from(c.raw()) << 64
            | U256::from(d.raw()) << 96
            | U256::from(e.raw()) << 128
            | U256::from(f.raw()) << 160
            | U256::from(g.raw()) << 192
            | U256::from(h.raw()) << 224;

        assert_eq!(path8, expected);
    }

    #[test]
    fn test_reverse_channel_path_ok() {
        let a = ChannelId!(1);
        let b = ChannelId!(2);
        let c = ChannelId!(3);
        let d = ChannelId!(4);
        let e = ChannelId!(5);
        let f = ChannelId!(6);
        let g = ChannelId!(7);
        let h = ChannelId!(8);

        let path = update_channel_path(
            update_channel_path(
                update_channel_path(
                    update_channel_path(
                        update_channel_path(
                            update_channel_path(
                                update_channel_path(update_channel_path(U256::ZERO, a).unwrap(), b)
                                    .unwrap(),
                                c,
                            )
                            .unwrap(),
                            d,
                        )
                        .unwrap(),
                        e,
                    )
                    .unwrap(),
                    f,
                )
                .unwrap(),
                g,
            )
            .unwrap(),
            h,
        )
        .unwrap();

        let reversed = reverse_channel_path(path);

        let expected = U256::from(h.raw())
            | U256::from(g.raw()) << 32
            | U256::from(f.raw()) << 64
            | U256::from(e.raw()) << 96
            | U256::from(d.raw()) << 128
            | U256::from(c.raw()) << 160
            | U256::from(b.raw()) << 192
            | U256::from(a.raw()) << 224;

        assert_eq!(reversed, expected);
    }

    #[test]
    fn test_reverse_channel_path_iso() {
        let path = U256::from(0x1234567890abcdef_u64);
        assert_eq!(reverse_channel_path(reverse_channel_path(path)), path);
    }

    #[test]
    fn test_channel_path_full() {
        let mut path = U256::ZERO;
        for i in 1..=8 {
            path = update_channel_path(path, i.try_into().unwrap()).unwrap();
        }

        // Trying to add one more should fail with ChannelPathIsFull
        let result = update_channel_path(path, ChannelId!(9));
        assert!(result.is_err());
        match result {
            Err(ContractError::ChannelPathIsFull { .. }) => {}
            _ => panic!("Expected ChannelPathIsFull error"),
        }
    }

    #[test]
    fn test_pop_channel_from_path_ok_1() {
        let a = ChannelId!(42);
        let path = update_channel_path(U256::ZERO, a).unwrap();
        let (base_path, channel_id) = pop_channel_from_path(path);
        assert_eq!(channel_id.unwrap(), a);
        assert_eq!(base_path, U256::ZERO);
    }

    #[test]
    fn test_pop_channel_from_path_ok_2() {
        let a = ChannelId!(42);
        let b = ChannelId!(123);
        let path1 = update_channel_path(U256::ZERO, a).unwrap();
        let path2 = update_channel_path(path1, b).unwrap();

        let (base_path, channel_id) = pop_channel_from_path(path2);
        assert_eq!(channel_id.unwrap(), b);
        assert_eq!(base_path, path1);
    }

    #[test]
    fn test_pop_channel_from_path_ok_3() {
        let a = ChannelId!(0xDEAD);
        let b = ChannelId!(0xC0DE);
        let c = ChannelId!(0xBEEF);
        let path1 = update_channel_path(U256::ZERO, a).unwrap();
        let path2 = update_channel_path(path1, b).unwrap();
        let path3 = update_channel_path(path2, c).unwrap();

        let (base_path1, channel_id1) = pop_channel_from_path(path3);
        assert_eq!(channel_id1.unwrap(), c);
        assert_eq!(base_path1, path2);

        let (base_path2, channel_id2) = pop_channel_from_path(base_path1);
        assert_eq!(channel_id2.unwrap(), b);
        assert_eq!(base_path2, path1);

        let (base_path3, channel_id3) = pop_channel_from_path(base_path2);
        assert_eq!(channel_id3.unwrap(), a);
        assert_eq!(base_path3, U256::ZERO);
    }

    #[test]
    fn test_pop_channel_from_path_complex() {
        let a = ChannelId!(1);
        let b = ChannelId!(2);
        let c = ChannelId!(3);
        let d = ChannelId!(4);
        let e = ChannelId!(5);
        let f = ChannelId!(6);
        let g = ChannelId!(7);
        let h = ChannelId!(8);

        let path1 = update_channel_path(U256::ZERO, a).unwrap();
        let path2 = update_channel_path(path1, b).unwrap();
        let path3 = update_channel_path(path2, c).unwrap();
        let path4 = update_channel_path(path3, d).unwrap();
        let path5 = update_channel_path(path4, e).unwrap();
        let path6 = update_channel_path(path5, f).unwrap();
        let path7 = update_channel_path(path6, g).unwrap();
        let path8 = update_channel_path(path7, h).unwrap();

        let expected_base_path = update_channel_path(
            update_channel_path(
                update_channel_path(
                    update_channel_path(
                        update_channel_path(
                            update_channel_path(update_channel_path(U256::ZERO, a).unwrap(), b)
                                .unwrap(),
                            c,
                        )
                        .unwrap(),
                        d,
                    )
                    .unwrap(),
                    e,
                )
                .unwrap(),
                f,
            )
            .unwrap(),
            g,
        )
        .unwrap();

        let (base_path, channel_id) = pop_channel_from_path(path8);
        assert_eq!(channel_id.unwrap(), h);
        assert_eq!(base_path, expected_base_path);
    }

    #[test]
    fn test_pop_channel_from_path_zero() {
        let (base_path, channel_id) = pop_channel_from_path(U256::ZERO);
        assert_eq!(channel_id, None);
        assert_eq!(base_path, U256::ZERO);
    }

    #[test]
    fn test_tint_forward_salt_ok() {
        let salt = H256::from([
            0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF, 0x12, 0x34, 0x56, 0x78, 0x90, 0xAB,
            0xCD, 0xEF, 0x12, 0x34, 0x56, 0x78, 0x90, 0xAB, 0xCD, 0xEF, 0x12, 0x34, 0x56, 0x78,
            0x90, 0xAB, 0xCD, 0xEF,
        ]);
        assert!(!is_forwarded_packet(salt));
        assert!(is_forwarded_packet(tint_forward_salt(salt)));
    }

    #[test]
    fn test_tint_forward_salt_idempotent() {
        let salt = H256::from([
            0xFE, 0xDC, 0xBA, 0x09, 0x87, 0x65, 0x43, 0x21, 0xFE, 0xDC, 0xBA, 0x09, 0x87, 0x65,
            0x43, 0x21, 0xFE, 0xDC, 0xBA, 0x09, 0x87, 0x65, 0x43, 0x21, 0xFE, 0xDC, 0xBA, 0x09,
            0x87, 0x65, 0x43, 0x21,
        ]);
        let tinted = tint_forward_salt(salt);
        assert_eq!(tint_forward_salt(tinted), tinted);
    }

    #[test]
    fn test_tint_forward_salt_preserves_data() {
        let salt = H256::from([
            0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE, 0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE,
            0xBA, 0xBE, 0xDE, 0xAD, 0xBE, 0xEF, 0xCA, 0xFE, 0xBA, 0xBE, 0xDE, 0xAD, 0xBE, 0xEF,
            0xCA, 0xFE, 0xBA, 0xBE,
        ]);
        let tinted = tint_forward_salt(salt);
        // Check that non-magic bits are preserved
        let mask = !FORWARD_SALT_MAGIC;
        assert_eq!(
            U256::from_be_bytes(*salt.get()) & mask,
            U256::from_be_bytes(*tinted.get()) & mask
        );
    }
}
