# Ajuna Network Pallet MatchMaker

This is a instantiable matchmaker Substrate pallet which lives as its own crate so it can be imported into multiple runtimes.

## Purpose

This pallet acts as a matchmaker for pairing players to match each other.

## Dependencies

### Traits

This pallet does not depend on any externally defined traits.

### Pallets

This pallet does not depend on any other FRAME pallet or externally developed modules.

## Installation

### Runtime `Cargo.toml`

To add this pallet to your runtime, simply include the following to your runtime's `Cargo.toml` file:

```TOML
# external pallets
pallet-matchmaker = {default-features = false, version = '3.0.0', git = 'https://github.com/ajuna-network/pallet-jton-matchmaker.git', tag = 'monthly-2021-10' }
```

and update your runtime's `std` feature to include this pallet:

```TOML
std = [
    # --snip--
    'pallet-matchmaker/std',
]
```

### Runtime `lib.rs`

You should implement it's trait like so:

AmountPlayers, amount of players need to create a match.
AmountBrackets, amount of brackets that exists for ranking or other purpose.

```rust
parameter_types! {
	pub const AmountPlayers: u8 = 2;
	pub const AmountBrackets: u8 = 3;
}

impl pallet_matchmaker::Config for Test {
	type Event = Event;
	type AmountPlayers = AmountPlayers;
	type AmountBrackets = AmountBrackets;
}
```

and include it in your `construct_runtime!` macro:

```rust
MatchMaker: pallet_matchmaker::{Pallet, Call, Storage, Event<T>},
```

### Genesis Configuration

This matchmaker pallet does not have any genesis configuration.

### Types

Additional types used in the matchmaker pallet

```json
{
  "MatchingType": {
    "_enum": [
      "None",
      "Simple",
      "Same",
      "Mix"
    ]
  },
  "PlayerStruct": {
    "account": "AccountId"
  }
}
```

## Reference Docs

You can view the reference docs for this pallet by running:

```
cargo doc --open
```
