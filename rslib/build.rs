// Copyright: Ankitects Pty Ltd and contributors
// License: GNU AGPL, version 3 or later; http://www.gnu.org/licenses/agpl.html

mod rust_interface;

use std::fs;
use std::path::Path;

use anki_proto_gen::descriptors_path;
use anyhow::Result;
use prost_reflect::DescriptorPool;

fn main() -> Result<()> {
    println!("cargo:rerun-if-changed=../out/buildhash");
    let buildhash = fs::read_to_string("../out/buildhash").unwrap_or_default();
    println!("cargo:rustc-env=BUILDHASH={buildhash}");

    let descriptors_path = descriptors_path();
    println!("cargo:rerun-if-changed={}", descriptors_path.display());
    let pool = DescriptorPool::decode(std::fs::read(descriptors_path)?.as_ref())?;
    rust_interface::write_rust_interface(&pool)?;

    generate_speedrun_seed_media()?;
    Ok(())
}

/// Bundle the Speedrun seed images (`seed_data/mcat_ch4_39_media/*.png`) into a
/// generated `SEED_MEDIA` table that `anki::speedrun::seed` copies into each
/// collection's media folder on seeding. Regenerated whenever a PNG is added,
/// removed or changed, so dropping a file into the folder and rebuilding ships
/// and installs it. An empty or missing folder yields an empty table.
fn generate_speedrun_seed_media() -> Result<()> {
    let manifest_dir = std::env::var("CARGO_MANIFEST_DIR")?;
    let out_dir = std::env::var("OUT_DIR")?;
    let media_dir = Path::new(&manifest_dir).join("src/speedrun/seed_data/mcat_ch4_39_media");
    println!("cargo:rerun-if-changed={}", media_dir.display());

    let mut pngs: Vec<(String, String)> = Vec::new();
    if media_dir.is_dir() {
        for entry in fs::read_dir(&media_dir)? {
            let path = entry?.path();
            let is_png = path
                .extension()
                .and_then(|ext| ext.to_str())
                .is_some_and(|ext| ext.eq_ignore_ascii_case("png"));
            if !is_png {
                continue;
            }
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .expect("seed media filename is valid utf-8")
                .to_string();
            println!("cargo:rerun-if-changed={}", path.display());
            // Forward slashes work in `include_bytes!` on every platform, so we
            // sidestep escaping backslashes into the generated literal.
            pngs.push((name, path.to_string_lossy().replace('\\', "/")));
        }
    }
    pngs.sort();

    let mut src = String::from("pub(crate) static SEED_MEDIA: &[(&str, &[u8])] = &[\n");
    for (name, abs_path) in &pngs {
        src.push_str(&format!("    ({name:?}, include_bytes!({abs_path:?})),\n"));
    }
    src.push_str("];\n");
    fs::write(Path::new(&out_dir).join("speedrun_seed_media.rs"), src)?;
    Ok(())
}
