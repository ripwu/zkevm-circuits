use crate::{
    evm_circuit::{
        param::N_BYTES_GAS,
        table::{AccountFieldTag, FixedTableTag, Lookup},
        util::{
            constraint_builder::{ConstraintBuilder, ReversionInfo, StepStateTransition},
            math_gadget::{AddWordsGadget, RangeCheckGadget},
            Cell, Word,
        },
        witness::ExecStep,
    },
    util::Expr,
};
use eth_types::{Field, U256};
use halo2_proofs::{
    circuit::Region,
    plonk::{Error, Expression},
};
use std::convert::TryInto;

/// Construction of execution state that stays in the same call context, which
/// lookups the opcode and verifies the execution state is responsible for it,
/// then calculates the gas_cost and constrain the state transition.
#[derive(Clone, Debug)]
pub(crate) struct SameContextGadget<F> {
    opcode: Cell<F>,
    sufficient_gas_left: RangeCheckGadget<F, N_BYTES_GAS>,
}

impl<F: Field> SameContextGadget<F> {
    pub(crate) fn construct(
        cb: &mut ConstraintBuilder<F>,
        opcode: Cell<F>,
        step_state_transition: StepStateTransition<F>,
    ) -> Self {
        cb.opcode_lookup(opcode.expr(), 1.expr());
        cb.add_lookup(
            "Responsible opcode lookup",
            Lookup::Fixed {
                tag: FixedTableTag::ResponsibleOpcode.expr(),
                values: [
                    cb.execution_state().as_u64().expr(),
                    opcode.expr(),
                    0.expr(),
                ],
            },
        );

        // Check gas_left is sufficient
        let sufficient_gas_left = RangeCheckGadget::construct(cb, cb.next.state.gas_left.expr());

        // State transition
        cb.require_step_state_transition(step_state_transition);

        Self {
            opcode,
            sufficient_gas_left,
        }
    }

    pub(crate) fn assign_exec_step(
        &self,
        region: &mut Region<'_, F>,
        offset: usize,
        step: &ExecStep,
    ) -> Result<(), Error> {
        let opcode = step.opcode.unwrap();
        self.opcode
            .assign(region, offset, Some(F::from(opcode.as_u64())))?;

        self.sufficient_gas_left.assign(
            region,
            offset,
            F::from((step.gas_left - step.gas_cost) as u64),
        )?;

        Ok(())
    }
}

#[derive(Clone, Debug)]
pub(crate) struct UpdateBalanceGadget<F, const N_ADDENDS: usize, const INCREASE: bool> {
    add_words: AddWordsGadget<F, N_ADDENDS, true>,
}

impl<F: Field, const N_ADDENDS: usize, const INCREASE: bool>
    UpdateBalanceGadget<F, N_ADDENDS, INCREASE>
{
    pub(crate) fn construct(
        cb: &mut ConstraintBuilder<F>,
        address: Expression<F>,
        updates: Vec<Word<F>>,
        reversion_info: Option<ReversionInfo<F>>,
    ) -> Self {
        debug_assert!(updates.len() == N_ADDENDS - 1);

        let balance_addend = cb.query_word();
        let balance_sum = cb.query_word();

        let [value, value_prev] = if INCREASE {
            [balance_sum.expr(), balance_addend.expr()]
        } else {
            [balance_addend.expr(), balance_sum.expr()]
        };

        let add_words = AddWordsGadget::construct(
            cb,
            std::iter::once(balance_addend)
                .chain(updates.to_vec())
                .collect::<Vec<_>>()
                .try_into()
                .unwrap(),
            balance_sum,
        );

        cb.account_write(
            address,
            AccountFieldTag::Balance,
            value,
            value_prev,
            reversion_info,
        );

        Self { add_words }
    }

    pub(crate) fn assign(
        &self,
        region: &mut Region<'_, F>,
        offset: usize,
        addends: Vec<U256>,
        sum: U256,
    ) -> Result<(), Error> {
        debug_assert!(addends.len() == N_ADDENDS);

        self.add_words
            .assign(region, offset, addends.try_into().unwrap(), sum)
    }
}

#[derive(Clone, Debug)]
pub(crate) struct TransferWithGasFeeGadget<F> {
    sender: UpdateBalanceGadget<F, 3, false>,
    receiver: UpdateBalanceGadget<F, 2, true>,
}

impl<F: Field> TransferWithGasFeeGadget<F> {
    pub(crate) fn construct(
        cb: &mut ConstraintBuilder<F>,
        sender_address: Expression<F>,
        receiver_address: Expression<F>,
        value: Word<F>,
        gas_fee: Word<F>,
        is_persistent: Expression<F>,
        rw_counter_end_of_reversion: Expression<F>,
    ) -> Self {
        let sender = UpdateBalanceGadget::construct(
            cb,
            sender_address,
            vec![value.clone(), gas_fee],
            Some((&is_persistent, &rw_counter_end_of_reversion).into()),
        );
        let receiver = UpdateBalanceGadget::construct(
            cb,
            receiver_address,
            vec![value],
            Some((is_persistent, rw_counter_end_of_reversion - 1.expr()).into()),
        );

        Self { sender, receiver }
    }

    pub(crate) fn assign(
        &self,
        region: &mut Region<'_, F>,
        offset: usize,
        (sender_balance, sender_balance_prev): (U256, U256),
        (receiver_balance, receiver_balance_prev): (U256, U256),
        value: U256,
        gas_fee: U256,
    ) -> Result<(), Error> {
        self.sender.assign(
            region,
            offset,
            vec![sender_balance, value, gas_fee],
            sender_balance_prev,
        )?;
        self.receiver.assign(
            region,
            offset,
            vec![receiver_balance_prev, value],
            receiver_balance,
        )?;
        Ok(())
    }
}
