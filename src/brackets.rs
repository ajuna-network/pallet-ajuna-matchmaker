//! # Matchmaker Brackets (based on Transient RingBuffer implementation)
//!
//! This pallet provides a trait and implementation for a brackets ranked system that
//! abstracts over storage items and presents them as multiple brackets, with each
//! having a FIFO queue. This allows an implementation of a matchmaker over different
//! ranking brackets.
//!
//! Usage Example:
//! ```rust, ignore
//! use bracketqueues::{BracketsTrait, BracketsTransient};
//!
//! // Trait object that we will be interacting with.
//! type Brackets = dyn BracketsTrait<SomeStruct>;
//! // Implementation that we will instantiate.
//! type Transient = BracketsTransient<
//!     SomeStruct,
//!     <TestModule as Store>::TestBracketIndices,
//!     <TestModule as Store>::TestBracketIndexKeyMap,
//! >;
//! {
//!     let mut ring: Box<Brackets> = Box::new(Transient::new());
//!     ring.push(SomeStruct { foo: 1, bar: 2 });
//! } // `ring.commit()` will be called on `drop` here and syncs to storage
//! ```
//!
//! Note: You might want to introduce a helper function that wraps the complex
//! types and just returns the boxed trait object.
use codec::{Codec, EncodeLike};
use core::marker::PhantomData;
use frame_support::storage::{StorageDoubleMap, StorageMap, StorageValue};
use sp_std::vec::{Vec};

/// Trait object presenting the brackets interface.
pub trait BracketsTrait<ItemKey, Item>
where
	ItemKey: Codec + EncodeLike,
	Item: Codec + EncodeLike,
{
	/// Store all changes made in the underlying storage.
	///
	/// Data is not guaranteed to be consistent before this call.
	///
	/// Implementation note: Call in `drop` to increase ergonomics.
	fn commit(&self);
	/// Push an item onto the end of the queue.
	fn push(&mut self, b: QueueCluster, j: ItemKey, i: Item) -> bool;
	/// Pop an item from the start of the queue.
	///
	/// Returns `None` if the queue is empty.
	fn pop(&mut self, b: QueueCluster) -> Option<Item>;
	/// Return whether the queue is empty.
	fn is_empty(&self, b: QueueCluster) -> bool;
    /// Return the size of the brackets queue.
    fn size(&self, b: QueueCluster) -> BufferIndex;
	/// Return whether the item_key is queued or not.
	fn is_queued(&self, j: ItemKey) -> bool;
}

// There is no equivalent trait in std so we create one.
pub trait WrappingOps {
	fn wrapping_add(self, rhs: Self) -> Self;
	fn wrapping_sub(self, rhs: Self) -> Self;
}

macro_rules! impl_wrapping_ops {
	($type:ty) => {
		impl WrappingOps for $type {
			fn wrapping_add(self, rhs: Self) -> Self {
				self.wrapping_add(rhs)
			}
			fn wrapping_sub(self, rhs: Self) -> Self {
				self.wrapping_sub(rhs)
			}
		}
	};
}

impl_wrapping_ops!(u8);
impl_wrapping_ops!(u16);
impl_wrapping_ops!(u32);
impl_wrapping_ops!(u64);

pub type BufferIndex = u16;
pub type BufferIndexVector = Vec<(BufferIndex, BufferIndex)>;
pub type QueueCluster = u8;

/// Transient backing data that is the backbone of the trait object.
pub struct BracketsTransient<ItemKey, Item, C, B, M, N>
where
	ItemKey: Codec + EncodeLike,
	Item: Codec + EncodeLike,
	C: StorageValue<QueueCluster, Query = QueueCluster>,
	B: StorageMap<QueueCluster, (BufferIndex, BufferIndex), Query = (BufferIndex, BufferIndex)>,
	M: StorageDoubleMap<QueueCluster, BufferIndex, ItemKey, Query = ItemKey>,
	N: StorageDoubleMap<QueueCluster, ItemKey, Item, Query = Item>,
{
	index_vector: BufferIndexVector,
	_phantom: PhantomData<(ItemKey, Item, C, B, M, N)>,
}

impl<ItemKey, Item, C, B, M, N> BracketsTransient<ItemKey, Item, C, B, M, N>
where
	ItemKey: Codec + EncodeLike,
	Item: Codec + EncodeLike,
	C: StorageValue<QueueCluster, Query = QueueCluster>,
	B: StorageMap<QueueCluster, (BufferIndex, BufferIndex), Query = (BufferIndex, BufferIndex)>,
	M: StorageDoubleMap<QueueCluster, BufferIndex, ItemKey, Query = ItemKey>,
	N: StorageDoubleMap<QueueCluster, ItemKey, Item, Query = Item>,
{
	/// Create a new `BracketsTransient` that backs the brackets implementation.
	///
	/// Initializes itself from the bounds storage `B`.
	pub fn new() -> BracketsTransient<ItemKey, Item, C, B, M, N> {

		// get brackets count
		let brackets_count = C::get();

		// initialize all brackets
		let mut index_vector = Vec::new();
		for i in 0..brackets_count {
			let (start, end) = B::get(i);
			index_vector.push((start, end));
		}

		BracketsTransient {
			index_vector,
			_phantom: PhantomData,
		}
	}
}

impl<ItemKey, Item, C, B, M, N> Drop for BracketsTransient<ItemKey, Item, C, B, M, N>
where
	ItemKey: Codec + EncodeLike,
	Item: Codec + EncodeLike,
	C: StorageValue<QueueCluster, Query = QueueCluster>,
	B: StorageMap<QueueCluster, (BufferIndex, BufferIndex), Query = (BufferIndex, BufferIndex)>,
	M: StorageDoubleMap<QueueCluster, BufferIndex, ItemKey, Query = ItemKey>,
	N: StorageDoubleMap<QueueCluster, ItemKey, Item, Query = Item>,
{
	/// Commit on `drop`.
	fn drop(&mut self) {
		<Self as BracketsTrait<ItemKey, Item>>::commit(self);
	}
}

/// Brackets implementation based on `BracketsTransient`
impl<ItemKey, Item, C, B, M, N> BracketsTrait<ItemKey, Item> for BracketsTransient<ItemKey, Item, C, B, M, N>
where
	ItemKey: Codec + EncodeLike,
	Item: Codec + EncodeLike,
	C: StorageValue<QueueCluster, Query = QueueCluster>,
	B: StorageMap<QueueCluster, (BufferIndex, BufferIndex), Query = (BufferIndex, BufferIndex)>,
	M: StorageDoubleMap<QueueCluster, BufferIndex, ItemKey, Query = ItemKey>,
	N: StorageDoubleMap<QueueCluster, ItemKey, Item, Query = Item>,
{
	/// Commit the (potentially) changed bounds to storage.
	fn commit(&self) {

		// commit indicies on all brackets
		for i in 0..self.index_vector.len() {
			let (v_start, v_end) = self.index_vector[i];
			B::insert(i as QueueCluster, (v_start, v_end));
		}
	}

	/// Push an item onto the end of the queue.
	///
	/// Will insert the new item, but will not update the bounds in storage.
	fn push(&mut self, bracket: QueueCluster, item_key: ItemKey, item: Item) -> bool {
		
		let (mut v_start, mut v_end) = self.index_vector[bracket as usize];

		// check if there is already such a key queued
		if N::contains_key(bracket, &item_key) {
			return false
		}

		// insert the item key and the item
		N::insert(bracket, &item_key, item);
		M::insert(bracket, v_end, item_key);

		// this will intentionally overflow and wrap around when bonds_end
		// reaches `Index::max_value` because we want a brackets.
		let next_index = v_end.wrapping_add(1 as u16);
		if next_index == v_start {
			// queue presents as empty but is not
			// --> overwrite the oldest item in the FIFO brackets
			v_start = v_start.wrapping_add(1 as u16);
		}
		v_end = next_index;

		self.index_vector[bracket as usize] = (v_start, v_end);
		true
	}

	/// Pop an item from the start of the queue.
	///
	/// Will remove the item, but will not update the bounds in storage.
	fn pop(&mut self, bracket: QueueCluster) -> Option<Item> {

		if self.is_empty(bracket) {
			return None;
		}

		let (mut v_start, v_end) = self.index_vector[bracket as usize];

		let item_key = M::take(bracket, v_start);
		let item = N::take(bracket, item_key);

		v_start = v_start.wrapping_add(1 as u16);

		self.index_vector[bracket as usize] = (v_start, v_end);

		item.into()
	}

	/// Return whether to consider the queue empty.
	fn is_empty(&self, bracket: QueueCluster) -> bool {

		let (v_start, v_end) = self.index_vector[bracket as usize];
		
		v_start == v_end
	}

    /// Return the current size of the ring buffer as a BufferIndex.
    fn size(&self, bracket: QueueCluster) -> BufferIndex {
		let queue_cluster:u8 = 0;

		let (v_start, v_end) = self.index_vector[queue_cluster as usize];

        if v_start <= v_end {
            return v_end - v_start
        } else {
            return (BufferIndex::MAX - v_start) + v_end;
        }
    }

	/// Return whether the item_key is queued or not.
	fn is_queued(&self, item_key: ItemKey) -> bool {

		// check all brackets if key is queued
		for i in 0..self.index_vector.len() {
			if N::contains_key(i as QueueCluster, &item_key) {
				return true
			}
		}

		false
	}
}

#[cfg(test)]
mod tests {
	use super::*;
	use BracketsTrait;

	use codec::{Decode, Encode};
	use frame_support::{decl_module, decl_storage, impl_outer_origin, parameter_types};
	use sp_core::H256;
	use sp_runtime::{
		testing::Header,
		traits::{BlakeTwo256, IdentityLookup},
	};

	impl_outer_origin! {
		pub enum Origin for Test {}
	}

	// For testing the pallet, we construct most of a mock runtime. This means
	// first constructing a configuration type (`Test`) which `impl`s each of the
	// configuration traits of modules we want to use.
	#[derive(Clone, Eq, PartialEq)]
	pub struct Test;

	pub trait Config: frame_system::Config {}

	decl_module! {
		pub struct Module<T: Config> for enum Call where origin: T::Origin {
		}
	}

	type TestIdx = BufferIndex;

	type SomeKey = u64;

	#[derive(Clone, PartialEq, Encode, Decode, Default, Debug)]
	pub struct SomeStruct {
		foo: u64,
		bar: u64,
	}

	decl_storage! {
		trait Store for Module<T: Config> as BracketsTest {
			TestBracketsCount get(fn get_test_brackets): QueueCluster = 1; // C
			TestBracketIndices get(fn get_test_range): map hasher(twox_64_concat) QueueCluster => (TestIdx, TestIdx); // B
			TestBracketIndexKeyMap get(fn get_test_value): double_map hasher(twox_64_concat) QueueCluster, hasher(twox_64_concat) TestIdx => SomeKey; // M
			TestBracketKeyValueMap get(fn get_test_list): double_map hasher(twox_64_concat) QueueCluster, hasher(twox_64_concat) SomeKey => SomeStruct; // N
		}
	}

	// https://github.com/paritytech/substrate/pull/8090#issuecomment-776069095
	pub struct MockPalletInfo;
	impl frame_support::traits::PalletInfo for MockPalletInfo {
		fn index<P: 'static>() -> Option<usize> {
			Some(0)
		}
		fn name<P: 'static>() -> Option<&'static str> {
			Some("test")
		}
	}

	parameter_types! {
		pub const BlockHashCount: u64 = 250;
		pub BlockWeights: frame_system::limits::BlockWeights =
			frame_system::limits::BlockWeights::simple_max(1024);
	}
	impl frame_system::Config for Test {
		type BaseCallFilter = ();
		type BlockWeights = ();
		type BlockLength = ();
		type Origin = Origin;
		type Index = u64;
		type Call = ();
		type BlockNumber = u64;
		type Hash = H256;
		type Hashing = BlakeTwo256;
		type AccountId = u64;
		type Lookup = IdentityLookup<Self::AccountId>;
		type Header = Header;
		type Event = ();
		type BlockHashCount = BlockHashCount;
		type DbWeight = ();
		type Version = ();
		type PalletInfo = MockPalletInfo;
		type AccountData = ();
		type OnNewAccount = ();
		type OnKilledAccount = ();
		type SystemWeightInfo = ();
		type SS58Prefix = ();
        type OnSetCode = ();
	}

	impl Config for Test {}

	type TestModule = Module<Test>;

	// This function basically just builds a genesis storage key/value store according to
	// our desired mockup.
	fn new_test_ext() -> sp_io::TestExternalities {
		let storage = frame_system::GenesisConfig::default()
			.build_storage::<Test>()
			.unwrap();
		storage.into()
	}

	// ------------------------------------------------------------
	// brackets

	// Trait object that we will be interacting with.
	type Brackets = dyn BracketsTrait<SomeKey, SomeStruct>;
	// Implementation that we will instantiate.
	type Transient = BracketsTransient<
		SomeKey,
		SomeStruct,
		<TestModule as Store>::TestBracketsCount,
		<TestModule as Store>::TestBracketIndices,
		<TestModule as Store>::TestBracketIndexKeyMap,
		<TestModule as Store>::TestBracketKeyValueMap,
	>;

	#[test]
	fn simple_push() {
		new_test_ext().execute_with(|| {
			let bracket: QueueCluster = 0;
			let mut ring: Box<Brackets> = Box::new(Transient::new());

			let some_struct = SomeStruct { foo: 1, bar: 2 };
			ring.push(bracket, some_struct.foo.clone(), some_struct);
			ring.commit();
			let start_end = TestModule::get_test_range(0);
			assert_eq!(start_end, (0, 1));
			let some_key = TestModule::get_test_value(0, 0);
			let some_struct =  TestModule::get_test_list(0, some_key);
			assert_eq!(some_struct, SomeStruct { foo: 1, bar: 2 });
		})
	}

	#[test]
	fn size_tests() {
		new_test_ext().execute_with(|| {
			let bracket: QueueCluster = 0;
			let mut ring: Box<Brackets> = Box::new(Transient::new());

			assert_eq!(0, ring.size(bracket));

			let some_struct = SomeStruct { foo: 1, bar: 2 };
			ring.push(bracket,some_struct.foo.clone(), some_struct);
			ring.commit();

			let start_end = TestModule::get_test_range(0);
			assert_eq!(start_end, (0, 1));
			let some_key = TestModule::get_test_value(0, 0);
			let some_struct =  TestModule::get_test_list(0, some_key);
			assert_eq!(some_struct, SomeStruct { foo: 1, bar: 2 });
            assert_eq!(1, ring.size(bracket));

			let some_struct = SomeStruct { foo: 2, bar: 2 };
			ring.push(bracket,some_struct.foo.clone(), some_struct);
			ring.commit();
            assert_eq!(2, ring.size(bracket));

		})
	}

	#[test]
	fn drop_does_commit() {
		new_test_ext().execute_with(|| {
			// test drop here
			{
				let bracket: QueueCluster = 0;
				let mut ring: Box<Brackets> = Box::new(Transient::new());
				let some_struct = SomeStruct { foo: 1, bar: 2 };
				ring.push(bracket,some_struct.foo.clone(), some_struct);
			}
			let start_end = TestModule::get_test_range(0);
			assert_eq!(start_end, (0, 1));
			let some_key = TestModule::get_test_value(0, 0);
			let some_struct =  TestModule::get_test_list(0, some_key);
			assert_eq!(some_struct, SomeStruct { foo: 1, bar: 2 });
		})
	}

	#[test]
	fn simple_pop() {
		new_test_ext().execute_with(|| {
			let bracket: QueueCluster = 0;
			let mut ring: Box<Brackets> = Box::new(Transient::new());
			let some_struct = SomeStruct { foo: 1, bar: 2 };
			ring.push(bracket,some_struct.foo.clone(), some_struct);

			let item = ring.pop(bracket);
			ring.commit();
			assert!(item.is_some());
			let start_end = TestModule::get_test_range(0);
			assert_eq!(start_end, (1, 1));
		})
	}

	#[test]
	fn duplicate_check() {
		new_test_ext().execute_with(|| {
			let bracket: QueueCluster = 0;
			let mut ring: Box<Brackets> = Box::new(Transient::new());

			let some_struct = SomeStruct { foo: 1, bar: 2 };
			ring.push(bracket,some_struct.foo.clone(), some_struct);

			assert_eq!(1, ring.size(bracket));

			assert_eq!(true, ring.is_queued(1));

			let some_struct = SomeStruct { foo: 1, bar: 2 };
			ring.push(bracket,some_struct.foo.clone(), some_struct);
			
			// no change as its a duplicate
			assert_eq!(1, ring.size(bracket));

			assert_eq!(false, ring.is_queued(2));

			let some_struct = SomeStruct { foo: 2, bar: 2 };
			ring.push(bracket,some_struct.foo.clone(), some_struct);

			assert_eq!(2, ring.size(bracket));
		})
	}

	#[test]
	fn overflow_wrap_around() {
		new_test_ext().execute_with(|| {
			let bracket: QueueCluster = 0;
			let mut ring: Box<Brackets> = Box::new(Transient::new());

			let mut key:u64 = 0;

			for i in 1..(TestIdx::max_value() as u64) + 2 {
				let some_struct = SomeStruct { foo: key, bar: i };
				key = key + 1;
				assert_eq!(true, ring.push(bracket,some_struct.foo.clone(), some_struct));
			}
			ring.commit();
			let start_end = TestModule::get_test_range(0);
			assert_eq!(
				start_end,
				(1, 0),
				"range should be inverted because the index wrapped around"
			);

			let item = ring.pop(bracket);
			ring.commit();
			let (start, end) = TestModule::get_test_range(0);
			assert_eq!(start..end, 2..0);
			let item = item.expect("an item should be returned");
			assert_eq!(
				item.bar, 2,
				"the struct for field `bar = 2`, was placed at index 1"
			);

			let item = ring.pop(bracket);
			ring.commit();
			let (start, end) = TestModule::get_test_range(0);
			assert_eq!(start..end, 3..0);
			let item = item.expect("an item should be returned");
			assert_eq!(
				item.bar, 3,
				"the struct for field `bar = 3`, was placed at index 2"
			);

			for i in 1..4 {
				let some_struct = SomeStruct { foo: key, bar: i };
				key = key + 1;
				assert_eq!(true, ring.push(bracket,some_struct.foo.clone(), some_struct));
			}
			ring.commit();
			let start_end = TestModule::get_test_range(0);
			assert_eq!(start_end, (4, 3));
		})
	}
}