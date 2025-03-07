//! Parachain runtime mock.

mod mock_msg_queue;
mod xcm_config;
pub use xcm_config::*;

use core::marker::PhantomData;
use frame_support::{
    construct_runtime, derive_impl, parameter_types,
    traits::{ContainsPair, EnsureOrigin, EnsureOriginWithArg, Everything, Nothing},
    weights::{constants::WEIGHT_REF_TIME_PER_SECOND, Weight},
};

use frame_system::EnsureRoot;
use sp_core::{ConstU32, H256};
use sp_runtime::{
    traits::{Get, IdentityLookup},
    AccountId32,
};
use sp_std::prelude::*;

use xcm::latest::prelude::*;
use xcm_builder::{EnsureXcmOrigin, SignedToAccountId32};
use xcm_executor::{traits::ConvertLocation, XcmExecutor};

pub type AccountId = AccountId32;
pub type Balance = u128;

parameter_types! {
    pub const BlockHashCount: u64 = 250;
}

#[derive_impl(frame_system::config_preludes::TestDefaultConfig)]
impl frame_system::Config for Runtime {
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    type Nonce = u64;
    type Hash = H256;
    type Hashing = ::sp_runtime::traits::BlakeTwo256;
    type AccountId = AccountId;
    type Lookup = IdentityLookup<Self::AccountId>;
    type Block = Block;
    type RuntimeEvent = RuntimeEvent;
    type BlockHashCount = BlockHashCount;
    type BlockWeights = ();
    type BlockLength = ();
    type Version = ();
    type PalletInfo = PalletInfo;
    type AccountData = pallet_balances::AccountData<Balance>;
    type OnNewAccount = ();
    type OnKilledAccount = ();
    type DbWeight = ();
    type BaseCallFilter = Everything;
    type SystemWeightInfo = ();
    type SS58Prefix = ();
    type OnSetCode = ();
    type MaxConsumers = ConstU32<16>;
}

parameter_types! {
    pub ExistentialDeposit: Balance = 1;
    pub const MaxLocks: u32 = 50;
    pub const MaxReserves: u32 = 50;
}

impl pallet_balances::Config for Runtime {
    type MaxLocks = MaxLocks;
    type Balance = Balance;
    type RuntimeEvent = RuntimeEvent;
    type DustRemoval = ();
    type ExistentialDeposit = ExistentialDeposit;
    type AccountStore = System;
    type WeightInfo = ();
    type MaxReserves = MaxReserves;
    type ReserveIdentifier = [u8; 8];
    type RuntimeHoldReason = RuntimeHoldReason;
    type RuntimeFreezeReason = RuntimeFreezeReason;
    type FreezeIdentifier = ();
    type MaxFreezes = ConstU32<0>;
}

impl pallet_uniques::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type CollectionId = Location;
    type ItemId = AssetInstance;
    type Currency = Balances;
    type CreateOrigin = ForeignCreators;
    type ForceOrigin = frame_system::EnsureRoot<AccountId>;
    type CollectionDeposit = frame_support::traits::ConstU128<1_000>;
    type ItemDeposit = frame_support::traits::ConstU128<1_000>;
    type MetadataDepositBase = frame_support::traits::ConstU128<1_000>;
    type AttributeDepositBase = frame_support::traits::ConstU128<1_000>;
    type DepositPerByte = frame_support::traits::ConstU128<1>;
    type StringLimit = ConstU32<64>;
    type KeyLimit = ConstU32<64>;
    type ValueLimit = ConstU32<128>;
    type Locker = ();
    type WeightInfo = ();
}

// `EnsureOriginWithArg` impl for `CreateOrigin` which allows only XCM origins
// which are locations containing the class location.
pub struct ForeignCreators;
impl EnsureOriginWithArg<RuntimeOrigin, Location> for ForeignCreators {
    type Success = AccountId;

    fn try_origin(
        o: RuntimeOrigin,
        a: &Location,
    ) -> sp_std::result::Result<Self::Success, RuntimeOrigin> {
        let origin_location = pallet_xcm::EnsureXcm::<Everything>::try_origin(o.clone())?;
        if !a.starts_with(&origin_location) {
            return Err(o);
        }
        xcm_config::LocationToAccountId::convert_location(&origin_location).ok_or(o)
    }
}

parameter_types! {
    pub const ReservedXcmpWeight: Weight = Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND.saturating_div(4), 0);
    pub const ReservedDmpWeight: Weight = Weight::from_parts(WEIGHT_REF_TIME_PER_SECOND.saturating_div(4), 0);
}

impl mock_msg_queue::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type XcmExecutor = XcmExecutor<XcmConfig>;
}

pub type LocalOriginToLocation = SignedToAccountId32<RuntimeOrigin, AccountId, RelayNetwork>;

pub struct TrustedLockerCase<T>(PhantomData<T>);
impl<T: Get<(Location, AssetFilter)>> ContainsPair<Location, Asset> for TrustedLockerCase<T> {
    fn contains(origin: &Location, asset: &Asset) -> bool {
        let (o, a) = T::get();
        a.matches(asset) && &o == origin
    }
}

parameter_types! {
    pub RelayTokenForRelay: (Location, AssetFilter) = (Parent.into(), Wild(AllOf { id: AssetId(Parent.into()), fun: WildFungible }));
}

pub type TrustedLockers = TrustedLockerCase<RelayTokenForRelay>;

impl pallet_xcm::Config for Runtime {
    type RuntimeEvent = RuntimeEvent;
    type SendXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type XcmRouter = XcmRouter;
    type ExecuteXcmOrigin = EnsureXcmOrigin<RuntimeOrigin, LocalOriginToLocation>;
    type XcmExecuteFilter = Everything;
    type XcmExecutor = XcmExecutor<XcmConfig>;
    type XcmTeleportFilter = Nothing;
    type XcmReserveTransferFilter = Everything;
    type Weigher = xcm_config::Weigher;
    type UniversalLocation = UniversalLocation;
    type RuntimeOrigin = RuntimeOrigin;
    type RuntimeCall = RuntimeCall;
    const VERSION_DISCOVERY_QUEUE_SIZE: u32 = 100;
    type AdvertisedXcmVersion = pallet_xcm::CurrentXcmVersion;
    type Currency = Balances;
    type CurrencyMatcher = ();
    type TrustedLockers = TrustedLockers;
    type SovereignAccountOf = xcm_config::LocationToAccountId;
    type MaxLockers = ConstU32<8>;
    type MaxRemoteLockConsumers = ConstU32<0>;
    type RemoteLockConsumerIdentifier = ();
    type WeightInfo = pallet_xcm::TestWeightInfo;
    type AdminOrigin = EnsureRoot<AccountId>;
}

type Block = frame_system::mocking::MockBlock<Runtime>;

construct_runtime!(
    pub enum Runtime
    {
        System: frame_system,
        Balances: pallet_balances,
        MsgQueue: mock_msg_queue,
        PolkadotXcm: pallet_xcm,
        ForeignUniques: pallet_uniques,
    }
);
