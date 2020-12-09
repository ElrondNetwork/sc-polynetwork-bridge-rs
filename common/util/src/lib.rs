#![no_std]

use elrond_wasm::Box;

pub const ETH_ADDRESS_LEN: usize = 20;
pub const POLYCHAIN_PUBKEY_LEN: usize = 67;
pub const POLYCHAIN_SIGNATURE_LEN: usize = 65;

pub struct EthAddress(Box<[u8;ETH_ADDRESS_LEN]>);
pub struct PublicKey(Box<[u8;POLYCHAIN_PUBKEY_LEN]>);
pub struct Signature(Box<[u8;POLYCHAIN_SIGNATURE_LEN]>);

impl EthAddress {
    pub fn as_slice(&self) -> &[u8] {
        &(*self.0)[..]
    }
}

impl PublicKey {
    pub fn as_slice(&self) -> &[u8] {
        &(*self.0)[..]
    }
}

impl Signature {
    pub fn as_slice(&self) -> &[u8] {
        &(*self.0)[..]
    }
}

impl<'a> From<&'a [u8]> for EthAddress {
	#[inline]
	fn from(byte_slice: &'a [u8]) -> Self {
        let mut data = [0u8; ETH_ADDRESS_LEN];

        if byte_slice.len() >= ETH_ADDRESS_LEN {
            for i in 0..ETH_ADDRESS_LEN {
                data[i] = byte_slice[i];
            }
        }
        
        EthAddress(Box::from(data))
	}
}

impl<'a> From<&'a [u8]> for PublicKey {
	#[inline]
	fn from(byte_slice: &'a [u8]) -> Self {
        let mut data = [0u8; POLYCHAIN_PUBKEY_LEN];

        if byte_slice.len() >= POLYCHAIN_PUBKEY_LEN {
            for i in 0..POLYCHAIN_PUBKEY_LEN {
                data[i] = byte_slice[i];
            }
        }
        
        PublicKey(Box::from(data))
	}
}

impl<'a> From<&'a [u8]> for Signature {
	#[inline]
	fn from(byte_slice: &'a [u8]) -> Self {
        let mut data = [0u8; POLYCHAIN_SIGNATURE_LEN];

        if byte_slice.len() >= POLYCHAIN_SIGNATURE_LEN {
            for i in 0..POLYCHAIN_SIGNATURE_LEN {
                data[i] = byte_slice[i];
            }
        }
        
        Signature(Box::from(data))
	}
}