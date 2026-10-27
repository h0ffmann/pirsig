//! `ztk-art`: writes docs/drawings/ztk.svg and its Desmos state, docs/drawings/ztk.desmos.json.
//!
//! ```console
//! cargo run -q -p ztk-drawings --bin ztk-art > docs/drawings/ztk.svg
//! cargo run -q -p ztk-drawings --bin ztk-art desmos > docs/drawings/ztk.desmos.json
//! ```

mod pen;
mod ztk;

fn main() -> Result<(), String> {
    let art = ztk::drawing();
    let out = match std::env::args().nth(1).as_deref() {
        Some("desmos") => serde_json::to_string_pretty(&art.desmos()).map_err(|e| e.to_string())?,
        None => art.svg()?,
        Some(other) => return Err(format!("unknown argument {other}: try `desmos` or nothing")),
    };
    println!("{out}");
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_committed_files_are_the_output() {
        let art = super::ztk::drawing();
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/drawings/");
        let svg = std::fs::read_to_string(format!("{dir}ztk.svg")).expect("ztk.svg");
        let json =
            std::fs::read_to_string(format!("{dir}ztk.desmos.json")).expect("ztk.desmos.json");
        assert_eq!(
            svg,
            art.svg().expect("svg") + "\n",
            "run `just`-less: see docs/drawings/README.md"
        );
        assert_eq!(
            json,
            serde_json::to_string_pretty(&art.desmos()).expect("json") + "\n"
        );
    }
}
