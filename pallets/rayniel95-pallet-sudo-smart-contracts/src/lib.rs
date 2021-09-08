#![cfg_attr(not(feature = "std"), no_std)]

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {
    use frame_support::{dispatch::DispatchResultWithPostInfo, pallet_prelude::*};
    use frame_system::pallet_prelude::*;
    // use sp_std::vec::Vec; // Step 3.1 will include this in `Cargo.toml`
	
    // NOTE - it is neccesary to access to a contracts instantia from the 
    // wrapper?
	// NOTE - it is necessary to inherit from sudo config and contracts config?
    #[pallet::config]  // <-- Step 2. code block will replace this.
	pub trait Config: frame_system::Config {
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
    
    // #[pallet::hooks]
    // impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}
    
    #[pallet::call]   // <-- Step 6. code block will replace this.

}