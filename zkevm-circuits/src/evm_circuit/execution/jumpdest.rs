use crate::{
    evm_circuit::{
        execution::ExecutionGadget,
        step::ExecutionState,
        util::{
            common_gadget::SameContextGadget,
            constraint_builder::{ConstraintBuilder, StepStateTransition, Transition::Delta},
        },
        witness::{Block, Call, ExecStep, Transaction},
    },
    util::Expr,
};
use bus_mapping::evm::OpcodeId;
use eth_types::Field;
use halo2_proofs::{circuit::Region, plonk::Error};

#[derive(Clone, Debug)]
pub(crate) struct JumpdestGadget<F> {
    same_context: SameContextGadget<F>,
}

impl<F: Field> ExecutionGadget<F> for JumpdestGadget<F> {
    const NAME: &'static str = "JUMPDEST";

    const EXECUTION_STATE: ExecutionState = ExecutionState::JUMPDEST;

    fn configure(cb: &mut ConstraintBuilder<F>) -> Self {
        // State transition
        let step_state_transition = StepStateTransition {
            program_counter: Delta(1.expr()),
            gas_left: Delta(-OpcodeId::JUMPDEST.constant_gas_cost().expr()),
            ..Default::default()
        };
        let opcode = cb.query_cell();
        let same_context = SameContextGadget::construct(cb, opcode, step_state_transition);

        Self { same_context }
    }

    fn assign_exec_step(
        &self,
        region: &mut Region<'_, F>,
        offset: usize,
        _: &Block<F>,
        _: &Transaction,
        _: &Call,
        step: &ExecStep,
    ) -> Result<(), Error> {
        self.same_context.assign_exec_step(region, offset, step)
    }
}

#[cfg(test)]
mod test {
    use crate::test_util::run_test_circuits;
    use eth_types::bytecode;

    fn test_ok() {
        let bytecode = bytecode! {
            JUMPDEST
            STOP
        };

        assert_eq!(run_test_circuits(bytecode), Ok(()));
    }

    #[test]
    fn jumpdest_gadget_simple() {
        test_ok();
    }
}
