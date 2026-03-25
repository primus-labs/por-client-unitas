#![no_main]
sp1_zkvm::entrypoint!(main);

use common::adapter::{run, ReadResult, ZkvmAdapter};
use common::structs::PublicValuesStruct;
use std::collections::HashMap;

struct Sp1Adapter;
impl ZkvmAdapter for Sp1Adapter {
    fn read() -> ReadResult {
        let config_data: String = sp1_zkvm::io::read();
        let attestations: HashMap<String, String> = sp1_zkvm::io::read();
        ReadResult {
            config_data,
            attestations,
        }
    }
    fn commit(pv: &PublicValuesStruct) {
        sp1_zkvm::io::commit(pv);
    }
}

pub fn main() {
    run::<Sp1Adapter>();
}
