use crate as pallet_account;
use crate::{mock::*, Error};
use frame_support::{assert_noop, assert_ok, traits::GenesisBuild};
use pallet_account::Role;

pub const ALICE: u64 = 1;
pub const BOB: u64 = 2;
pub const CHARLIE: u64 = 3;
pub const DAVE: u64 = 4;
pub const EVE: u64 = 5;
pub const FRANK: u64 = 6;
pub const GEORGE: u64 = 7;

// ALICE(1): system manager
// BOB(2): manufacture
// CHARLIE(3): VAO
// DAVE(4): VAD
// EVE(5): USER

#[test]
fn start_system() {
	ExtBuilder::default().set_genesis_account().execute_with(|| {
		// register account
		assert_ok!(Account::register(Origin::signed(BOB)));
		assert_ok!(Account::register(Origin::signed(CHARLIE)));
		assert_ok!(Account::register(Origin::signed(DAVE)));
		assert_ok!(Account::register(Origin::signed(EVE)));
		// 2 claimed to be manufacture
		assert_ok!(Account::claim_role(Origin::signed(BOB), Role::VM));
		// 3 claimed to be authorized organization
		assert_ok!(Account::claim_role(Origin::signed(CHARLIE), Role::VAO));
		// 4 claimed to be distributer
		assert_ok!(Account::claim_role(Origin::signed(DAVE), Role::VAD));
		// 2 is manufacture
		assert_ok!(Account::approve_role(Origin::signed(ALICE), BOB, Role::VM));
		// 3 is authorized organization
		assert_ok!(Account::approve_role(Origin::signed(ALICE), CHARLIE, Role::VAO));
		// 4 is distributer
		assert_ok!(Account::approve_role(Origin::signed(ALICE), DAVE, Role::VAD));
		// VM(2) register vaccine information
		assert_ok!(pallet_vaccine::register_vac_info(Origin::signed(BOB), Some(1)));
		// VAO approve vaccine
		assert_ok!(Account::approve_vaccine(Origin::signed(CHARLIE), Some(1)));
		// VM transfer vaccine to VAD
		assert_ok!(Account::transfer_vaccine(Origin::signed(BOB), DAVE, Some(1)));
		// VAD receive vaccine from VM
		assert_ok!(Account::receive_vaccine(Origin::signed(DAVE), BOB, Some(1)));
		// User have a vaccine in VAD
		assert_ok!(Account::transfer_get_vaccine_right(Origin::signed(DAVE), EVE, Some(1)));
		assert_ok!(Account::confirm_vaccine(Origin::signed(EVE), Some(DAVE), Some(1)));

		println!("vaccine info: {:?}", Account::vaccines(1));
		println!("vaccine passport info: {:?}", Account::vaccine_passports(EVE));
	});
}

// ALICE(1): system manager
// BOB(2): manufacture
// CHARLIE(3): user

#[test]
fn should_cause_error_for_register_role() {
	ExtBuilder::default().set_genesis_account().execute_with(|| {
		// 1 is system manager by default
		assert_eq!(Account::sys_man(ALICE), true);
		// 2 claimed to be manufacture
		assert_ok!(Account::claim_vm(Origin::signed(BOB)));

		// claim again after claimed but before approved
		assert_noop!(Account::claim_vm(Origin::signed(BOB)), Error::<Test>::AlreadyClaimed);

		// 2 is manufacture
		assert_ok!(Account::approve_vm(Origin::signed(ALICE), BOB));

		// claim again after approved
		assert_noop!(Account::claim_vm(Origin::signed(BOB)), Error::<Test>::AlreadyApproved);

		// not system manager approve role
		assert_noop!(Account::approve_vm(Origin::signed(CHARLIE), BOB), Error::<Test>::NotSysMan);

		// approve again after approved
		assert_noop!(Account::approve_vm(Origin::signed(ALICE), BOB), Error::<Test>::NotClaimed);
		// approve entity which does not claim
		assert_noop!(
			Account::approve_vm(Origin::signed(ALICE), CHARLIE),
			Error::<Test>::NotClaimed
		);
	});
}

// ALICE(1): system manager
// BOB(2): manufacture
// CHARLIE(3): VAO
// DAVE(4): user
#[test]
fn should_cause_error_for_register_info_about_vaccine() {
	ExtBuilder::default().set_genesis_account().execute_with(|| {
		// 1 is system manager by default
		assert_eq!(Account::sys_man(ALICE), true);
		// 2 claimed to be manufacture
		assert_ok!(Account::claim_vm(Origin::signed(BOB)));
		// 3 claimed to be authorized organization
		assert_ok!(Account::claim_vao(Origin::signed(CHARLIE)));
		// 2 is manufacture
		assert_ok!(Account::approve_vm(Origin::signed(ALICE), BOB));
		// 3 is authorized organization
		assert_ok!(Account::approve_vao(Origin::signed(ALICE), CHARLIE));

		// NOT sysman register vaccine type
		assert_noop!(Account::register_vac_type(Origin::signed(DAVE)), Error::<Test>::NotSysMan);

		// sysman(1) register vaccine type
		assert_ok!(Account::register_vac_type(Origin::signed(ALICE)));

		// NOT manufacture register vaccine type
		assert_noop!(
			Account::register_vac_info(Origin::signed(DAVE), Some(1)),
			Error::<Test>::NotManufacture
		);

		// sysman register not exist vaccine type
		assert_noop!(
			Account::register_vac_info(Origin::signed(BOB), Some(2)),
			Error::<Test>::NotRegisteredVaccineType
		);

		//register vaciine info
		assert_ok!(Account::register_vac_info(Origin::signed(BOB), Some(1)));

		// vaccine has been already registered
		assert_noop!(
			Account::register_vac_info(Origin::signed(BOB), Some(1)),
			Error::<Test>::VaccineIsRegistered
		);

		// approve vaccine
		assert_ok!(Account::approve_vaccine(Origin::signed(CHARLIE), Some(1)));

		// not VAO register vaciine info
		assert_noop!(
			Account::approve_vaccine(Origin::signed(DAVE), Some(1)),
			Error::<Test>::NotOrganization
		);

		// VAO register not exist vaciine id
		assert_noop!(
			Account::approve_vaccine(Origin::signed(CHARLIE), Some(2)),
			Error::<Test>::NotRegisteredVaccine
		);

		// TODO: duplicate vaccine approvement
		// TODO: exceed max list size (3 in mock runtime)
	});
}

// ALICE(1): system manager
// BOB(2): manufacture
// CHARLIE(3): VAO
// DAVE(4): VAD
// EVE(5): VAD
// FRANK(6): user
// GEORGE(7):
#[test]
fn should_cause_error_for_transfer_vaccine() {
	ExtBuilder::default().set_genesis_account().execute_with(|| {
		// 1 is system manager by default
		assert_eq!(Account::sys_man(ALICE), true);
		// 2 claimed to be manufacture
		assert_ok!(Account::claim_vm(Origin::signed(BOB)));
		// 2 is manufacture
		assert_ok!(Account::approve_vm(Origin::signed(ALICE), BOB));
		// 4 claimed to be distributer
		assert_ok!(Account::claim_vad(Origin::signed(DAVE)));
		// 4 is distributer
		assert_ok!(Account::approve_vad(Origin::signed(ALICE), DAVE));
		// 7 claimed to be distributer
		assert_ok!(Account::claim_vad(Origin::signed(EVE)));
		// 7 is distributer
		assert_ok!(Account::approve_vad(Origin::signed(ALICE), EVE));
		//register vaciine type info
		assert_ok!(Account::register_vac_type(Origin::signed(ALICE)));
		//register vaciine info
		assert_ok!(Account::register_vac_info(Origin::signed(BOB), Some(1)));

		// Not manufacture or distributer transfer vaccine
		assert_noop!(
			Account::transfer_vaccine(Origin::signed(FRANK), DAVE, Some(1)),
			Error::<Test>::NotManufacture
		);

		// transfer to myself
		assert_noop!(
			Account::transfer_vaccine(Origin::signed(BOB), BOB, Some(1)),
			Error::<Test>::TransferByMyself
		);

		// Not owner transfer vaccine
		assert_noop!(
			Account::transfer_vaccine(Origin::signed(DAVE), FRANK, Some(1)),
			Error::<Test>::WrongVaccineOwner
		);

		// transfer not exist vaccine
		assert_noop!(
			Account::transfer_vaccine(Origin::signed(BOB), DAVE, Some(3)),
			Error::<Test>::NotRegisteredVaccine
		);

		// transfer vaccine
		assert_ok!(Account::transfer_vaccine(Origin::signed(BOB), DAVE, Some(1)));

		// receive not exist vaccine
		assert_noop!(
			Account::receive_vaccine(Origin::signed(DAVE), BOB, Some(3)),
			Error::<Test>::NotRegisteredVaccine
		);

		// not manufacture or distributer receive vaccine
		assert_noop!(
			Account::receive_vaccine(Origin::signed(FRANK), BOB, Some(1)),
			Error::<Test>::NotManufacture
		);

		// not specified receiver
		assert_noop!(
			Account::receive_vaccine(Origin::signed(EVE), FRANK, Some(1)),
			Error::<Test>::NotVaccineBuyer
		);

		// receive from not vaccine owner
		assert_noop!(
			Account::receive_vaccine(Origin::signed(DAVE), EVE, Some(1)),
			Error::<Test>::WrongVaccineOwner
		);

		// receive vaccine
		assert_ok!(Account::receive_vaccine(Origin::signed(DAVE), BOB, Some(1)));

		// receive twice
		assert_noop!(
			Account::receive_vaccine(Origin::signed(DAVE), BOB, Some(1)),
			Error::<Test>::VaccineAlreadyMine
		);
	});
}

// ALICE(1): system manager
// BOB(2): manufacture
// CHARLIE(3): VAO
// DAVE(4): VAD
// EVE(5): VAD
// FRANK(6): user
// GEORGE(7): user
#[test]
fn should_cause_error_for_get_confirm_vaccine() {
	ExtBuilder::default().set_genesis_account().execute_with(|| {
		// 1 is system manager by default
		assert_eq!(Account::sys_man(ALICE), true);
		// 2 claimed to be manufacture
		assert_ok!(Account::claim_vm(Origin::signed(BOB)));
		// 2 is manufacture
		assert_ok!(Account::approve_vm(Origin::signed(ALICE), BOB));
		// 4 claimed to be distributer
		assert_ok!(Account::claim_vad(Origin::signed(DAVE)));
		// 4 is distributer
		assert_ok!(Account::approve_vad(Origin::signed(ALICE), DAVE));
		// 7 claimed to be distributer
		assert_ok!(Account::claim_vad(Origin::signed(EVE)));
		// 7 is distributer
		assert_ok!(Account::approve_vad(Origin::signed(ALICE), EVE));
		//register vaciine type info
		assert_ok!(Account::register_vac_type(Origin::signed(ALICE)));
		//register vaciine info
		assert_ok!(Account::register_vac_info(Origin::signed(BOB), Some(1)));
		//register vaciine type info
		assert_ok!(Account::register_vac_type(Origin::signed(ALICE)));
		//register vaciine info
		assert_ok!(Account::register_vac_info(Origin::signed(BOB), Some(2)));
		// transfer vaccine
		assert_ok!(Account::transfer_vaccine(Origin::signed(BOB), DAVE, Some(1)));
		// receive vaccine
		assert_ok!(Account::receive_vaccine(Origin::signed(DAVE), BOB, Some(1)));
		// transfer vaccine
		assert_ok!(Account::transfer_vaccine(Origin::signed(BOB), DAVE, Some(2)));
		// receive vaccine
		assert_ok!(Account::receive_vaccine(Origin::signed(DAVE), BOB, Some(2)));
		// transfer vaccine
		assert_ok!(Account::transfer_vaccine(Origin::signed(DAVE), FRANK, Some(2)));

		// only manufacture or distributer
		assert_noop!(
			Account::transfer_get_vaccine_right(Origin::signed(FRANK), DAVE, Some(1)),
			Error::<Test>::NotManufacture
		);
		// only manufacture or distributer
		assert_noop!(
			Account::transfer_get_vaccine_right(Origin::signed(DAVE), FRANK, Some(3)),
			Error::<Test>::NotRegisteredVaccine
		);
		// transfer to myself
		assert_noop!(
			Account::transfer_get_vaccine_right(Origin::signed(DAVE), DAVE, Some(1)),
			Error::<Test>::TransferByMyself
		);
		// not vaccine owner
		assert_noop!(
			Account::transfer_get_vaccine_right(Origin::signed(EVE), FRANK, Some(1)),
			Error::<Test>::WrongVaccineOwner
		);
		// receive vaccine inoculation right
		assert_ok!(Account::transfer_get_vaccine_right(Origin::signed(DAVE), FRANK, Some(1)));

		// not send final transfer to me
		assert_noop!(
			Account::confirm_vaccine(Origin::signed(FRANK), Some(DAVE), Some(3)),
			Error::<Test>::NotRegisteredVaccine
		);
		// not vaccine user
		assert_noop!(
			Account::confirm_vaccine(Origin::signed(GEORGE), Some(DAVE), Some(1)),
			Error::<Test>::NotVaccineBuyer
		);
		// not vaccine previous owner
		assert_noop!(
			Account::confirm_vaccine(Origin::signed(FRANK), Some(EVE), Some(1)),
			Error::<Test>::WrongVaccineOwner
		);
		// not transfer vaccine but transfer_get_vaccine_right
		assert_noop!(
			Account::confirm_vaccine(Origin::signed(FRANK), Some(DAVE), Some(2)),
			Error::<Test>::NotSendFinalTransfer
		);

		// confirm use vaccine
		assert_ok!(Account::confirm_vaccine(Origin::signed(FRANK), Some(DAVE), Some(1)));

		// not send final transfer to me
		assert_noop!(
			Account::confirm_vaccine(Origin::signed(FRANK), Some(DAVE), Some(1)),
			Error::<Test>::VaccineAlreadyMine
		);
	});
}
