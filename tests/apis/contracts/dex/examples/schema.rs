use abstract_dex_api::contract::DexApi;
use abstract_dex_api::msg::SimulateSwapResponse;
use cosmwasm_schema::{export_schema_with_title, remove_schemas, schema_for};
use std::env::current_dir;
use std::fs::create_dir_all;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    DexApi::export_schema(&out_dir);
    export_schema_with_title(&schema_for!(SimulateSwapResponse), &out_dir, "ApiResponse");
}
