use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, traits::GenesisBuild};
use crate as pallet_template;

pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;
pub const DAVE: u64 = 4;
pub const EVE: u64 = 5;


// ALICE(1): system manager
// BOB(2): manufacture
// CHARLIE(3): VAO
// DAVE(4): VAD
// EVE(5): USER

	#[test]
fn start_system() {
	ExtBuilder::default()
		.set_genesis_account()
        .execute_with(|| {
		// 1 is system manager by default
		assert_eq!(TemplateModule::sys_man(ALICE), true);
		// 2 claimed to be manufacture
		assert_ok!(TemplateModule::claim_vm(Origin::signed(BOB)));
		// 3 claimed to be authorized organization
		assert_ok!(TemplateModule::claim_vao(Origin::signed(CHARLIE)));
		// 4 claimed to be distributer
		assert_ok!(TemplateModule::claim_vad(Origin::signed(DAVE)));
		// 2 is manufacture
		assert_ok!(TemplateModule::approve_vm(Origin::signed(ALICE), BOB));
		// 3 is authorized organization
		assert_ok!(TemplateModule::approve_vao(Origin::signed(ALICE), CHARLIE));
		// 4 is distributer
		assert_ok!(TemplateModule::approve_vad(Origin::signed(ALICE), DAVE));
		// sysman(1) register vaccine type
		assert_ok!(TemplateModule::register_vac_type(Origin::signed(ALICE)));
		// VM(2) register vaccine information
		assert_ok!(TemplateModule::register_vac_info(Origin::signed(BOB), Some(1)));
		// VAO approve vaccine
		assert_ok!(TemplateModule::approve_vaccine(Origin::signed(CHARLIE), Some(1)));
		// VM transfer vaccine to VAD
		assert_ok!(TemplateModule::transfer_vaccine(Origin::signed(BOB), DAVE, Some(1)));
		// VAD receive vaccine from VM
		assert_ok!(TemplateModule::receive_vaccine(Origin::signed(DAVE), BOB, Some(1)));
		// User have a vaccine in VAD
		assert_ok!(TemplateModule::transfer_get_vaccine_right(Origin::signed(DAVE), EVE, Some(1)));
		assert_ok!(TemplateModule::confirm_vaccine(Origin::signed(EVE), Some(DAVE), Some(1)));

		println!("vaccine info: {:?}", TemplateModule::vaccines(1));
		println!("vaccine passport info: {:?}", TemplateModule::vaccine_passports(EVE));
	});
}


// #[test]
// fn correct_error_for_none_value() {
// 	new_test_ext().execute_with(|| {
// 		// Ensure the expected error is thrown when no value is present.
// 		assert_noop!(TemplateModule::cause_error(Origin::signed(1)), Error::<Test>::NoneValue);
// 	});
// }
