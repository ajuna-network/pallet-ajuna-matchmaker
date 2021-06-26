use crate::{Error, mock::*};
use frame_support::{assert_ok, assert_noop};

#[test]
fn it_works_for_default_value() {
	new_test_ext().execute_with(|| {
		// Dispatch a signed extrinsic.
		assert_ok!(MatchMaker::do_something(Origin::signed(1), 42));
		// Read pallet storage and assert an expected result.
		assert_eq!(MatchMaker::something(), Some(42));
	});
}

#[test]
fn correct_error_for_none_value() {
	new_test_ext().execute_with(|| {
		// Ensure the expected error is thrown when no value is present.
		assert_noop!(
			MatchMaker::cause_error(Origin::signed(1)),
			Error::<Test>::NoneValue
		);
	});
}

#[test]
fn test_add_queue() {
	new_test_ext().execute_with(|| {

		let player1 = 1;
		let player2 = 2;

		assert_eq!(MatchMaker::do_try_match(), None);
		assert_eq!(MatchMaker::do_add_queue(player1), true);
		assert_eq!(MatchMaker::do_try_match(), None);
		assert_eq!(MatchMaker::do_add_queue(player2), true);
		assert_eq!(MatchMaker::do_try_match(), Some([1, 2]));
		assert_eq!(MatchMaker::do_try_match(), None);

		assert_eq!(MatchMaker::do_add_queue(player1), true);
		assert_eq!(MatchMaker::do_add_queue(player2), true);
		MatchMaker::do_empty_queue();
		assert_eq!(MatchMaker::do_try_match(), None);
	});
}

#[test]
fn test_queue() {
	new_test_ext().execute_with(|| {

	});
}