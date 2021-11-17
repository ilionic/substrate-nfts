#![cfg_attr(not(feature = "std"), no_std)]
#![allow(clippy::unused_unit)]
#![allow(clippy::upper_case_acronyms)]

use codec::HasCompact;
use frame_support::{
	dispatch::DispatchResult,
	ensure,
	traits::{tokens::nonfungibles::*, BalanceStatus, Currency, NamedReservableCurrency, ReservableCurrency},
	transactional, BoundedVec,
};
use frame_system::ensure_signed;

use sp_runtime::traits::{AtLeast32BitUnsigned, CheckedAdd, One, StaticLookup, Zero};
use sp_std::{convert::TryInto, vec::Vec};

use types::{ClassInfo, InstanceInfo};

// use pallet_uniques::traits::InstanceReserve;
// use pallet_uniques::{ClassTeam, DepositBalanceOf};

// pub type BalanceOf<T> = <<T as Config>::Currency as Currency<<T as frame_system::Config>::AccountId>>::Balance;
pub type ClassInfoOf<T> = ClassInfo<BoundedVec<u8, <T as pallet_uniques::Config>::StringLimit>>;
pub type InstanceInfoOf<T> =
	InstanceInfo<<T as frame_system::Config>::AccountId, BoundedVec<u8, <T as pallet_uniques::Config>::StringLimit>>;

pub mod types;

// Re-export pallet items so that they can be accessed from the crate namespace.
pub use pallet::*;

#[frame_support::pallet]
pub mod pallet {

	use super::*;
	use frame_support::{dispatch::DispatchResult, pallet_prelude::*};
	use frame_system::pallet_prelude::*;

	/// Configure the pallet by specifying the parameters and types on which it depends.
	#[pallet::config]
	pub trait Config: frame_system::Config + pallet_uniques::Config {
		/// Because this pallet emits events, it depends on the runtime's definition of an event.
		type Event: From<Event<Self>> + IsType<<Self as frame_system::Config>::Event>;

		type NftClassId: Member + Parameter + Default + Copy + HasCompact + AtLeast32BitUnsigned + Into<Self::ClassId>;
		type ProtocolOrigin: EnsureOrigin<Self::Origin>;
		type NftInstanceId: Member
			+ Parameter
			+ Default
			+ Copy
			+ HasCompact
			+ AtLeast32BitUnsigned
			+ From<Self::InstanceId>
			+ Into<Self::InstanceId>;		
	}

	/// Next available class ID.
	#[pallet::storage]
	#[pallet::getter(fn next_class_id)]
	pub type NextClassId<T: Config> = StorageValue<_, T::NftClassId, ValueQuery>;	

	/// Next available token ID.
	#[pallet::storage]
	#[pallet::getter(fn next_instance_id)]
	pub type NextInstanceId<T: Config> = StorageMap<_, Twox64Concat, T::NftClassId, T::NftInstanceId, ValueQuery>;	

	#[pallet::storage]
	#[pallet::getter(fn classes)]
	/// Stores class info
	pub type Classes<T: Config> = StorageMap<_, Twox64Concat, T::NftClassId, ClassInfoOf<T>>;	

	#[pallet::storage]
	#[pallet::getter(fn instances)]
	/// Stores instance info
	pub type Instances<T: Config> =
		StorageDoubleMap<_, Twox64Concat, T::NftClassId, Twox64Concat, T::NftInstanceId, InstanceInfoOf<T>>;	

	#[pallet::pallet]
	#[pallet::generate_store(pub(super) trait Store)]
	pub struct Pallet<T>(_);

	// Pallets use events to inform users when important changes are made.
	// https://docs.substrate.io/v3/runtime/events-and-errors
	#[pallet::event]
	#[pallet::generate_deposit(pub(super) fn deposit_event)]
	pub enum Event<T: Config> {
		/// Event documentation should end with an array that provides descriptive names for event
		/// parameters. [something, who]
		SomethingStored(u32, T::AccountId),
		ClassCreated(T::AccountId, T::NftClassId),
		InstanceMinted(T::AccountId, T::NftClassId, T::NftInstanceId),
	}

	// Errors inform users that something went wrong.
	#[pallet::error]
	pub enum Error<T> {
		/// Error names should be descriptive.
		NoneValue,
		/// Errors should have helpful documentation associated with them.
		StorageOverflow,
		TooLong,
		NoAvailableClassId,
		MetadataNotSet,
		AuthorNotSet,
		NoAvailableInstanceId,
		NotInRange,
		RoyaltyNotSet
	}

	#[pallet::call]
	impl<T: Config> Pallet<T> {


		/// Mints an NFT in the specified class
		/// Sets metadata and the royalty attribute
		///
		/// Parameters:
		/// - `class_id`: The class of the asset to be minted.
		/// - `instance_id`: The instance value of the asset to be minted.
		/// - `author`: Receiver of the royalty
		/// - `royalty`: Percentage reward from each trade for the author
		/// - `metadata`: Arbitrary data about an instance, e.g. IPFS hash
		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		#[transactional]
		pub fn mint(
			origin: OriginFor<T>,
			class_id: T::NftClassId,
			author: Option<T::AccountId>,
			royalty: Option<u8>,
			metadata: Option<Vec<u8>>,
		) -> DispatchResult {
			let sender = match T::ProtocolOrigin::try_origin(origin) {
				Ok(_) => None,
				Err(origin) => Some(ensure_signed(origin)?),
			};

			if let Some(r) = royalty {
				ensure!(r < 100, Error::<T>::NotInRange);
			}

			let instance_id: T::NftInstanceId =
				NextInstanceId::<T>::try_mutate(class_id, |id| -> Result<T::NftInstanceId, DispatchError> {
					let current_id = *id;
					*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableInstanceId)?;
					Ok(current_id)
				})?;

			pallet_uniques::Pallet::<T>::do_mint(
				class_id.into(),
				instance_id.into(),
				sender.clone().unwrap_or_default(),
				|_details| {
					Ok(())
				},
			)?;

			let metadata_bounded = Self::to_bounded_string(metadata.ok_or(Error::<T>::MetadataNotSet)?)?;
			let author = author.ok_or(Error::<T>::AuthorNotSet)?;
			let royalty = royalty.ok_or(Error::<T>::RoyaltyNotSet)?;

			Instances::<T>::insert(
				class_id,
				instance_id,
				InstanceInfo {
					author,
					royalty,
					metadata: metadata_bounded,
				},
			);

			Self::deposit_event(Event::InstanceMinted(sender.unwrap_or_default(), class_id, instance_id));

			Ok(())
		}


		#[pallet::weight(10_000 + T::DbWeight::get().reads_writes(1,1))]
		#[transactional]
		pub fn create_class(origin: OriginFor<T>, metadata: Vec<u8>) -> DispatchResult {
			let sender = match T::ProtocolOrigin::try_origin(origin) {
				Ok(_) => None,
				Err(origin) => Some(ensure_signed(origin)?),
			};

			let class_id = NextClassId::<T>::try_mutate(|id| -> Result<T::NftClassId, DispatchError> {
				let current_id = *id;
				*id = id.checked_add(&One::one()).ok_or(Error::<T>::NoAvailableClassId)?;
				Ok(current_id)
			})?;

			let metadata_bounded = Self::to_bounded_string(metadata)?;

			pallet_uniques::Pallet::<T>::do_create_class(
				class_id.into(),
				sender.clone().unwrap_or_default(),
				sender.clone().unwrap_or_default(),
				T::ClassDeposit::get(),
				false,
				pallet_uniques::Event::Created(
					class_id.into(),
					sender.clone().unwrap_or_default(),
					sender.clone().unwrap_or_default(),
				),
			)?;

			Classes::<T>::insert(
				class_id,
				ClassInfo {
					metadata: metadata_bounded,
				},
			);

			Self::deposit_event(Event::ClassCreated(sender.unwrap_or_default(), class_id));
			Ok(())
		}


	}

	impl<T: Config> Pallet<T> {
		fn to_bounded_string(name: Vec<u8>) -> Result<BoundedVec<u8, T::StringLimit>, Error<T>> {
			name.try_into().map_err(|_| Error::<T>::TooLong)
		}
	}	
}


