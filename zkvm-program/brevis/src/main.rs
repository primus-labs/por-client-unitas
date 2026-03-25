#![no_main]
pico_sdk::entrypoint!(main);

use common::{
    adapter::{run, ReadResult, ZkvmAdapter},
    structs::PublicValuesStruct,
};
use std::collections::HashMap;

struct PicoAdapter;
impl ZkvmAdapter for PicoAdapter {
    fn read() -> ReadResult {
        let config_data: String = pico_sdk::io::read_as();
        let attestations: HashMap<String, String> = pico_sdk::io::read_as();
        ReadResult {
            config_data,
            attestations,
        }
    }
    fn commit(pv: &PublicValuesStruct) {
        let bytes = bincode::serialize(pv).unwrap();
        pico_sdk::io::commit_bytes(&bytes);
    }
}

pub fn main() {
    run::<PicoAdapter>();
}
