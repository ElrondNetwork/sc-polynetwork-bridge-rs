#![no_std]

use elrond_wasm::{api::BigUintApi, elrond_codec::*};
use elrond_wasm::types::{BoxedBytes, TokenIdentifier};

use zero_copy_sink::*;
use zero_copy_source::*;

elrond_wasm::derive_imports!();

#[derive(TypeAbi)]
pub struct EsdtPayment<BigUint: BigUintApi> {
    pub sender: BoxedBytes,
    pub receiver: BoxedBytes,
    pub token_id: TokenIdentifier,
    pub amount: BigUint,
}

impl<BigUint: BigUintApi> NestedEncode for EsdtPayment<BigUint> {
    fn dep_encode<O: NestedEncodeOutput>(&self, dest: &mut O) -> Result<(), EncodeError> {
        let mut sink = ZeroCopySink::new();

        sink.write_var_bytes(self.sender.as_slice());
        sink.write_var_bytes(self.receiver.as_slice());
        sink.write_var_bytes(self.token_id.as_esdt_identifier());
        sink.write_var_bytes(self.amount.to_bytes_be().as_slice());

        dest.write(sink.get_sink().as_slice());

        Ok(())
    }
}

impl<BigUint: BigUintApi> NestedDecode for EsdtPayment<BigUint> {
    fn dep_decode<I: NestedDecodeInput>(input: &mut I) -> Result<Self, DecodeError> {
        let mut source = ZeroCopySource::new(input.flush());

        let sender;
        let receiver;
        let token_identifier;
        let amount;

        match source.next_var_bytes() {
            Some(val) => sender = val,
            None => return Err(DecodeError::INPUT_TOO_SHORT),
        };

        match source.next_var_bytes() {
            Some(val) => receiver = val,
            None => return Err(DecodeError::INPUT_TOO_SHORT),
        };

        match source.next_var_bytes() {
            Some(val) => token_identifier = val,
            None => return Err(DecodeError::INPUT_TOO_SHORT),
        };

        match source.next_var_bytes() {
            Some(val) => amount = BigUint::from_bytes_be(val.as_slice()),
            None => return Err(DecodeError::INPUT_TOO_SHORT),
        };

        return Ok(EsdtPayment {
            sender,
            receiver,
            token_id: TokenIdentifier::from(token_identifier),
            amount,
        });
    }
}

impl<BigUint: BigUintApi> TopEncode for EsdtPayment<BigUint> {
    #[inline]
    fn top_encode<O: TopEncodeOutput>(&self, output: O) -> Result<(), EncodeError> {
        top_encode_from_nested(self, output)
    }
}

impl<BigUint: BigUintApi> TopDecode for EsdtPayment<BigUint> {
    fn top_decode<I: TopDecodeInput>(input: I) -> Result<Self, DecodeError> {
        top_decode_from_nested(input)
    }
}
