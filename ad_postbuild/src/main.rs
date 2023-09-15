use regex::Regex;
use std::env;
use std::fs;
use std::path::Path;
use std::path::PathBuf;

fn main() -> Result<(), anyhow::Error> {
    let staging_dir = env::var("TRUNK_STAGING_DIR")?;

    let staging_dir = Path::new(staging_dir.as_str());

    let dir = fs::read_dir(staging_dir)?;

    let mut wasm_file_path: Option<PathBuf> = None;
    let mut html_file_path: Option<PathBuf> = None;
    let mut js_file_path: Option<PathBuf> = None;

    for file in dir {
        let file = file?;

        //let file_type = file.file_type()?;
        let path = file.path();

        let extension = path.extension().expect("File is missing extension");
        if extension.eq_ignore_ascii_case("wasm") {
            assert!(wasm_file_path.is_none());
            wasm_file_path = Some(path);
        } else if extension.eq_ignore_ascii_case("html") {
            assert!(html_file_path.is_none());
            html_file_path = Some(path);
        } else if extension.eq_ignore_ascii_case("js") {
            assert!(js_file_path.is_none());
            js_file_path = Some(path);
        } else {
            panic!("Unexpected file {path:?}");
        }
    }
    let wasm_file_path = wasm_file_path.expect("wasm file was missing");
    let html_file_path = html_file_path.expect("html file was missing");
    let js_file_path = js_file_path.expect("js file was missing");

    let wasm_data = fs::read(wasm_file_path.clone())?;

    println!("Wasm data is {} bytes", wasm_data.len());

    let compressed_wasm = miniz_oxide::deflate::compress_to_vec(&wasm_data, 10);
    let encoded_wasm_bytes = base91::slice_encode(&compressed_wasm);

    let encoded_wasm = String::from_utf8_lossy(&encoded_wasm_bytes);

    println!(
        "Wasm data encodes to {} base91 chars",
        encoded_wasm.chars().count()
    );

    let contents = "const data = '".to_string()
        + encoded_wasm.to_string().as_str()
        + "';\n export default data";

    fs::write(staging_dir.join("encoded_wasm.js"), contents)?;

    fs::write(
        staging_dir.join("pako_inflate.js"),
        include_bytes!("pako_inflate.js"),
    )?;

    fs::remove_file(wasm_file_path)?;
    let js_file_name = js_file_path
        .file_name()
        .expect("js file should have name")
        .to_str()
        .unwrap();

    let html_text = fs::read_to_string(html_file_path.clone())?;

    let regex1 = Regex::new(
        r#"<link rel="preload" href="/steks_ad-[a-z0-9]+_bg\.wasm" as="fetch" type="application/wasm" crossorigin="">"#,
    )?;
    let rep1 = r#"<link rel="modulepreload" href="./encoded_wasm.js"></head>"#;
    let html_text = regex1.replace(&html_text, rep1);

    let regex2 = Regex::new(
        r#"<script type="module">import init from '/steks_ad-[a-z0-9]+\.js';init\('/steks_ad-[a-z0-9]+_bg.wasm'\);</script>"#,
    )?;

    let base_91_js = include_str!("base91.js");
    let rep2: String = r#"
    <script type="text/javascript" src="pako_inflate.js"></script>
    <script type="module">
    import data from "./encoded_wasm.js"

    import init from './"#
        .to_string()
        + js_file_name
        + r#"';"#
        + base_91_js
        + r#"const decoded = decode(data);
        const inflated = pako.inflateRaw(decoded);
          init(inflated);
    </script>
    "#;
    let html_text = regex2.replace(&html_text, rep2);

    let regex3 = Regex::new(r#"<link rel="modulepreload" href="/steks_ad-[a-z0-9]+\.js"></head>"#)?;
    let rep3 = r#"<link rel="modulepreload" href="./"#.to_string() + js_file_name + r#""></head>"#;

    let html_text = regex3.replace(&html_text, rep3);

    fs::write(html_file_path, html_text.as_bytes())?;

    Ok(())
}
