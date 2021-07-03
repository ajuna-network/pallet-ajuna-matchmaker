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

mod brackets;

use brackets::{BracketsTrait, BracketsTransient, BufferIndex, Bracket};

const AMOUNT_PLAYERS: u8 = 2;

#[derive(Encode, Decode, Clone, PartialEq)]
pub enum MatchingType {
	// ranked matches, if no one in bracket drop down
	Simple,
	// only allow same bracket matches
	Same,
	// take only one of one bracket
	Mix,
}

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

		/// Constant that indicates how many players are need to create a new match.
		#[pallet::constant]
		type AmountPlayers: Get<u8>;

		/// Constant that indicates how many ranking brackets exist for players.
		#[pallet::constant]
		type AmountBrackets: Get<u8>;
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

	#[pallet::type_value]
	pub fn BracketsCountDefault<T: Config>() -> u8 { T::AmountBrackets::get() }
	#[pallet::storage]
	#[pallet::getter(fn brackets_count)]
	pub type BracketsCount<T> = StorageValue<_, u8, ValueQuery, BracketsCountDefault<T>>;

	// Default value for Nonce
	#[pallet::type_value]
	pub fn BracketIndicesDefault<T: Config>() -> (BufferIndex, BufferIndex) { (0, 0) }
	#[pallet::storage]
	#[pallet::getter(fn indices)]
	pub type BracketIndices<T: Config> = StorageMap<_, Blake2_128Concat, Bracket, (BufferIndex, BufferIndex), ValueQuery, BracketIndicesDefault<T>>;

	#[pallet::storage]
	#[pallet::getter(fn index_key)]
	pub type BracketIndexKeyMap<T: Config> = StorageDoubleMap<_, Blake2_128Concat, Bracket, Blake2_128Concat, BufferIndex, T::AccountId, ValueQuery>;

	#[pallet::storage]
	#[pallet::getter(fn key_value)]
	pub type BracketKeyValueMap<T: Config> = StorageDoubleMap<_, Blake2_128Concat, Bracket, Blake2_128Concat, T::AccountId, PlayerStruct<T::AccountId>, ValueQuery>;

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
	}
}

impl<T: Config> Pallet<T> {

	/// Constructor function so we don't have to specify the types every time.
	///
	/// Constructs a ringbuffer transient and returns it as a boxed trait object.
	/// See [this part of the Rust book](https://doc.rust-lang.org/book/ch17-02-trait-objects.html#trait-objects-perform-dynamic-dispatch)
	fn queue_transient() -> Box<dyn BracketsTrait<T::AccountId,PlayerStruct<T::AccountId>>> {
		Box::new(BracketsTransient::<
			T::AccountId,
			PlayerStruct<T::AccountId>,
			<Self as Store>::BracketsCount,
			<Self as Store>::BracketIndices,
			<Self as Store>::BracketIndexKeyMap,
			<Self as Store>::BracketKeyValueMap,
		>::new())
	}
	
	fn do_add_queue(account: T::AccountId, bracket: u8) -> bool {
		let mut queue = Self::queue_transient();

		let player = PlayerStruct{ account };
		// duplicate check if we can add key to the queue
		if !queue.push(bracket, player.account.clone(), player.clone()) {
			return false
		}

		Self::deposit_event(Event::Queued(player));	
		true
	}

	fn do_empty_queue(bracket: u8) {
		let mut queue = Self::queue_transient();

		while queue.size(bracket) > 0 {
			queue.pop(bracket);
		}
	}

	fn do_all_empty_queue() {

		for i in 0..Self::brackets_count() {
			Self::do_empty_queue(i);
		}
	}

	fn do_try_match() -> Option<[T::AccountId; AMOUNT_PLAYERS as usize]> {	
		let mut queue = Self::queue_transient();

		 let mut brackets: Vec<Bracket> = Vec::new();
		// pass trough all brackets
		for i in 0..Self::brackets_count() {
			// skip if bracket is empty
			if queue.size(i) == 0 {
				continue;
			} 
			// iterate for each slot occupied and fill, till player match size reached
			for _j in 0..queue.size(i) {
				if brackets.len() == AMOUNT_PLAYERS as usize {
					break;
				}
				brackets.push(i);
			}
			// leave if brackets is filled with brackets
			if brackets.len() == AMOUNT_PLAYERS as usize {
				break;
			}
		}
		// vec not filled with enough brackets leave
		if brackets.len() < AMOUNT_PLAYERS as usize {
			return None;
		}
		// pop from the harvested brackets players
		let mut accounts: [T::AccountId; AMOUNT_PLAYERS as usize] = Default::default();
		for i in 0..brackets.len() {
			if let Some(p) = queue.pop(brackets[i]) {
				accounts[i] = p.account.clone();
				Self::deposit_event(Event::Popped(p));	
			}
		}
		// return result
		Some(accounts)
	}

	fn do_is_queued(account: T::AccountId) -> bool {

		Self::queue_transient().is_queued(account)
	}

	fn do_queue_size(bracket: u8) -> BufferIndex {

		Self::queue_transient().size(bracket)
	}

	fn do_all_queue_size() -> BufferIndex {
		let queue = Self::queue_transient();

		let mut total_queued: BufferIndex = 0;
		// count all existing brackets
		for i in 0..Self::brackets_count() {
			total_queued = total_queued + queue.size(i);
		}
		// return result
		total_queued
	}
}

impl<T: Config> MatchFunc<T::AccountId> for Pallet<T> {

	fn empty_queue(bracket: u8) {

		Self::do_empty_queue(bracket);
	}

	fn all_empty_queue() {

		Self::do_all_empty_queue();
	}

	fn add_queue(account: T::AccountId, bracket: u8) -> bool {

		Self::do_add_queue(account, bracket)
	}

	fn try_match() -> Option<[T::AccountId; AMOUNT_PLAYERS as usize]> {
		
		Self::do_try_match()
	}

	fn is_queued(account: T::AccountId) -> bool {
		
		Self::do_is_queued(account)
	}

	fn queue_size(bracket: u8) -> BufferIndex {
		
		Self::do_queue_size(bracket)
	}

	fn all_queue_size() -> BufferIndex {

		Self::do_all_queue_size()
	}
}

pub trait MatchFunc<AccountId> {

	/// empty specific bracket queue
	fn empty_queue(bracket: u8);

	/// empty all queues
	fn all_empty_queue();

	/// return true if adding account to bracket queue was successful
	fn add_queue(account: AccountId, bracket: u8) -> bool;

	/// try create a match
	fn try_match() -> Option<[AccountId; AMOUNT_PLAYERS as usize]>;

	// return true if an account is queued in any bracket
	fn is_queued(account: AccountId) -> bool;

	// return size of a specific bracket queue
	fn queue_size(bracket: u8) -> BufferIndex;

	// return total size of all queued accounts in all brackets
	fn all_queue_size() -> BufferIndex;
}
