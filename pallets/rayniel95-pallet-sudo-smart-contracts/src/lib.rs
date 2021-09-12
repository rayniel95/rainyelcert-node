#![cfg_attr(not(feature = "std"), no_std)]

#[cfg(test)]
mod tests;

use sp_core::crypto::UncheckedFrom;

use sp_runtime::traits::StaticLookup;
use frame_support::traits::Currency;

type BalanceOf<T> =
	<<T as pallet_contracts::Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    use sp_std::vec::Vec; // Step 3.1 will include this in `Cargo.toml`
	use super::*;
    // NOTE - it is neccesary to access to a contracts instantia from the 
    // wrapper?
	// NOTE - it is necessary to inherit from sudo config and contracts config?
    #[pallet::config]  // <-- Step 2. code block will replace this.
	pub trait Config: frame_system::Config + pallet_contracts::Config{
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
            pallet_contracts::Pallet::<T>::call(
                origin, dest, value, gas_limit, data
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
		#[pallet::weight(0)]
		pub fn instantiate_with_code(
			origin: OriginFor<T>,
			#[pallet::compact] endowment: BalanceOf<T>,
			#[pallet::compact] gas_limit: Weight,
			code: Vec<u8>,
			data: Vec<u8>,
			salt: Vec<u8>,
		) -> DispatchResultWithPostInfo {
            pallet_contracts::Pallet::<T>::instantiate_with_code(
                origin, endowment, gas_limit, code, data, salt
            )
		}
	}
}