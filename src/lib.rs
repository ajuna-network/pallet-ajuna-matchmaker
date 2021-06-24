#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

pub use pallet::*;

use codec::{Decode, Encode};

#[cfg(test)]
mod mock;

#[cfg(test)]
mod tests;

#[cfg(feature = "runtime-benchmarks")]
mod benchmarking;

mod ringbuffer;

use ringbuffer::{RingBufferTrait, RingBufferTransient, BufferIndex};

#[derive(Encode, Decode, Default, Clone, PartialEq, Eq)]
#[cfg_attr(feature = "std", derive(Debug))]
pub struct PlayerStruct<AccountId> {
	ranking: u32,
	account: AccountId,
}

#[frame_support::pallet]
pub mod pallet {
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	// important to use outside structs and consts
	use super::*;



	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;
	}

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// The pallet's runtime storage items.
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage
	#[pallet::storage]
	#[pallet::getter(fn something)]
	// Learn more about declaring storage items:
	// https://substrate.dev/docs/en/knowledgebase/runtime/storage#declaring-storage-items
	pub type Something<T> = StorageValue<_, u32>;

	#[pallet::storage]
	#[pallet::getter(fn get_value)]
	pub type BufferMap<T: Config> = StorageMap<_, Twox64Concat, BufferIndex , PlayerStruct<T::AccountId>, ValueQuery>;

	// Default value for Nonce
	#[pallet::type_value]
	pub fn BufferRangeDefault<T: Config>() -> (BufferIndex, BufferIndex) { (0, 0) }
	#[pallet::storage]
	#[pallet::getter(fn range)]
	pub type BufferRange<T: Config> = StorageValue<_, (BufferIndex, BufferIndex), ValueQuery, BufferRangeDefault<T>>;

	// Pallets use events to inform users when important changes are made.
	// https://substrate.dev/docs/en/knowledgebase/runtime/events
	#[pallet::event]
	#[pallet::metadata(T::AccountId = "AccountId")]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		/// Queued event
		Queued(PlayerStruct<T::AccountId>),
		/// Popped event
		Popped(PlayerStruct<T::AccountId>),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {}

	// Dispatchable functions allows users to interact with the pallet and invoke state changes.
	// These functions materialize as "extrinsics", which are often compared to transactions.
	// Dispatchable functions must be annotated with a weight and must return a DispatchResult.
	#[pallet::call]
	impl<T:Config> Pallet<T> {
		/// An example dispatchable that takes a singles value as a parameter, writes the value to
		/// storage and emits an event. This function must be dispatched by a signed extrinsic.
		#[pallet::weight(10_000 + T::DbWeight::get().writes(1))]
		pub fn do_something(origin: OriginFor<T>, something: u32) -> DispatchResult {
			// Check that the extrinsic was signed and get the signer.
			// This function will return an error if the extrinsic is not signed.
			// https://substrate.dev/docs/en/knowledgebase/runtime/origin
			let who = ensure_signed(origin)?;

			// Update storage.
			<Something<T>>::put(something);

			// Emit an event.
			Self::deposit_event(Event::SomethingStored(something, who));
			// Return a successful DispatchResultWithPostInfo
			Ok(())
		}

		/// An example dispatchable that may throw a custom error.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn cause_error(origin: OriginFor<T>) -> DispatchResult {
			let _who = ensure_signed(origin)?;

			// Read a value from storage.
			match <Something<T>>::get() {
				// Return an error if the value has not been set.
				None => Err(Error::<T>::NoneValue)?,
				Some(old) => {
					// Increment the value read from storage; will error in the event of overflow.
					let new = old.checked_add(1).ok_or(Error::<T>::StorageOverflow)?;
					// Update the value in storage with the incremented result.
					<Something<T>>::put(new);
					Ok(())
				},
			}
		}

		/// Add an item to the queue
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn add_to_queue(origin: OriginFor<T>, ranking: u32, account: T::AccountId) -> DispatchResult {
			// only a user can push into the queue
			let _user = ensure_signed(origin)?;
	
			let mut queue = Self::queue_transient();
			let player = PlayerStruct{ ranking, account };
			queue.push(player.clone());
		
			Self::deposit_event(Event::Queued(player));	

			Ok(())
		}

		/// Add several items to the queue
		//#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		//pub fn add_multiple(origin: OriginFor<T>, rankings: Vec<i32>, boolean: bool) -> DispatchResult {
		//	// only a user can push into the queue
		//	let _user = ensure_signed(origin)?;
		//
		//	let mut queue = Self::queue_transient();
		//	for ranking in rankings {
		//		queue.push(PlayerStruct{ ranking, boolean });
		//	}
		//
		//	Ok(())
		//}
		
		/// Remove and return an item from the queue
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn pop_from_queue(origin: OriginFor<T>) -> DispatchResult {
			// only a user can pop from the queue
			let _user = ensure_signed(origin)?;		

			let mut queue = Self::queue_transient();
			if let Some(player_struct) = queue.pop() {
				Self::deposit_event(Event::Popped(player_struct));	
			}
		
			Ok(())	
		}

		/// Try to create a match with a certain amount of players.
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		pub fn make_match(origin: OriginFor<T>) -> DispatchResult {
			// only a user can pop from the queue
			let _user = ensure_signed(origin)?;	
						
			let mut queue = Self::queue_transient();
			ensure!(queue.min_size_reached(1), "There aren't enough players queued currently.");
			
			let mut player_array: [PlayerStruct<T::AccountId>; 2] = Default::default();

			for p in 0..2 {
				if let Some(player_struct) = queue.pop() {
					player_array[p] = player_struct.clone();
					Self::deposit_event(Event::Popped(player_struct));	
				}
				else
				{
					return Err(Error::<T>::NoneValue)?
				}
			}

			Ok(())	
		}
	}
}

impl<T: Config> Pallet<T> {
	/// Constructor function so we don't have to specify the types every time.
	///
	/// Constructs a ringbuffer transient and returns it as a boxed trait object.
	/// See [this part of the Rust book](https://doc.rust-lang.org/book/ch17-02-trait-objects.html#trait-objects-perform-dynamic-dispatch)
	fn queue_transient() -> Box<dyn RingBufferTrait<PlayerStruct<T::AccountId>>> {
		Box::new(RingBufferTransient::<
			PlayerStruct<T::AccountId>,
			<Self as Store>::BufferRange,
			<Self as Store>::BufferMap,
		>::new())
	}

	pub fn test() -> bool {
		true
	}
}