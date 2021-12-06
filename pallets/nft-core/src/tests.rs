use frame_support::{assert_noop, assert_ok, error::BadOrigin};

// use crate::types::ClassType;

use super::*;
use mock::*;
use pallet_uniques as UNQ;

type NFTCore = Pallet<Test>;

/// Turns a string into a BoundedVec
fn stb(s: &str) -> BoundedVec<u8, ValueLimit> {
	s.as_bytes().to_vec().try_into().unwrap()
}

/// Turns a string into a Vec
fn stv(s: &str) -> Vec<u8> {
	s.as_bytes().to_vec()
}

#[test]
fn create_collection_works() {
	ExtBuilder::default().build().execute_with(|| {
        let metadata = stv("testing");
		assert_ok!(NFTCore::create_collection(Origin::signed(ALICE), metadata.clone()));
		assert_noop!(
			NFTCore::create_collection(
				Origin::signed(ALICE),
				vec![0; <Test as UNQ::Config>::StringLimit::get() as usize + 1]
			),
			Error::<Test>::TooLong
		);
		NextCollectionId::<Test>::mutate(|id| *id = <Test as UNQ::Config>::ClassId::max_value());
		assert_noop!(
			NFTCore::create_collection(Origin::signed(ALICE), metadata.clone()),
			Error::<Test>::NoAvailableCollectionId
		);        
	});
}

#[test]
fn mint_nft_works() {
	ExtBuilder::default().build().execute_with(|| {
		assert_ok!(NFTCore::create_collection(Origin::signed(ALICE), b"metadata".to_vec()));
		assert_ok!(NFTCore::mint_nft(
			Origin::signed(ALICE),
			0,
			Some(ALICE),
			Some(0),
			Some(b"metadata".to_vec())
		));
		assert_ok!(NFTCore::mint_nft(
			Origin::signed(ALICE),
			COLLECTION_ID_0,
			Some(ALICE),
			Some(20),
			Some(b"metadata".to_vec())
		));       
        assert_ok!(NFTCore::mint_nft(
            Origin::signed(BOB),
            COLLECTION_ID_0,
            Some(CHARLIE),
            Some(20),
            Some(b"metadata".to_vec())
        ));
		assert_noop!(
			NFTCore::mint_nft(
				Origin::signed(ALICE),
				NOT_EXISTING_CLASS_ID,
				Some(CHARLIE),
				Some(20),
				Some(b"metadata".to_vec())
			),
			Error::<Test>::CollectionUnknown
		);        
	});
}