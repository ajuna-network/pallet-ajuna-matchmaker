#![cfg_attr(not(feature = "std"), no_std)]

/// Edit this file to define custom logic or remove it if it is not needed.
/// Learn more about FRAME and the core library of Substrate FRAME pallets:
/// <https://substrate.dev/docs/en/knowledgebase/runtime/frame>

pub use pallet::*;

use codec::{Decode, Encode};
use sp_std::boxed::{
	Box
};

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
	pub type BufferMap<T: Config> = StorageMap<_, Twox64Concat, BufferIndex, PlayerStruct<T::AccountId>, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn get_list)]
	pub type BufferList<T: Config> = StorageMap<_, Twox64Concat, T::AccountId, BufferIndex, ValueQuery>;

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
		/// Queue size is to low.
		QueueSizeToLow,
		/// Queue is empty.
		QueueIsEmpty,
	}

	#[pallet::hooks]
	impl<T: Config> Hooks<BlockNumberFor<T>> for Pallet<T> {
		// `on_initialize` is executed at the beginning of the block before any extrinsic are
		// dispatched.
		//
		// This function must return the weight consumed by `on_initialize` and `on_finalize`.
		fn on_initialize(_: T::BlockNumber) -> Weight {
			// Anything that needs to be done at the start of the block.
			// We don't do anything here.
			0
		}
	}

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
		pub fn add_to_queue(origin: OriginFor<T>, account: T::AccountId) -> DispatchResult {
			// only a user can push into the queue
			let _user = ensure_root(origin)?;
			let mut queue = Self::queue_transient();
			let player = PlayerStruct{ account };
			queue.push(player.account.clone(), player.clone());
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
			let _user = ensure_root(origin)?;		

			let mut queue = Self::queue_transient();
			if let Some(player_struct) = queue.pop() {
				Self::deposit_event(Event::Popped(player_struct));	
			}
		
			Ok(())	
		}

		// /// Remove and return an item from the queue
		//#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		//pub fn try_match(origin: OriginFor<T>) -> DispatchResult {
		//	// only a user can pop from the queue
		//	let _sender = ensure_root(origin)?;		
		//	let result = Self::do_try_match();
		//	Ok(())	
		//}
	}
}

impl<T: Config> Pallet<T> {
	/// Constructor function so we don't have to specify the types every time.
	///
	/// Constructs a ringbuffer transient and returns it as a boxed trait object.
	/// See [this part of the Rust book](https://doc.rust-lang.org/book/ch17-02-trait-objects.html#trait-objects-perform-dynamic-dispatch)
	fn queue_transient() -> Box<dyn RingBufferTrait<T::AccountId,PlayerStruct<T::AccountId>>> {
		Box::new(RingBufferTransient::<
			T::AccountId,
			PlayerStruct<T::AccountId>,
			<Self as Store>::BufferRange,
			<Self as Store>::BufferMap,
			<Self as Store>::BufferList,
		>::new())
	}
	
	fn do_add_queue(account: T::AccountId) -> bool {
		
		let mut queue = Self::queue_transient();

		let player = PlayerStruct{ account };
		queue.push(player.account.clone(), player.clone());

		Self::deposit_event(Event::Queued(player));	

		// do duplicate check for false later
		true
	}

	fn do_empty_queue() {

		let mut queue = Self::queue_transient();

		while queue.size() > 0 {
			queue.pop();
		}
	}

	fn do_try_match() -> Option<[T::AccountId; 2]> {
		
		let mut queue = Self::queue_transient();

		if queue.size() < 2 {
			return None;
		}

		let mut accounts: [T::AccountId; 2] = Default::default();

		if let Some(player1) = queue.pop() {
			accounts[0] = player1.account.clone();
			Self::deposit_event(Event::Popped(player1));	
		}

		if let Some(player2) = queue.pop() {
			accounts[1] = player2.account.clone();
			Self::deposit_event(Event::Popped(player2));	
		}

		Some(accounts)
	}

	fn do_is_queued(account: T::AccountId) -> bool {

		Self::queue_transient().is_queued(account)
	}
}

impl<T: Config> MatchFunc<T::AccountId> for Pallet<T> {

	fn empty_queue() {

		Self::do_empty_queue();
	}

	fn add_queue(account: T::AccountId) -> bool {

		Self::do_add_queue(account)
	}

	fn try_match() -> Option<[T::AccountId; 2]> {
		
		Self::do_try_match()
	}

	fn is_queued(account: T::AccountId) -> bool {
		
		Self::do_is_queued(account)
	}
}

pub trait MatchFunc<AccountId> {

	fn empty_queue();

	fn add_queue(account: AccountId) -> bool;

	fn try_match() -> Option<[AccountId; 2]>;

	fn is_queued(account: AccountId) -> bool;
}
