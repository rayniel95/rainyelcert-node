#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

use sp_core::crypto::UncheckedFrom;

use sp_runtime::traits::StaticLookup;
use frame_support::traits::Currency;
use pallet_contracts::{Schedule};

type CodeHash<T> = <T as frame_system::Config>::Hash;

type BalanceOf<T> =
	<<T as pallet_contracts::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{
		dispatch::{
			DispatchResultWithPostInfo,
			DispatchErrorWithPostInfo
		}, 
		pallet_prelude::*,
		weights::PostDispatchInfo
	};
    use frame_system::pallet_prelude::*;
	use frame_system::RawOrigin;
    use sp_std::vec::Vec; // Step 3.1 will include this in `Cargo.toml`
	use super::*;

	#[pallet::config]  // <-- Step 2. code block will replace this.
	pub trait Config: 
		frame_system::Config + pallet_contracts::Config + pallet_sudo::Config {
		// type Currency: Currency<Self::AccountId>;
    }
    // #[pallet::event]   // <-- Step 3. code block will replace this.
	
    #[pallet::error]   // <-- Step 4. code block will replace this.
    pub enum Error<T> {
		/// The origin is not the root origin
		NotRootOrigin,
	}

    #[pallet::pallet]
    #[pallet::generate_store(pub(super) trait Store)]
    pub struct Pallet<T>(_);
    
    // #[pallet::storage] // <-- Step 5. code block will replace this.
    
    #[pallet::hooks]
    impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
    
    #[pallet::call]   // <-- Step 6. code block will replace this.
    impl<T: Config> Pallet<T>
	where
		T::AccountId: UncheckedFrom<T::Hash>,
		T::AccountId: AsRef<[u8]>,
	{
		/// Makes a call to an account, optionally transferring some balance.
		///
		/// * If the account is a smart-contract account, the associated code will be
		/// executed and any value will be transferred.
		/// * If the account is a regular account, any value will be transferred.
		/// * If no account exists and the call value is not less than `existential_deposit`,
		/// a regular account will be created and any value will be transferred.
		#[pallet::weight(0)]
		pub fn call(
			origin: OriginFor<T>,
			dest: <T::Lookup as StaticLookup>::Source,
			#[pallet::compact] value: BalanceOf<T>,
			#[pallet::compact] gas_limit: Weight,
			data: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			// TODO - test different call syntax to check wich consume gas fees
            let result = pallet_contracts::Pallet::<T>::call(
                origin, dest, value, gas_limit, data
            );	
			result
				.map(
					|mut post_info| {
						post_info.actual_weight = None;
						post_info.pays_fee = Pays::No;
						post_info
					}
				)
				.map_err(
					|mut e| {
						e.post_info.actual_weight = None;
						e.post_info.pays_fee = Pays::No;
						e
					}
				)
		}

		/// Instantiates a new contract from the supplied `code` optionally transferring
		/// some balance.
		///
		/// This is the only function that can deploy new code to the chain.
		///
		/// # Parameters
		///
		/// * `endowment`: The balance to transfer from the `origin` to the newly created contract.
		/// * `gas_limit`: The gas limit enforced when executing the constructor.
		/// * `code`: The contract code to deploy in raw bytes.
		/// * `data`: The input data to pass to the contract constructor.
		/// * `salt`: Used for the address derivation. See [`Pallet::contract_address`].
		///
		/// Instantiation is executed as follows:
		///
		/// - The supplied `code` is instrumented, deployed, and a `code_hash` is created for that
		///   code.
		/// - If the `code_hash` already exists on the chain the underlying `code` will be shared.
		/// - The destination address is computed based on the sender, code_hash and the salt.
		/// - The smart-contract account is created at the computed address.
		/// - The `endowment` is transferred to the new account.
		/// - The `deploy` function is executed in the context of the newly-created account.
		#[pallet::weight(0)] // TODO - put here Pays::No and DispatchClass::Normal
		pub fn instantiate_with_code(
			origin: OriginFor<T>,
			#[pallet::compact] endowment: BalanceOf<T>,
			#[pallet::compact] gas_limit: Weight,
			code: Vec<u8>,
			data: Vec<u8>,
			salt: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			Self::is_root(origin.clone())?;
            let result = pallet_contracts::Pallet::<T>::instantiate_with_code(
                origin, endowment, gas_limit, code, data, salt
            );
			result
				.map(
					|mut post_info| {
						post_info.actual_weight = None;
						post_info.pays_fee = Pays::No;
						post_info
					}
				)
				.map_err(
					|mut e| {
						e.post_info.actual_weight = None;
						e.post_info.pays_fee = Pays::No;
						e
					}
				)
				// FIXME - For some reason if I use Ok(Pays::No.into()) I do not need to pay
				// fee but event are not launched
		}
		/// Updates the schedule for metering contracts.
		///
		/// The schedule's version cannot be less than the version of the stored schedule.
		/// If a schedule does not change the instruction weights the version does not
		/// need to be increased. Therefore we allow storing a schedule that has the same
		/// version as the stored one.
		#[pallet::weight(0)]
		pub fn update_schedule(
			origin: OriginFor<T>,
			schedule: Schedule<T>
		) -> DispatchResultWithPostInfo {
			Self::is_root(origin.clone())?;
			pallet_contracts::Pallet::update_schedule(origin, schedule)
		}

		/// Instantiates a contract from a previously deployed wasm binary.
		///
		/// This function is identical to [`Self::instantiate_with_code`] but without the
		/// code deployment step. Instead, the `code_hash` of an on-chain deployed wasm binary
		/// must be supplied.
		#[pallet::weight(0)]
		pub fn instantiate(
			origin: OriginFor<T>,
			#[pallet::compact] endowment: BalanceOf<T>,
			#[pallet::compact] gas_limit: Weight,
			code_hash: CodeHash<T>,
			data: Vec<u8>,
			salt: Vec<u8>,
		) -> DispatchResultWithPostInfo {
			Self::is_root(origin.clone())?;
			let result = pallet_contracts::Pallet::<T>::instantiate(
				origin, endowment, gas_limit, code_hash, data, salt
			);
			result
				.map(
					|mut post_info| {
						post_info.actual_weight = None;
						post_info.pays_fee = Pays::No;
						post_info
					}
				)
				.map_err(
					|mut e| {
						e.post_info.actual_weight = None;
						e.post_info.pays_fee = Pays::No;
						e
					}
				)
		}
		/// Allows block producers to claim a small reward for evicting a contract. If a block
		/// producer fails to do so, a regular users will be allowed to claim the reward.
		///
		/// In case of a successful eviction no fees are charged from the sender. However, the
		/// reward is capped by the total amount of rent that was payed by the contract while
		/// it was alive.
		///
		/// If contract is not evicted as a result of this call, [`Error::ContractNotEvictable`]
		/// is returned and the sender is not eligible for the reward.
		#[pallet::weight(0)]
		pub fn claim_surcharge(
			origin: OriginFor<T>,
			dest: T::AccountId,
			aux_sender: Option<T::AccountId>
		) -> DispatchResultWithPostInfo {
			Self::is_root(origin.clone())?;
			pallet_contracts::Pallet::<T>::claim_surcharge(
				origin, dest, aux_sender
			)
		}
	}

	impl<T: Config> Pallet<T>
	where
		T::AccountId: UncheckedFrom<T::Hash>,
		T::AccountId: AsRef<[u8]>,
	{
		fn is_root(origin: OriginFor<T>) -> Result<(), DispatchError>{
			let sudo_key = pallet_sudo::Pallet::<T>::key();
			let test_origin = match origin.into() {
				Ok(RawOrigin::Signed(account_id)) => {
					let mut result = Ok(());
					if sudo_key != account_id{
						result = Err(DispatchError::BadOrigin);
					}
					result
				}
				Ok(RawOrigin::Root) => Ok(()),
				_ => Err(DispatchError::BadOrigin),
			};
			test_origin
		}	
	}
}
