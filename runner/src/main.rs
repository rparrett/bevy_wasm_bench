use std::fs::File;
use std::io::{BufReader, Write};
use std::process::Stdio;
use std::time::{Duration, Instant};
use std::{os::unix::fs::MetadataExt, path::Path, process::Command};

use flate2::write::GzEncoder;
use flate2::Compression;
use options::*;

use anyhow::{Context, Result};
use itertools::iproduct;
use size::Size;
use strum::IntoEnumIterator;

mod options;

const PROFILE: &str = "bevy_wasm_bench";
const NAME: &str = "bevy_wasm_bench";
const OUT_DIR: &str = "web";

fn main() -> Result<()> {
    check_all_deps(&[
        "cargo",
        "wasm-opt",
        "basic-http-server",
        "wasm-bindgen",
        "node",
    ])?;
    println!();

    let workspace_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../");
    let wasm_path = format!("target/wasm32-unknown-unknown/{}/bench.wasm", PROFILE);

    std::env::set_current_dir(workspace_dir)?;

    std::fs::create_dir_all("web/assets")?;
    std::fs::copy("bench/assets/icon.png", "web/assets/icon.png")?;

    let mut csv = File::create("out.csv")?;
    writeln!(
        csv,
        "opt_level,wasm_opt,lto,codegen_units,build_time,wasm_opt_time,frame_time\n"
    )?;

    for (opt_level, lto, codegen_units) in
        iproduct!(OptLevel::iter(), Lto::iter(), CodegenUnits::iter())
    {
        // Create cargo options

        let options_toml = [opt_level.option(), lto.option(), codegen_units.option()].join("\n");

        std::fs::create_dir_all(".cargo")?;
        std::fs::write(
            ".cargo/config.toml",
            format!(
                "[profile.{}]\ninherits = \"release\"\n{}",
                PROFILE, options_toml
            ),
        )?;

        // Clean

        println!("Cleaning up.");

        Command::new("cargo")
            .arg("clean")
            .output()
            .context("Running cargo clean")?;

        // Build wasm

        println!(
            "Building with OptLevel::{:?}, Lto::{:?}, CodegenUnits::{:?}",
            opt_level, lto, codegen_units
        );

        let now = Instant::now();

        Command::new("cargo")
            .arg("build")
            .arg("-p")
            .arg("bench")
            .arg("--target=wasm32-unknown-unknown")
            .args(["--profile", PROFILE])
            .output()
            .context("Running cargo build")?;

        let build_time = now.elapsed();

        for wasm_opt in WasmOpt::iter() {
            // Bindgen

            println!("Running bindgen.");

            Command::new("wasm-bindgen")
                .args([
                    "--out-name",
                    NAME,
                    "--out-dir",
                    OUT_DIR,
                    "--target",
                    "web",
                    &wasm_path,
                ])
                .output()
                .context("Running wasm-bindgen")?;

            println!("Running wasm-opt with WasmOpt::{:?}", wasm_opt);

            let bindgen_wasm_path = format!("{}/{}_bg.wasm", OUT_DIR, NAME);

            let now = Instant::now();

            if wasm_opt.enabled() {
                Command::new("wasm-opt")
                    .args(wasm_opt.args())
                    .arg(&bindgen_wasm_path)
                    .args(["-o", &bindgen_wasm_path])
                    .output()
                    .context("Running wasm-opt")?;
            }

            let wasm_opt_time = if wasm_opt.enabled() {
                now.elapsed()
            } else {
                Duration::default()
            };

            let attr = std::fs::metadata(&bindgen_wasm_path)?;

            // gzip to measure resulting filesize

            println!("Compressing.");

            let compressed_path = format!("{}.gz", bindgen_wasm_path);

            compress(&bindgen_wasm_path, &compressed_path).context("Compressing wasm")?;

            let attr_gz = std::fs::metadata(format!("{}.gz", bindgen_wasm_path))?;

            println!(
                "{} ({} gzipped)",
                Size::from_bytes(attr.size()),
                Size::from_bytes(attr_gz.size())
            );
            println!("{:.2?} (+{:.2?} wasm-opt)", build_time, wasm_opt_time);

            println!("Testing runtime performance.");

            let mut h = Command::new("basic-http-server")
                .stdout(Stdio::null())
                .stderr(Stdio::null())
                .arg("web")
                .arg("-a")
                .arg("127.0.0.1:1334")
                .spawn()
                .context("Starting web server")?;

            // Run Puppeteer, collect output

            let puppeteer_out = Command::new("node")
                .arg("index.js")
                .output()
                .context("Running puppeteer")?;

            let frame_time = String::from_utf8(puppeteer_out.stdout)
                .context("Building utf8 from puppeteer output")?
                .trim()
                .parse::<f32>()
                .context("Parsing puppeteer output")?;

            println!("{:2}ms", frame_time);

            h.kill().context("Killing web server")?;

            println!();

            writeln!(
                csv,
                "{:?},{:?},{:?},{:?},{:?},{:?},{:?}",
                opt_level,
                wasm_opt,
                lto,
                codegen_units,
                build_time.as_secs_f32(),
                wasm_opt_time.as_secs_f32(),
                frame_time
            )
            .context("Writing to out.csv")?;
        }
    }

    Ok(())
}

fn check_all_deps(deps: &[&str]) -> Result<()> {
    let mut failed = false;

    for dep in deps {
        let output = Command::new(dep).arg("--help").output();

        match output {
            Ok(_output) => {
                println!("Checking for {dep} in PATH: ✅");
            }
            Err(e) if matches!(e.kind(), std::io::ErrorKind::NotFound) => {
                eprintln!("Checking for {dep} in PATH: ❌");
                failed = true;
            }
            Err(e) => {
                eprintln!("Checking for {dep} in PATH: ❌");
                panic!("Unknown IO error: {:?}", e);
            }
        }
    }

    if failed {
        anyhow::bail!("Missing required program(s)".to_string());
    }

    Ok(())
}

fn compress<P>(input_path: P, output_path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let mut input = BufReader::new(File::open(input_path)?);
    let output = File::create(output_path)?;
    let mut encoder = GzEncoder::new(output, Compression::default());

    std::io::copy(&mut input, &mut encoder)?;
    let _ = encoder.finish()?;

    Ok(())
}
