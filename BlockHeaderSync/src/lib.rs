#![no_std]

use header::peer_config::*;
use header::*;

use util::*;

use public_key::*;
use signature::*;

elrond_wasm::imports!();

#[elrond_wasm_derive::contract(BlockHeaderSyncImpl)]
pub trait BlockHeaderSync {
    #[init]
    fn init(&self) {}

    // endpoints

    #[endpoint(syncGenesisHeader)]
    fn sync_genesis_header(&self, header: Header) -> SCResult<()> {
        require!(
            self.genesis_header().is_empty(),
            "Genesis header already set!"
        );
        require!(
            !header.consensus_payload.is_some(),
            "Invalid genesis header!"
        );

        let sc_result = self.update_consensus_peer(&header);
        if sc_result.is_ok() {
            self.genesis_header().set(&header);
            self.store_header(&header);

            self.block_header_sync_event(&header);
        }

        sc_result
    }

    #[endpoint(syncBlockHeader)]
    fn sync_block_header(&self, header: Header) -> SCResult<()> {
        if self
            .header_by_height(header.chain_id, header.height)
            .is_empty()
        {
            match self.verify_header(&header) {
                Ok(()) => {}
                Err(err) => return Err(err),
            };

            let sc_result = self.update_consensus_peer(&header);
            if sc_result.is_ok() {
                self.store_header(&header);
                self.block_header_sync_event(&header);
            }

            return sc_result;
        }

        // if block exists already, no sync needed
        Ok(())
    }

    #[view(getHeaderByHeight)]
    fn get_header_by_height_endpoint(&self, chain_id: u64, height: u32) -> OptionalResult<Header> {
        if !self.header_by_height(chain_id, height).is_empty() {
            OptionalResult::Some(self.header_by_height(chain_id, height).get())
        } else {
            OptionalResult::None
        }
    }

    #[view(getHeaderByHash)]
    fn get_header_by_hash_endpoint(&self, chain_id: u64, hash: &H256) -> OptionalResult<Header> {
        if !self.header_by_hash(chain_id, hash).is_empty() {
            OptionalResult::Some(self.header_by_hash(chain_id, hash).get())
        } else {
            OptionalResult::None
        }
    }

    #[endpoint(verifyHeader)]
    fn verify_header(&self, header: &Header) -> SCResult<()> {
        let chain_id = header.chain_id;
        let height = header.height;

        let key_height = match self.find_key_height(chain_id, height) {
            Some(k) => k,
            None => return sc_error!("Couldn't find key height!"),
        };
        let prev_consensus = self.consensus_peers(chain_id, key_height).get();

        if header.book_keepers.len() * 3 < prev_consensus.len() * 2 {
            return sc_error!("Header bookkeepers num must be > 2/3 of consensus num");
        }

        for bk in &header.book_keepers {
            let mut serialized_key = Vec::new();
            let _ = bk.dep_encode(&mut serialized_key);
            let key_id = hex_converter::byte_slice_to_hex(&serialized_key);

            // if key doesn't exist, something is wrong
            if !prev_consensus.iter().any(|p| p.id == key_id) {
                return sc_error!("Invalid pubkey!");
            }
        }

        let hashed_header = BoxedBytes::from(self.hash_header(header).as_bytes());

        self.verify_multi_signature(
            &hashed_header,
            &header.book_keepers,
            header.book_keepers.len(),
            &header.sig_data,
        )
    }

    // private

    fn update_consensus_peer(&self, header: &Header) -> SCResult<()> {
        if let Some(consensus_payload) = &header.consensus_payload {
            if let Some(chain_config) = &consensus_payload.new_chain_config {
                let chain_id = header.chain_id;
                let height = header.height;

                // update key heights
                self.key_height_list(chain_id).push_back(height);

                // update consensus peer list
                if !chain_config.peers.is_empty() {
                    self.consensus_peers(chain_id, height).set(&chain_config.peers);
                } else {
                    return sc_error!("Consensus peer list is empty!");
                }
            }
        }

        Ok(())
    }

    // header-related

    /// hashed twice, for some reason
    fn hash_header(&self, header: &Header) -> H256 {
        self.sha256(
            self.sha256(header.get_partial_serialized().as_slice())
                .as_bytes(),
        )
    }

    fn store_header(&self, header: &Header) {
        self.header_by_hash(header.chain_id, &header.block_hash)
            .set(header);
        self.header_by_height(header.chain_id, header.height)
            .set(header);
        self.current_height(header.chain_id).set(&header.height);
    }

    // verification-related

    /// _height_ should not be lower than current max (which should be the last element).
    /// If the list is empty (i.e. None is returned from last()),  
    /// then it means genesis header was not initialized
    fn find_key_height(&self, chain_id: u64, height: u32) -> Option<u32> {
        match self.key_height_list(chain_id).back() {
            Some(last_key_height) => {
                if last_key_height > height {
                    None
                } else {
                    Some(last_key_height)
                }
            }
            None => None,
        }
    }

    fn verify(&self, public_key: &PublicKey, data: &BoxedBytes, signature: &Signature) -> bool {
        if data.is_empty() {
            return false;
        }

        match public_key.algorithm {
            EllipticCurveAlgorithm::ECDSA => {
                match signature.scheme {
                    SignatureScheme::SM3withSM2 => {
                        // not implemented for DSA signature yet

                        self.verify_secp256k1(
                            public_key.value_as_slice(),
                            data.as_slice(),
                            signature.value_as_slice(),
                        )
                    }
                    SignatureScheme::Unknown => false,
                    _ => {
                        // not implemented yet
                        false
                    }
                }
            }
            EllipticCurveAlgorithm::SM2 => {
                if signature.scheme == SignatureScheme::SHA512withEDDSA {
                    self.verify_ed25519(
                        public_key.value_as_slice(),
                        data.as_slice(),
                        signature.value_as_slice(),
                    )
                } else {
                    false
                }
            }
            EllipticCurveAlgorithm::Unknown => false,
        }
    }

    fn verify_multi_signature(
        &self,
        data: &BoxedBytes,
        keys: &[PublicKey],
        min_sigs: usize,
        sigs: &[Signature],
    ) -> SCResult<()> {
        if sigs.len() < min_sigs {
            return sc_error!("Not enough signatures!");
        }

        let mut mask = Vec::with_capacity(keys.len());
        mask.resize(keys.len(), false);

        for sig in sigs {
            let mut valid = false;

            for j in 0..keys.len() {
                if mask[j] {
                    continue;
                }
                if self.verify(&keys[j], data, sig) {
                    mask[j] = true;
                    valid = true;

                    break;
                }
            }

            if !valid {
                return sc_error!("Multi-signature verification failed!");
            }
        }

        Ok(())
    }

    // events

    #[event("blockHeaderSyncEvent")]
    fn block_header_sync_event(&self, header: &Header);

    // storage

    #[storage_mapper("genesisHeader")]
    fn genesis_header(&self) -> SingleValueMapper<Self::Storage, Header>;

    #[storage_mapper("headerByHash")]
    fn header_by_hash(
        &self,
        chain_id: u64,
        hash: &H256,
    ) -> SingleValueMapper<Self::Storage, Header>;

    #[storage_mapper("headerByHeight")]
    fn header_by_height(
        &self,
        chain_id: u64,
        height: u32,
    ) -> SingleValueMapper<Self::Storage, Header>;

    #[view(getCurrentHeight)]
    #[storage_mapper("currentHeight")]
    fn current_height(&self, chain_id: u64) -> SingleValueMapper<Self::Storage, u32>;

    #[storage_mapper("consensusPeers")]
    fn consensus_peers(&self, chain_id: u64, height: u32) -> SingleValueMapper<Self::Storage, Vec<PeerConfig>>;

    #[storage_mapper("keyHeightList")]
    fn key_height_list(&self, chain_id: u64) -> LinkedListMapper<Self::Storage, u32>;
}
