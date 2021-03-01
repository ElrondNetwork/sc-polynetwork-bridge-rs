#![no_std]

use elrond_wasm::types::{Address, BoxedBytes, H256, Vec};
use elrond_wasm::elrond_codec::*;

use util::*;
use vbft_block_info::*;
use zero_copy_sink::*;
use zero_copy_source::*;

pub mod peer_config;
pub mod chain_config;
pub mod vbft_block_info;

elrond_wasm::derive_imports!();

#[derive(TypeAbi, Debug, PartialEq)]
pub struct Header {
    pub version: u32,
    pub chain_id: u64,
    pub prev_block_hash: H256,
    pub transactions_root: H256,
    pub cross_state_root: H256,
    pub block_root: H256,
    pub timestamp: u32,
    pub height: u32,
    pub consensus_data: u64,
	pub consensus_payload: Option<VbftBlockInfo>, // This is serialized as: 
		// byte(0) if value does not exist
		// byte(1) followed by the actual value if it exists
	pub next_book_keeper: Address,

	pub book_keepers: Vec<PublicKey>,
	pub sig_data: Vec<Signature>,
	pub block_hash: H256
}

impl Header {
	pub fn is_start_of_epoch(&self) -> bool {
		self.height % POLYCHAIN_EPOCH_HEIGHT == 0
	}

	pub fn get_partial_serialized(&self) -> BoxedBytes {
		self.serialize_partial().get_sink()
	}

	pub fn decode_from_source(source: &mut ZeroCopySource) -> Result<Self, DecodeError> {
		let version;
		let chain_id;
		let prev_block_hash;
		let transactions_root;
		let cross_state_root;
		let block_root;
		let timestamp;
		let height;
		let consensus_data;
		let consensus_payload;
		let next_book_keeper;
		let mut book_keepers = Vec::new();
		let mut sig_data = Vec::new();
		let block_hash;

		match source.next_u32() {
			Some(val) => version = val,
			None => return Err(DecodeError::INPUT_TOO_SHORT)
		};

		match source.next_u64() {
			Some(val) => chain_id = val,
			None => return Err(DecodeError::INPUT_TOO_SHORT)
		};

		match source.next_hash() {
			Some(val) => prev_block_hash = val,
			None => return Err(DecodeError::INPUT_TOO_SHORT)
		};

		match source.next_hash() {
			Some(val) => transactions_root = val,
			None => return Err(DecodeError::INPUT_TOO_SHORT)
		};

		match source.next_hash() {
			Some(val) => cross_state_root = val,
			None => return Err(DecodeError::INPUT_TOO_SHORT)
		};

		match source.next_hash() {
			Some(val) => block_root = val,
			None => return Err(DecodeError::INPUT_TOO_SHORT)
		};

		match source.next_u32() {
			Some(val) => timestamp = val,
			None => return Err(DecodeError::INPUT_TOO_SHORT)
		};

		match source.next_u32() {
			Some(val) => height = val,
			None => return Err(DecodeError::INPUT_TOO_SHORT)
		};

		match source.next_u64() {
			Some(val) => consensus_data = val,
			None => return Err(DecodeError::INPUT_TOO_SHORT)
		};

		match source.next_u8() {
			Some(val) => {
				if val == 0 {
					consensus_payload = None;
				}
				else if val == 1 {
					match VbftBlockInfo::decode_from_source(source) {
						Ok(bi) => consensus_payload = Some(bi),
						Err(err) => return Err(err)
					}
				}
				else {
					return Err(DecodeError::INVALID_VALUE);
				}
			},
			None => return Err(DecodeError::INPUT_TOO_SHORT)
		}

		match source.next_address() {
			Some(val) => next_book_keeper = val,
			None => return Err(DecodeError::INPUT_TOO_SHORT)
		};

		match source.next_var_uint() {
			Some(len) => {
				for _ in 0..len {
					match source.next_public_key() {
						Some(val) => book_keepers.push(val),
						None => return Err(DecodeError::INPUT_TOO_SHORT)
					}
				}
			},
			None => return Err(DecodeError::INPUT_TOO_SHORT)
		}

		match source.next_var_uint() {
			Some(len) => {
				for _ in 0..len {
					match source.next_signature() {
						Some(val) => sig_data.push(val),
						None => return Err(DecodeError::INPUT_TOO_SHORT)
					}
				}
			},
			None => return Err(DecodeError::INPUT_TOO_SHORT)
		}

		match source.next_hash() {
			Some(val) => block_hash = val,
			None => return Err(DecodeError::INPUT_TOO_SHORT)
		};

		Ok(Header {
			version,
			chain_id,
			prev_block_hash,
			transactions_root,
			cross_state_root,
			block_root,
			timestamp,
			height,
			consensus_data,
			consensus_payload,
			next_book_keeper,
			book_keepers,
			sig_data,
			block_hash,
		})
	}
}

// private methods
impl Header {
	fn serialize_partial(&self) -> ZeroCopySink {
		let mut sink = ZeroCopySink::new();

		sink.write_u32(self.version);
		sink.write_u64(self.chain_id);
		sink.write_hash(&self.prev_block_hash);
		sink.write_hash(&self.transactions_root);
		sink.write_hash(&self.cross_state_root);
		sink.write_hash(&self.block_root);
		sink.write_u32(self.timestamp);
		sink.write_u32(self.height);
		sink.write_u64(self.consensus_data);

		if let Some(payload) = &self.consensus_payload {
			sink.write_u8(1);
			let _ = payload.dep_encode(&mut sink);
		}
		else {
			sink.write_u8(0);
		}

		sink.write_address(&self.next_book_keeper);

		sink
	}
}

impl NestedEncode for Header {
	fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
		let mut sink = self.serialize_partial();
		
		sink.write_var_uint(self.book_keepers.len() as u64);
		for pubkey in &self.book_keepers {
			sink.write_public_key(pubkey);
		}

		sink.write_var_uint(self.sig_data.len() as u64);
		for sig in &self.sig_data {
			sink.write_signature(sig);
		}

		sink.write_hash(&self.block_hash);

		dest.write(sink.get_sink().as_slice());

		Ok(())
	}
}

impl NestedDecode for Header {
	fn dep_decode<I: NestedDecodeInput>(input: &mut I) -> Result<Self, DecodeError> {
		let mut source = ZeroCopySource::new(input.flush());

		Self::decode_from_source(&mut source)
	}
}

impl TopEncode for Header {
	#[inline]
	fn top_encode<O: TopEncodeOutput>(&self, output: O) -> Result<(), EncodeError> {
		top_encode_from_nested(self, output)
	}
}

impl TopDecode for Header {
	fn top_decode<I: TopDecodeInput>(input: I) -> Result<Self, DecodeError> {
		top_decode_from_nested(input)
	}
}
