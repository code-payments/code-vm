#[macro_export]
macro_rules! advance_poh {
    ($ctx:ident, $vm:ident, $hash:expr, $event:expr) => {
        crate::advance_poh_without_event!($ctx, $vm, $hash);
        crate::log_event!($ctx, $vm, $event);
    };
}

#[macro_export]
macro_rules! advance_poh_without_event {
    ($ctx:ident, $vm:ident, $hash:expr) => {
        let poh  = $vm.advance_poh($hash);
        let slot = $vm.advance_slot();

        $ctx.accounts.vm.slot = slot;
        $ctx.accounts.vm.poh  = poh;
    };
}

#[macro_export]
macro_rules! log_event {
    ($ctx:ident, $vm:ident, $event:expr) => {
        crate::cvm::ChangeLog::push(
            $ctx.accounts.vm.to_account_info().as_ref(), 
            crate::cvm::ChangeLogEvent {
                id: $vm.get_current_poh(),
                seq: $vm.get_current_slot(),
                data: $event,
            }
        )?;
    };
}

#[macro_export]
macro_rules! get_message_hash {
    ($ctx:ident, $vm:ident, $ix:ident, $($name:ident),*) => {{

        let blockhash = $vm.get_current_poh();
        let args = instruction::$ix {
            $(
                $name,
            )*
        };

        let accounts = $ctx.accounts.to_account_metas(None);
        let data = args.try_to_vec().unwrap();
        let ix = vec![
            Instruction {
                program_id: program::CodeVm::id(),
                accounts,
                data: [
                    instruction::$ix::DISCRIMINATOR.to_vec(),
                    data,
                ].concat(),
            }
        ];
        
        let message = crate::utils::message_with_sorted_keys(
            &ix,
            Some(&$vm.get_authority()),
            &blockhash,
        );

        let message = message.serialize();
        crate::utils::hash(&message)
    }};
}
