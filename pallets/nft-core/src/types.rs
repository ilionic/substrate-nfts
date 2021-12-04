use frame_support::pallet_prelude::*;

#[cfg(feature = "std")]
use serde::{Deserialize, Serialize};

use scale_info::TypeInfo;

#[derive(Encode, Decode, Eq, Copy, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct ClassInfo<BoundedString> {
	/// Arbitrary data about a class, e.g. IPFS hash
	pub metadata: BoundedString,
}

#[derive(Encode, Decode, Eq, Copy, PartialEq, Clone, RuntimeDebug, TypeInfo)]
#[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
pub struct InstanceInfo<AccountId, BoundedString> {
	/// The user account which receives the royalty
	pub author: AccountId,
	/// Royalty in percent in range 0-99
	pub royalty: u8,
	/// Arbitrary data about an instance, e.g. IPFS hash
	pub metadata: BoundedString,
}

// TODO ResourceInfo
// #[derive(Encode, Decode, Eq, Copy, PartialEq, Clone, RuntimeDebug, TypeInfo)]
// #[cfg_attr(feature = "std", derive(Serialize, Deserialize))]
// pub struct ResourceInfo<BoundedString> {
// 	base_id: Option<u32>,
// 	pending: bool,
// 	src: BoundedString,
// 	slot_id: Option<BoundedString>,
// 	thumb: Option<BoundedString>,
// 	theme: Option<BoundedString> // TODO: theme could be a struct?
// }