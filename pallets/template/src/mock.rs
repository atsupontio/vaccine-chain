use crate as pallet_template;
// use crate as pallet_balances;
use frame_support::traits::{GenesisBuild, ConstU16, ConstU32, ConstU64};
use frame_system as system;
use sp_core::H256;
use sp_runtime::{
	testing::Header,
	traits::{BlakeTwo256, IdentityLookup},
};
use frame_support::parameter_types;

type UncheckedExtrinsic = frame_system::mocking::MockUncheckedExtrinsic<Test>;
type Block = frame_system::mocking::MockBlock<Test>;

// Configure a mock runtime to test the pallet.
frame_support::construct_runtime!(
	pub enum Test where
		Block = Block,
		NodeBlock = Block,
		UncheckedExtrinsic = UncheckedExtrinsic,
	{
		System: frame_system::{Pallet, Call, Config, Storage, Event<T>},
		Balances: pallet_balances::{Pallet, Call, Storage, Config<T>, Event<T>},
		TemplateModule: pallet_template::{Pallet, Call, Storage, Event<T>},
	}
);

impl system::Config for Test {
	type BaseCallFilter = frame_support::traits::Everything;
	type BlockWeights = ();
	type BlockLength = ();
	type DbWeight = ();
	type Origin = Origin;
	type Call = Call;
	type Index = u64;
	type BlockNumber = u64;
	type Hash = H256;
	type Hashing = BlakeTwo256;
	type AccountId = u64;
	type Lookup = IdentityLookup<Self::AccountId>;
	type Header = Header;
	type Event = Event;
	type BlockHashCount = ConstU64<250>;
	type Version = ();
	type PalletInfo = PalletInfo;
	type AccountData = pallet_balances::AccountData<u64>;
	type OnNewAccount = ();
	type OnKilledAccount = ();
	type SystemWeightInfo = ();
	type SS58Prefix = ConstU16<42>;
	type OnSetCode = ();
	type MaxConsumers = frame_support::traits::ConstU32<16>;
}


impl pallet_balances::Config for Test {
	type MaxLocks = ConstU32<50>;
	type MaxReserves = ();
	type ReserveIdentifier = [u8; 8];
	type Balance = u64;
	type Event = Event;
	type DustRemoval = ();
	type ExistentialDeposit = ConstU64<500>;
	type AccountStore = System;
	type WeightInfo = ();
}

impl pallet_template::Config for Test {
	type Event = Event;
	type MaxListSize = ConstU32<30>;
	type UnixTime = pallet_timestamp::Pallet<Self>;
}

parameter_types! {
	pub const MinimumPeriod: u64 = 5;
}

impl pallet_timestamp::Config for Test {
	type Moment = u64;
	type OnTimestampSet = ();
	type MinimumPeriod = MinimumPeriod;
	type WeightInfo = ();
}


pub struct ExtBuilder;

impl ExtBuilder {
	pub fn build(self) -> sp_io::TestExternalities {
	 let mut t = frame_system::GenesisConfig::default().build_storage::<Test>().unwrap();
	 pallet_balances::GenesisConfig::<Test> {
	  balances: vec![
	   (1, 1000000000000000),
	   (2, 2000000000000000),
	   (3, 3000000000000000),
	   (4, 4000000000000000),
	   (5, 5000000000000000),
	   (6, 6000000000000000)
	  ],
	 }
	  .assimilate_storage(&mut t)
	  .unwrap();

   
	 let mut ext = sp_io::TestExternalities::new(t);
	 ext.execute_with(|| System::set_block_number(1));
	 ext
	}

	pub fn set_genesis_account(self) -> sp_io::TestExternalities {
		let mut t = pallet_template::GenesisConfig::<Test>::default().build_storage().unwrap();
	  pallet_template::GenesisConfig::<Test> {
		genesis_account: vec![
		 1
		],
	   }
		.assimilate_storage(&mut t)
		.unwrap();
	   
		let mut ext = sp_io::TestExternalities::new(t);
	 	ext.execute_with(|| System::set_block_number(1));
	 	ext
	}
}

impl Default for ExtBuilder {
    fn default() -> Self {
		Self
    }
}

// Build genesis storage according to the mock runtime.
pub fn new_test_ext() -> sp_io::TestExternalities {
	system::GenesisConfig::default().build_storage::<Test>().unwrap().into()
}
