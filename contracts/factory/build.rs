use std::env;
use std::fs;
use std::path::PathBuf;

fn main() {
    let manifest = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    // Workspace root is two levels up from contracts/factory
    let workspace_root = manifest.join("../..").canonicalize().unwrap();
    let target_release = workspace_root.join("target/wasm32v1-none/release");

    let amm_src = target_release.join("amm.wasm");
    let token_src = target_release.join("token.wasm");

    let dest_dir = manifest.join("src");
    let amm_dest = dest_dir.join("amm.wasm");
    let token_dest = dest_dir.join("token.wasm");

    // Copy if sources exist; otherwise emit helpful note and continue so compile errors show.
    if amm_src.exists() {
        fs::copy(&amm_src, &amm_dest).expect("failed to copy amm.wasm to src/");
    }
    if token_src.exists() {
        fs::copy(&token_src, &token_dest).expect("failed to copy token.wasm to src/");
    }
}
