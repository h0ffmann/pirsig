//! `pirsig-art`: writes docs/drawings/pirsig.svg and its Desmos state, docs/drawings/pirsig.desmos.json.
//!
//! ```console
//! cargo run -q -p pirsig-drawings --bin pirsig-art > docs/drawings/pirsig.svg
//! cargo run -q -p pirsig-drawings --bin pirsig-art desmos > docs/drawings/pirsig.desmos.json
//! cargo run -q -p pirsig-drawings --bin pirsig-art check > /tmp/pirsig-check.svg   # to look at
//! ```

mod pen;
mod pirsig;

fn main() -> Result<(), String> {
    let art = pirsig::drawing();
    let out = match std::env::args().nth(1).as_deref() {
        Some("desmos") => serde_json::to_string_pretty(&art.desmos()).map_err(|e| e.to_string())?,
        Some("check") => art.check_svg()?,
        None => art.svg(),
        Some(other) => {
            return Err(format!(
                "unknown argument {other}: try `desmos`, `check` or nothing"
            ))
        }
    };
    println!("{out}");
    Ok(())
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_committed_files_are_the_output() {
        let art = super::pirsig::drawing();
        let dir = concat!(env!("CARGO_MANIFEST_DIR"), "/../../docs/drawings/");
        let svg = std::fs::read_to_string(format!("{dir}pirsig.svg")).expect("pirsig.svg");
        let json = std::fs::read_to_string(format!("{dir}pirsig.desmos.json"))
            .expect("pirsig.desmos.json");
        assert_eq!(
            svg,
            art.svg() + "\n",
            "run `just`-less: see docs/drawings/README.md"
        );
        assert_eq!(
            json,
            serde_json::to_string_pretty(&art.desmos()).expect("json") + "\n"
        );
    }

    #[test]
    fn the_desmos_file_is_the_drawing() {
        let worst = super::pirsig::drawing()
            .check()
            .expect("every expression reads back");
        assert!(worst < 0.01, "an expression strays {worst} from its line");
    }
}
