use prog_vm::instruction::*;

#[derive(Debug, Clone)]
pub(crate) struct IntermediateLabel {
	pub(crate) name: String,
	pub(crate) instructions: Vec<Instruction>
}

impl IntermediateLabel {
	/// Flattens the intermediate label to a list of instructions containing the following:
	/// 1. `LABEL` instruction
	/// 2. Intermediate label's instructions
	pub(crate) fn flatten(&self) -> Vec<Instruction> {
		let mut emitted = vec![];

		emitted.push(Instruction::LABEL(LABEL {
			name: self.name.clone(),
			start: 0,
			length: self.instructions.len()
		}));
		emitted.extend_from_slice(&self.instructions);

		emitted
	}
}

#[derive(Debug, Clone)]
pub(crate) struct ConditionalBranch {
	pub(crate) condition: IntermediateLabel,
	pub(crate) body: IntermediateLabel
}

#[derive(Debug, Clone)]
pub(crate) struct UnconditionalBranch {
	pub(crate) body: IntermediateLabel
}
