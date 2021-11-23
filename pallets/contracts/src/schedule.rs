// This file is part of Substrate.

// Copyright (C) 2020-2021 Parity Technologies (UK) Ltd.
// SPDX-License-Identifier: Apache-2.0

// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
// 	http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! This module contains the cost schedule and supporting code that constructs a
//! sane default schedule from a `WeightInfo` implementation.

use crate::{Config, weights::WeightInfo};

#[cfg(feature = "std")]
use serde::{Serialize, Deserialize};
use pallet_contracts_proc_macro::{ScheduleDebug, WeightDebug};
use frame_support::weights::Weight;
use sp_std::{marker::PhantomData, vec::Vec};
use codec::{Encode, Decode};
use parity_wasm::elements;
use pwasm_utils::rules;
use sp_runtime::RuntimeDebug;

/// How many API calls are executed in a single batch. The reason for increasing the amount
/// of API calls in batches (per benchmark component increase) is so that the linear regression
/// has an easier time determining the contribution of that component.
pub const API_BENCHMARK_BATCH_SIZE: u32 = 100;

/// How many instructions are executed in a single batch. The reasoning is the same
/// as for `API_BENCHMARK_BATCH_SIZE`.
pub const INSTR_BENCHMARK_BATCH_SIZE: u32 = 1_000;

/// Definition of the cost schedule and other parameterizations for the wasm vm.
///
/// Its fields are private to the crate in order to allow addition of new contract
/// callable functions without bumping to a new major version. A genesis config should
/// rely on public functions of this type.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[cfg_attr(feature = "std", serde(bound(serialize = "", deserialize = "")))]
#[derive(Clone, Encode, Decode, PartialEq, Eq, ScheduleDebug)]
pub struct Schedule<T: Config> {
	/// Version of the schedule.
	///
	/// # Note
	///
	/// Must be incremented whenever the [`self.instruction_weights`] are changed. The
	/// reason is that changes to instruction weights require a re-instrumentation
	/// of all contracts which are triggered by a version comparison on call.
	/// Changes to other parts of the schedule should not increment the version in
	/// order to avoid unnecessary re-instrumentations.
	pub(crate) version: u32,

	/// Whether the `seal_println` function is allowed to be used contracts.
	/// MUST only be enabled for `dev` chains, NOT for production chains
	pub(crate) enable_println: bool,

	/// Describes the upper limits on various metrics.
	pub(crate) limits: Limits,

	/// The weights for individual wasm instructions.
	pub(crate) instruction_weights: InstructionWeights<T>,

	/// The weights for each imported function a contract is allowed to call.
	pub(crate) host_fn_weights: HostFnWeights<T>,
}

/// Describes the upper limits on various metrics.
///
/// # Note
///
/// The values in this struct should only ever be increased for a deployed chain. The reason
/// is that decreasing those values will break existing contracts which are above the new limits.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, PartialEq, Eq, RuntimeDebug)]
pub struct Limits {
	/// The maximum number of topics supported by an event.
	pub event_topics: u32,

	/// Maximum allowed stack height in number of elements.
	///
	/// See <https://wiki.parity.io/WebAssembly-StackHeight> to find out
	/// how the stack frame cost is calculated. Each element can be of one of the
	/// wasm value types. This means the maximum size per element is 64bit.
	pub stack_height: u32,

	/// Maximum number of globals a module is allowed to declare.
	///
	/// Globals are not limited through the `stack_height` as locals are. Neither does
	/// the linear memory limit `memory_pages` applies to them.
	pub globals: u32,

	/// Maximum numbers of parameters a function can have.
	///
	/// Those need to be limited to prevent a potentially exploitable interaction with
	/// the stack height instrumentation: The costs of executing the stack height
	/// instrumentation for an indirectly called function scales linearly with the amount
	/// of parameters of this function. Because the stack height instrumentation itself is
	/// is not weight metered its costs must be static (via this limit) and included in
	/// the costs of the instructions that cause them (call, call_indirect).
	pub parameters: u32,

	/// Maximum number of memory pages allowed for a contract.
	pub memory_pages: u32,

	/// Maximum number of elements allowed in a table.
	///
	/// Currently, the only type of element that is allowed in a table is funcref.
	pub table_size: u32,

	/// Maximum number of elements that can appear as immediate value to the br_table instruction.
	pub br_table_size: u32,

	/// The maximum length of a subject in bytes used for PRNG generation.
	pub subject_len: u32,
}

impl Limits {
	/// The maximum memory size in bytes that a contract can occupy.
	pub fn max_memory_size(&self) -> u32 {
		self.memory_pages * 64 * 1024
	}
}

/// Describes the weight for all categories of supported wasm instructions.
///
/// There there is one field for each wasm instruction that describes the weight to
/// execute one instruction of that name. There are a few execptions:
///
/// 1. If there is a i64 and a i32 variant of an instruction we use the weight
///    of the former for both.
/// 2. The following instructions are free of charge because they merely structure the
///    wasm module and cannot be spammed without making the module invalid (and rejected):
///    End, Unreachable, Return, Else
/// 3. The following instructions cannot be benchmarked because they are removed by any
///    real world execution engine as a preprocessing step and therefore don't yield a
///    meaningful benchmark result. However, in contrast to the instructions mentioned
///    in 2. they can be spammed. We price them with the same weight as the "default"
///    instruction (i64.const): Block, Loop, Nop
/// 4. We price both i64.const and drop as InstructionWeights.i64const / 2. The reason
///    for that is that we cannot benchmark either of them on its own but we need their
///    individual values to derive (by subtraction) the weight of all other instructions
///    that use them as supporting instructions. Supporting means mainly pushing arguments
///    and dropping return values in order to maintain a valid module.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, PartialEq, Eq, WeightDebug)]
pub struct InstructionWeights<T: Config> {
	pub i64const: u32,
	pub i64load: u32,
	pub i64store: u32,
	pub select: u32,
	pub r#if: u32,
	pub br: u32,
	pub br_if: u32,
	pub br_table: u32,
	pub br_table_per_entry: u32,
	pub call: u32,
	pub call_indirect: u32,
	pub call_indirect_per_param: u32,
	pub local_get: u32,
	pub local_set: u32,
	pub local_tee: u32,
	pub global_get: u32,
	pub global_set: u32,
	pub memory_current: u32,
	pub memory_grow: u32,
	pub i64clz: u32,
	pub i64ctz: u32,
	pub i64popcnt: u32,
	pub i64eqz: u32,
	pub i64extendsi32: u32,
	pub i64extendui32: u32,
	pub i32wrapi64: u32,
	pub i64eq: u32,
	pub i64ne: u32,
	pub i64lts: u32,
	pub i64ltu: u32,
	pub i64gts: u32,
	pub i64gtu: u32,
	pub i64les: u32,
	pub i64leu: u32,
	pub i64ges: u32,
	pub i64geu: u32,
	pub i64add: u32,
	pub i64sub: u32,
	pub i64mul: u32,
	pub i64divs: u32,
	pub i64divu: u32,
	pub i64rems: u32,
	pub i64remu: u32,
	pub i64and: u32,
	pub i64or: u32,
	pub i64xor: u32,
	pub i64shl: u32,
	pub i64shrs: u32,
	pub i64shru: u32,
	pub i64rotl: u32,
	pub i64rotr: u32,
	/// The type parameter is used in the default implementation.
	#[codec(skip)]
	pub _phantom: PhantomData<T>,
}

/// Describes the weight for each imported function that a contract is allowed to call.
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
#[derive(Clone, Encode, Decode, PartialEq, Eq, WeightDebug)]
pub struct HostFnWeights<T: Config> {
	/// Weight of calling `seal_caller`.
	pub caller: Weight,

	/// Weight of calling `seal_address`.
	pub address: Weight,

	/// Weight of calling `seal_gas_left`.
	pub gas_left: Weight,

	/// Weight of calling `seal_balance`.
	pub balance: Weight,

	/// Weight of calling `seal_value_transferred`.
	pub value_transferred: Weight,

	/// Weight of calling `seal_minimum_balance`.
	pub minimum_balance: Weight,

	/// Weight of calling `seal_tombstone_deposit`.
	pub tombstone_deposit: Weight,

	/// Weight of calling `seal_rent_allowance`.
	pub rent_allowance: Weight,

	/// Weight of calling `seal_block_number`.
	pub block_number: Weight,

	/// Weight of calling `seal_now`.
	pub now: Weight,

	/// Weight of calling `seal_weight_to_fee`.
	pub weight_to_fee: Weight,

	/// Weight of calling `gas`.
	pub gas: Weight,

	/// Weight of calling `seal_input`.
	pub input: Weight,

	/// Weight per input byte copied to contract memory by `seal_input`.
	pub input_per_byte: Weight,

	/// Weight of calling `seal_return`.
	pub r#return: Weight,

	/// Weight per byte returned through `seal_return`.
	pub return_per_byte: Weight,

	/// Weight of calling `seal_terminate`.
	pub terminate: Weight,

	/// Weight per byte of the terminated contract.
	pub terminate_per_code_byte: Weight,

	/// Weight of calling `seal_restore_to`.
	pub restore_to: Weight,

	/// Weight per byte of the restoring contract.
	pub restore_to_per_caller_code_byte: Weight,

	/// Weight per byte of the restored contract.
	pub restore_to_per_tombstone_code_byte: Weight,

	/// Weight per delta key supplied to `seal_restore_to`.
	pub restore_to_per_delta: Weight,

	/// Weight of calling `seal_random`.
	pub random: Weight,

	/// Weight of calling `seal_reposit_event`.
	pub deposit_event: Weight,

	/// Weight per topic supplied to `seal_deposit_event`.
	pub deposit_event_per_topic: Weight,

	/// Weight per byte of an event deposited through `seal_deposit_event`.
	pub deposit_event_per_byte: Weight,

	/// Weight of calling `seal_set_rent_allowance`.
	pub set_rent_allowance: Weight,

	/// Weight of calling `seal_set_storage`.
	pub set_storage: Weight,

	/// Weight per byte of an item stored with `seal_set_storage`.
	pub set_storage_per_byte: Weight,

	/// Weight of calling `seal_clear_storage`.
	pub clear_storage: Weight,

	/// Weight of calling `seal_get_storage`.
	pub get_storage: Weight,

	/// Weight per byte of an item received via `seal_get_storage`.
	pub get_storage_per_byte: Weight,

	/// Weight of calling `seal_transfer`.
	pub transfer: Weight,

	/// Weight of calling `seal_call`.
	pub call: Weight,

	/// Weight per byte of the called contract.
	pub call_per_code_byte: Weight,

	/// Weight surcharge that is claimed if `seal_call` does a balance transfer.
	pub call_transfer_surcharge: Weight,

	/// Weight per input byte supplied to `seal_call`.
	pub call_per_input_byte: Weight,

	/// Weight per output byte received through `seal_call`.
	pub call_per_output_byte: Weight,

	/// Weight of calling `seal_instantiate`.
	pub instantiate: Weight,

	/// Weight per byte of the instantiated contract.
	pub instantiate_per_code_byte: Weight,

	/// Weight per input byte supplied to `seal_instantiate`.
	pub instantiate_per_input_byte: Weight,

	/// Weight per output byte received through `seal_instantiate`.
	pub instantiate_per_output_byte: Weight,

	/// Weight per salt byte supplied to `seal_instantiate`.
	pub instantiate_per_salt_byte: Weight,

	/// Weight of calling `seal_hash_sha_256`.
	pub hash_sha2_256: Weight,

	/// Weight per byte hashed by `seal_hash_sha_256`.
	pub hash_sha2_256_per_byte: Weight,

	/// Weight of calling `seal_hash_keccak_256`.
	pub hash_keccak_256: Weight,

	/// Weight per byte hashed by `seal_hash_keccak_256`.
	pub hash_keccak_256_per_byte: Weight,

	/// Weight of calling `seal_hash_blake2_256`.
	pub hash_blake2_256: Weight,

	/// Weight per byte hashed by `seal_hash_blake2_256`.
	pub hash_blake2_256_per_byte: Weight,

	/// Weight of calling `seal_hash_blake2_128`.
	pub hash_blake2_128: Weight,

	/// Weight per byte hashed by `seal_hash_blake2_128`.
	pub hash_blake2_128_per_byte: Weight,

	/// Weight of calling `seal_rent_params`.
	pub rent_params: Weight,

	/// The type parameter is used in the default implementation.
	#[codec(skip)]
	pub _phantom: PhantomData<T>
}

macro_rules! replace_token {
	($_in:tt $replacement:tt) => { $replacement };
}

macro_rules! call_zero {
	($name:ident, $( $arg:expr ),*) => {
		T::WeightInfo::$name($( replace_token!($arg 0) ),*)
	};
}

macro_rules! cost_args {
	($name:ident, $( $arg: expr ),+) => {
		(T::WeightInfo::$name($( $arg ),+).saturating_sub(call_zero!($name, $( $arg ),+)))
	}
}

macro_rules! cost_batched_args {
	($name:ident, $( $arg: expr ),+) => {
		cost_args!($name, $( $arg ),+) / Weight::from(API_BENCHMARK_BATCH_SIZE)
	}
}

macro_rules! cost_instr_no_params_with_batch_size {
	($name:ident, $batch_size:expr) => {
		(cost_args!($name, 1) / Weight::from($batch_size)) as u32
	}
}

macro_rules! cost_instr_with_batch_size {
	($name:ident, $num_params:expr, $batch_size:expr) => {
		cost_instr_no_params_with_batch_size!($name, $batch_size)
			.saturating_sub((cost_instr_no_params_with_batch_size!(instr_i64const, $batch_size) / 2).saturating_mul($num_params))
	}
}

macro_rules! cost_instr {
	($name:ident, $num_params:expr) => {
		cost_instr_with_batch_size!($name, $num_params, INSTR_BENCHMARK_BATCH_SIZE)
	}
}

macro_rules! cost_byte_args {
	($name:ident, $( $arg: expr ),+) => {
		cost_args!($name, $( $arg ),+) / 1024
	}
}

macro_rules! cost_byte_batched_args {
	($name:ident, $( $arg: expr ),+) => {
		cost_batched_args!($name, $( $arg ),+) / 1024
	}
}

macro_rules! cost {
	($name:ident) => {
		cost_args!($name, 1)
	}
}

macro_rules! cost_batched {
	($name:ident) => {
		cost_batched_args!($name, 1)
	}
}

macro_rules! cost_byte {
	($name:ident) => {
		cost_byte_args!($name, 1)
	}
}

macro_rules! cost_byte_batched {
	($name:ident) => {
		cost_byte_batched_args!($name, 1)
	}
}

impl<T: Config> Default for Schedule<T> {
	fn default() -> Self {
		Self {
			version: 0,
			enable_println: false,
			limits: Default::default(),
			instruction_weights: Default::default(),
			host_fn_weights: Default::default(),
		}
	}
}

impl Default for Limits {
	fn default() -> Self {
		Self {
			event_topics: 4,
			// 512 * sizeof(i64) will give us a 4k stack.
			stack_height: 512,
			globals: 256,
			parameters: 128,
			memory_pages: 16,
			// 4k function pointers (This is in count not bytes).
			table_size: 4096,
			br_table_size: 256,
			subject_len: 32,
		}
	}
}

impl<T: Config> Default for InstructionWeights<T> {
	fn default() -> Self {
		let max_pages = Limits::default().memory_pages;
		Self {
			i64const: 0,
			i64load: 0,
			i64store: 0,
			select: 0,
			r#if: 0,
			br: 0,
			br_if: 0,
			br_table: 0,
			br_table_per_entry: 0,
			call: 0,
			call_indirect: 0,
			call_indirect_per_param: 0,
			local_get: 0,
			local_set: 0,
			local_tee: 0,
			global_get: 0,
			global_set: 0,
			memory_current: 0,
			memory_grow: 0,
			i64clz: 0,
			i64ctz: 0,
			i64popcnt: 0,
			i64eqz: 0,
			i64extendsi32: 0,
			i64extendui32: 0,
			i32wrapi64: 0,
			i64eq: 0,
			i64ne:0,
			i64lts: 0,
			i64ltu: 0,
			i64gts: 0,
			i64gtu: 0,
			i64les: 0,
			i64leu: 0,
			i64ges: 0,
			i64geu: 0,
			i64add: 0,
			i64sub: 0,
			i64mul: 0,
			i64divs: 0,
			i64divu: 0,
			i64rems: 0,
			i64remu: 0,
			i64and: 0,
			i64or: 0,
			i64xor: 0,
			i64shl:0,
			i64shrs: 0,
			i64shru: 0,
			i64rotl:0,
			i64rotr:0,
			_phantom: PhantomData,
		}
	}
}

impl<T: Config> Default for HostFnWeights<T> {
	fn default() -> Self {
		Self {
			caller:0,
			address: 0,
			gas_left: 0,
			balance: 0,
			value_transferred: 0,
			minimum_balance: 0,
			tombstone_deposit: 0,
			rent_allowance: 0,
			block_number: 0,
			now: 0,
			weight_to_fee: 0,
			gas: 0,
			input:0,
			input_per_byte: 0,
			r#return: 0,
			return_per_byte: 0,
			terminate: 0,
			terminate_per_code_byte: 0,
			restore_to: 0,
			restore_to_per_caller_code_byte:0,
			restore_to_per_tombstone_code_byte: 0,
			restore_to_per_delta: 0,
			random: 0,
			deposit_event: 0,
			deposit_event_per_topic: 0,
			deposit_event_per_byte: 0,
			set_rent_allowance: 0,
			set_storage:0,
			set_storage_per_byte: 0,
			clear_storage: 0,
			get_storage: 0,
			get_storage_per_byte: 0,
			transfer: 0,
			call: 0,
			call_per_code_byte:0,
			call_transfer_surcharge: 0,
			call_per_input_byte: 0,
			call_per_output_byte: 0,
			instantiate:0,
			instantiate_per_code_byte: 0,
			instantiate_per_input_byte: 0,
			instantiate_per_output_byte: 0,
			instantiate_per_salt_byte: 0,
			hash_sha2_256:0,
			hash_sha2_256_per_byte: 0,
			hash_keccak_256: 0,
			hash_keccak_256_per_byte: 0,
			hash_blake2_256: 0,
			hash_blake2_256_per_byte: 0,
			hash_blake2_128: 0,
			hash_blake2_128_per_byte: 0,
			rent_params: 0,
			_phantom: PhantomData,
		}
	}
}

struct ScheduleRules<'a, T: Config> {
	schedule: &'a Schedule<T>,
	params: Vec<u32>,
}

impl<T: Config> Schedule<T> {
	/// Allow contracts to call `seal_println` in order to print messages to the console.
	///
	/// This should only ever be activated in development chains. The printed messages
	/// can be observed on the console by setting the environment variable
	/// `RUST_LOG=runtime=debug` when running the node.
	///
	/// # Note
	///
	/// Is set to `false` by default.
	pub fn enable_println(mut self, enable: bool) -> Self {
		self.enable_println = enable;
		self
	}

	pub(crate) fn rules(&self, module: &elements::Module) -> impl rules::Rules + '_ {
		ScheduleRules {
			schedule: &self,
			params: module
				.type_section()
				.iter()
				.flat_map(|section| section.types())
				.map(|func| {
					let elements::Type::Function(func) = func;
					func.params().len() as u32
				})
				.collect()
		}
	}
}

impl<'a, T: Config> rules::Rules for ScheduleRules<'a, T> {
	fn instruction_cost(&self, instruction: &elements::Instruction) -> Option<u32> {
		use parity_wasm::elements::Instruction::*;
		let w = &self.schedule.instruction_weights;
		let max_params = self.schedule.limits.parameters;

		let weight = match *instruction {
			End | Unreachable | Return | Else => 0,
			I32Const(_) | I64Const(_) | Block(_) | Loop(_) | Nop | Drop => w.i64const,
			I32Load(_, _) | I32Load8S(_, _) | I32Load8U(_, _) | I32Load16S(_, _) |
			I32Load16U(_, _) | I64Load(_, _) | I64Load8S(_, _) | I64Load8U(_, _) |
			I64Load16S(_, _) | I64Load16U(_, _) | I64Load32S(_, _) | I64Load32U(_, _)
				=> w.i64load,
			I32Store(_, _) | I32Store8(_, _) | I32Store16(_, _) | I64Store(_, _) |
			I64Store8(_, _) | I64Store16(_, _) | I64Store32(_, _) => w.i64store,
			Select => w.select,
			If(_) => w.r#if,
			Br(_) => w.br,
			BrIf(_) => w.br_if,
			Call(_) => w.call,
			GetLocal(_) => w.local_get,
			SetLocal(_) => w.local_set,
			TeeLocal(_) => w.local_tee,
			GetGlobal(_) => w.global_get,
			SetGlobal(_) => w.global_set,
			CurrentMemory(_) => w.memory_current,
			GrowMemory(_) => w.memory_grow,
			CallIndirect(idx, _) => 0,
			BrTable(ref data) => 0,
			I32Clz | I64Clz => w.i64clz,
			I32Ctz | I64Ctz => w.i64ctz,
			I32Popcnt | I64Popcnt => w.i64popcnt,
			I32Eqz | I64Eqz => w.i64eqz,
			I64ExtendSI32 => w.i64extendsi32,
			I64ExtendUI32 => w.i64extendui32,
			I32WrapI64 => w.i32wrapi64,
			I32Eq | I64Eq => w.i64eq,
			I32Ne | I64Ne => w.i64ne,
			I32LtS | I64LtS => w.i64lts,
			I32LtU | I64LtU => w.i64ltu,
			I32GtS | I64GtS => w.i64gts,
			I32GtU | I64GtU => w.i64gtu,
			I32LeS | I64LeS => w.i64les,
			I32LeU | I64LeU => w.i64leu,
			I32GeS | I64GeS => w.i64ges,
			I32GeU | I64GeU => w.i64geu,
			I32Add | I64Add => w.i64add,
			I32Sub | I64Sub => w.i64sub,
			I32Mul | I64Mul => w.i64mul,
			I32DivS | I64DivS => w.i64divs,
			I32DivU | I64DivU => w.i64divu,
			I32RemS | I64RemS => w.i64rems,
			I32RemU | I64RemU => w.i64remu,
			I32And | I64And => w.i64and,
			I32Or | I64Or => w.i64or,
			I32Xor | I64Xor => w.i64xor,
			I32Shl | I64Shl => w.i64shl,
			I32ShrS | I64ShrS => w.i64shrs,
			I32ShrU | I64ShrU => w.i64shru,
			I32Rotl | I64Rotl => w.i64rotl,
			I32Rotr | I64Rotr => w.i64rotr,

			// Returning None makes the gas instrumentation fail which we intend for
			// unsupported or unknown instructions.
			_ => return None,
		};
		Some(weight)
	}

	fn memory_grow_cost(&self) -> Option<rules::MemoryGrowCost> {
		// We benchmarked the memory.grow instruction with the maximum allowed pages.
		// The cost for growing is therefore already included in the instruction cost.
		None
	}
}

#[cfg(test)]
mod test {
	use crate::tests::Test;
	use super::*;

	#[test]
	fn print_test_schedule() {
		let schedule = Schedule::<Test>::default();
		println!("{:#?}", schedule);
	}
}
