pub struct ReadResult {
    pub config_data: String,
    pub attestations: std::collections::HashMap<String, String>,
}
pub trait ZkvmAdapter {
    fn read() -> ReadResult;
    fn commit(pv: &crate::structs::PublicValuesStruct);
}

pub fn run<A: ZkvmAdapter>() {
    let mut pv = crate::structs::PublicValuesStruct::default();
    pv.version = "0.1.0".to_string();
    pv.kind = "asset-balance".to_string();

    let inp = A::read();
    if let Err(e) = crate::core::app_main(&mut pv, &inp.config_data, &inp.attestations) {
        println!("Error: {} {}", e.icode(), e.msg());
        pv.status = e.icode();
    } else {
        println!("OK");
    }

    A::commit(&pv);
}
