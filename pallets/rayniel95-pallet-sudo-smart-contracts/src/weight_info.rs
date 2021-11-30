use pallet_contracts::weights::WeightInfo;
use frame_support::weights::Weight;

pub struct RainyelWeight{}

impl WeightInfo for RainyelWeight{
    fn on_initialize() -> Weight{
        0
    }
	fn on_initialize_per_trie_key(k: u32, ) -> Weight{
        0
    }
	fn on_initialize_per_queue_item(q: u32, ) -> Weight{
        0
    }
	fn instrument(c: u32, ) -> Weight{
        0
    }
	fn update_schedule() -> Weight{
        0
    }
	fn instantiate_with_code(c: u32, s: u32, ) -> Weight{
        0
    }
	fn instantiate(c: u32, s: u32, ) -> Weight{
        0
    }
	fn call(c: u32, ) -> Weight{
        0
    }
	fn claim_surcharge(c: u32, ) -> Weight{
        0
    }
	fn seal_caller(r: u32, ) -> Weight{
        0
    }
	fn seal_address(r: u32, ) -> Weight{
        0
    }
	fn seal_gas_left(r: u32, ) -> Weight{
        0
    }
	fn seal_balance(r: u32, ) -> Weight{
        0
    }
	fn seal_value_transferred(r: u32, ) -> Weight{
        0
    }
	fn seal_minimum_balance(r: u32, ) -> Weight{
        0
    }
	fn seal_tombstone_deposit(r: u32, ) -> Weight{
        0
    }
	fn seal_rent_allowance(r: u32, ) -> Weight{
        0
    }
	fn seal_block_number(r: u32, ) -> Weight{
        0
    }
	fn seal_now(r: u32, ) -> Weight{
        0
    }
	fn seal_rent_params(r: u32, ) -> Weight{
        0
    }
	fn seal_weight_to_fee(r: u32, ) -> Weight{
        0
    }
	fn seal_gas(r: u32, ) -> Weight{
        0
    }
	fn seal_input(r: u32, ) -> Weight{
        0
    }
	fn seal_input_per_kb(n: u32, ) -> Weight{
        0
    }
	fn seal_return(r: u32, ) -> Weight{
        0
    }
	fn seal_return_per_kb(n: u32, ) -> Weight{
        0
    }
	fn seal_terminate(r: u32, ) -> Weight{
        0
    }
	fn seal_terminate_per_code_kb(c: u32, ) -> Weight{
        0
    }
	fn seal_restore_to(r: u32, ) -> Weight{
        0
    }
	fn seal_restore_to_per_code_kb_delta(c: u32, t: u32, d: u32, ) -> Weight{
        0
    }
	fn seal_random(r: u32, ) -> Weight{
        0
    }
	fn seal_deposit_event(r: u32, ) -> Weight{
        0
    }
	fn seal_deposit_event_per_topic_and_kb(t: u32, n: u32, ) -> Weight{
        0
    }
	fn seal_set_rent_allowance(r: u32, ) -> Weight{
        0
    }
	fn seal_set_storage(r: u32, ) -> Weight{
        0
    }
	fn seal_set_storage_per_kb(n: u32, ) -> Weight{
        0
    }
	fn seal_clear_storage(r: u32, ) -> Weight{
        0
    }
	fn seal_get_storage(r: u32, ) -> Weight{
        0
    }
	fn seal_get_storage_per_kb(n: u32, ) -> Weight{
        0
    }
	fn seal_transfer(r: u32, ) -> Weight{
        0
    }
	fn seal_call(r: u32, ) -> Weight{
        0
    }
	fn seal_call_per_code_transfer_input_output_kb(c: u32, t: u32, i: u32, o: u32, ) -> Weight{
        0
    }
	fn seal_instantiate(r: u32, ) -> Weight{
        0
    }
	fn seal_instantiate_per_code_input_output_salt_kb(c: u32, i: u32, o: u32, s: u32, ) -> Weight{
        0
    }
	fn seal_hash_sha2_256(r: u32, ) -> Weight{
        0
    }
	fn seal_hash_sha2_256_per_kb(n: u32, ) -> Weight{
        0
    }
	fn seal_hash_keccak_256(r: u32, ) -> Weight{0}
	fn seal_hash_keccak_256_per_kb(n: u32, ) -> Weight{0}
	fn seal_hash_blake2_256(r: u32, ) -> Weight{0}
	fn seal_hash_blake2_256_per_kb(n: u32, ) -> Weight{0}
	fn seal_hash_blake2_128(r: u32, ) -> Weight{0}
	fn seal_hash_blake2_128_per_kb(n: u32, ) -> Weight{0}
	fn instr_i64const(r: u32, ) -> Weight{0}
	fn instr_i64load(r: u32, ) -> Weight{0}
	fn instr_i64store(r: u32, ) -> Weight{0}
	fn instr_select(r: u32, ) -> Weight{0}
	fn instr_if(r: u32, ) -> Weight{0}
	fn instr_br(r: u32, ) -> Weight{0}
	fn instr_br_if(r: u32, ) -> Weight{0}
	fn instr_br_table(r: u32, ) -> Weight{0}
	fn instr_br_table_per_entry(e: u32, ) -> Weight{0}
	fn instr_call(r: u32, ) -> Weight{0}
	fn instr_call_indirect(r: u32, ) -> Weight{0}
	fn instr_call_indirect_per_param(p: u32, ) -> Weight{0}
	fn instr_local_get(r: u32, ) -> Weight{0}
	fn instr_local_set(r: u32, ) -> Weight{0}
	fn instr_local_tee(r: u32, ) -> Weight{0}
	fn instr_global_get(r: u32, ) -> Weight{0}
	fn instr_global_set(r: u32, ) -> Weight{0}
	fn instr_memory_current(r: u32, ) -> Weight{0}
	fn instr_memory_grow(r: u32, ) -> Weight{0}
	fn instr_i64clz(r: u32, ) -> Weight{0}
	fn instr_i64ctz(r: u32, ) -> Weight{0}
	fn instr_i64popcnt(r: u32, ) -> Weight{0}
	fn instr_i64eqz(r: u32, ) -> Weight{0}
	fn instr_i64extendsi32(r: u32, ) -> Weight{0}
	fn instr_i64extendui32(r: u32, ) -> Weight{0}
	fn instr_i32wrapi64(r: u32, ) -> Weight{0}
	fn instr_i64eq(r: u32, ) -> Weight{0}
	fn instr_i64ne(r: u32, ) -> Weight{0}
	fn instr_i64lts(r: u32, ) -> Weight{0}
	fn instr_i64ltu(r: u32, ) -> Weight{0}
	fn instr_i64gts(r: u32, ) -> Weight{0}
	fn instr_i64gtu(r: u32, ) -> Weight{0}
	fn instr_i64les(r: u32, ) -> Weight{0}
	fn instr_i64leu(r: u32, ) -> Weight{0}
	fn instr_i64ges(r: u32, ) -> Weight{0}
	fn instr_i64geu(r: u32, ) -> Weight{0}
	fn instr_i64add(r: u32, ) -> Weight{0}
	fn instr_i64sub(r: u32, ) -> Weight{0}
	fn instr_i64mul(r: u32, ) -> Weight{0}
	fn instr_i64divs(r: u32, ) -> Weight{0}
	fn instr_i64divu(r: u32, ) -> Weight{0}
	fn instr_i64rems(r: u32, ) -> Weight{0}
	fn instr_i64remu(r: u32, ) -> Weight{0}
	fn instr_i64and(r: u32, ) -> Weight{0}
	fn instr_i64or(r: u32, ) -> Weight{0}
	fn instr_i64xor(r: u32, ) -> Weight{0}
	fn instr_i64shl(r: u32, ) -> Weight{0}
	fn instr_i64shrs(r: u32, ) -> Weight{0}
	fn instr_i64shru(r: u32, ) -> Weight{0}
	fn instr_i64rotl(r: u32, ) -> Weight{0}
	fn instr_i64rotr(r: u32, ) -> Weight{0}
}