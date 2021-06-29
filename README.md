# Jeton Network Pallet MatchMaker

This is a matchmaker Substrate pallet which lives as its own crate so it can be imported into multiple runtimes.

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
pallet-matchmaker = {default-features = false, version = '0.1.0', git = 'https://github.com/JetonNetwork/pallet-jton-matchmaker.git'}
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

```rust
/// Used for test_module
impl pallet_matchmaker::Config for Runtime {
	type Event = Event;
}
```

and include it in your `construct_runtime!` macro:

```rust
MatchMaker: pallet_matchmaker::{Pallet, Call, Storage, Event<T>},
```

### Genesis Configuration

This matchmaker pallet does not have any genesis configuration.

### Types

Additional types used in ConnectFour

```json
{
  "BoardState": {
    "_enum": [
      "None",
      "Running",
      "Finished(AccountId)"
    ]
  },
  "BoardStruct": {
    "id": "Hash",
    "red": "AccountId",
    "blue": "AccountId",
    "board": "[[u8; 6]; 7]",
    "last_turn": "BlockNumber",
    "next_player": "u8",
    "board_state": "BoardState"
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
