use base64::{engine::general_purpose, Engine as _};
use sevenz_rust::lzma::*;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::path::PathBuf;

fn main() -> Result<(), anyhow::Error> {
    let profile = env::var("TRUNK_PROFILE")?;
    if profile != "release" {
        println!("Profile is {profile}. Doing nothing");
        return Ok(());
    }
    let compress_wasm = env::args().any(|x| x.eq_ignore_ascii_case("COMPRESS_WASM"));
    if compress_wasm {
        go_compressed_wasm()
    } else {
        go_no_compress()
    }
}

fn go_compressed_wasm() -> Result<(), anyhow::Error> {
    println!("Compress Wasm");

    let Files {
        wasm_file_path,
        html_file_path,
        js_file_path,
    } = Files::try_new()?;

    let wasm_data = fs::read(wasm_file_path.clone())?;
    //let wasm_data = "Hello world".as_bytes();

    println!("Wasm data is {} bytes", wasm_data.len());
    let mut compressed_wasm: Vec<u8> = vec![];

    let mut lzma_writer = LZMAWriter::new_use_header(
        CountingWriter::new(&mut compressed_wasm),
        &LZMA2Options::with_preset(9),
        None,
    )
    .unwrap();
    lzma_writer.write_all(&wasm_data)?;
    lzma_writer.finish()?;

    println!("Compressed Wasm data is {} bytes", compressed_wasm.len());
    //println!("{compressed_wasm:?}");
    let encoded_wasm_bytes = base91::slice_encode(&compressed_wasm);

    let encoded_wasm = String::from_utf8_lossy(&encoded_wasm_bytes);

    println!(
        "Wasm data encodes to {} base91 chars",
        encoded_wasm.chars().count()
    );

    let wasm_file_name = wasm_file_path
        .file_name()
        .expect("wasm file should have name")
        .to_str()
        .unwrap();

    let js_file_name = js_file_path
        .file_name()
        .expect("js file should have name")
        .to_str()
        .unwrap();

    let js_minified_text = {
        let mut js_text = fs::read_to_string(js_file_path.clone())?;
        js_text = js_text.replace("export { initSync }", "");
        js_text = js_text.replace("export default __wbg_init;", "");
        js_text = js_text.replace(
            format!("input = new URL('{wasm_file_name}', import.meta.url);").as_str(),
            "",
        );

        println!("Js Text is {} chars", js_text.len());
        use minify_js::{Session, TopLevelMode};
        let session = Session::new();
        let mut js_out = Vec::new();

        let minify_result = minify_js::minify(
            &session,
            TopLevelMode::Global,
            js_text.as_bytes(),
            &mut js_out,
        );
        println!("Js Text is {} minified chars", js_out.len());
        match minify_result {
            Ok(()) => {
                //fs::write(js_file_path, js_out)?;
            }
            Err(e) => {
                anyhow::bail!(e.to_string());
            }
        }
        let jmt = String::from_utf8(js_out)?;
        jmt
    };

    let mut html_text = fs::read_to_string(html_file_path.clone())?;

    html_text = html_text.replace(format!(
        r#"<link rel="preload" href="/{wasm_file_name}" as="fetch" type="application/wasm" crossorigin="">"#
    ).as_str(), "");

    let base_91_js = include_str!("base91.js");
    let lzma_js = include_str!("lzma.js");

    let rep2: String = format!(
        r#"
    <script>{js_minified_text} </script>
    <script>{base_91_js} </script>
    <script>{lzma_js} </script>

    <script>const data = '{encoded_wasm}'; </script>
    <script type="module">

        const decoded = decode(data);
        let start = Date.now();
        const inflated = LZMA.decompressFile(decoded.buffer);
        console.info("Decompression time: " + (Date.now() - start) + " milliseconds");
        __wbg_init(inflated.buffers[0]);
    </script>
    "#
    );

    html_text = html_text.replace(format!(
        r#"<script type="module">import init from '/{js_file_name}';init('/{wasm_file_name}');</script>"#
    ).as_str(), &rep2);

    html_text = html_text.replace(
        format!(r#"<link rel="modulepreload" href="/{js_file_name}">"#).as_str(),
        "",
    );

    fs::write(html_file_path, html_text.as_bytes())?;

    fs::remove_file(wasm_file_path.clone())?;
    fs::remove_file(js_file_path.clone())?;
    Ok(())
}

fn go_no_compress() -> Result<(), anyhow::Error> {
    println!("Don't compress wasm");

    let Files {
        wasm_file_path,
        html_file_path,
        js_file_path,
    } = Files::try_new()?;

    let wasm_data = fs::read(wasm_file_path.clone())?;
    //let wasm_data = "Hello world".as_bytes();

    println!("Wasm data is {} bytes", wasm_data.len());
    let encoded_wasm = general_purpose::STANDARD.encode(wasm_data);

    println!(
        "Wasm data encodes to {} base64 chars",
        encoded_wasm.chars().count()
    );

    let wasm_file_name = wasm_file_path
        .file_name()
        .expect("wasm file should have name")
        .to_str()
        .unwrap();

    let js_file_name = js_file_path
        .file_name()
        .expect("js file should have name")
        .to_str()
        .unwrap();

    let js_minified_text = {
        let mut js_text = fs::read_to_string(js_file_path.clone())?;
        js_text = js_text.replace("export { initSync }", "");
        js_text = js_text.replace("export default __wbg_init;", "");
        js_text = js_text.replace(
            format!("input = new URL('{wasm_file_name}', import.meta.url);").as_str(),
            "",
        );

        println!("Js Text is {} chars", js_text.len());
        use minify_js::{Session, TopLevelMode};
        let session = Session::new();
        let mut js_out = Vec::new();

        let minify_result = minify_js::minify(
            &session,
            TopLevelMode::Global,
            js_text.as_bytes(),
            &mut js_out,
        );
        println!("Js Text is {} minified chars", js_out.len());
        match minify_result {
            Ok(()) => {
                //fs::write(js_file_path, js_out)?;
            }
            Err(e) => {
                anyhow::bail!(e.to_string());
            }
        }
        let jmt = String::from_utf8(js_out)?;
        jmt
    };

    let mut html_text = fs::read_to_string(html_file_path.clone())?;

    html_text = html_text.replace(format!(
        r#"<link rel="preload" href="/{wasm_file_name}" as="fetch" type="application/wasm" crossorigin="">"#
    ).as_str(), "");

    let rep2: String = format!(
        r#"
    <script>{js_minified_text} </script>

    <script>const data = '{encoded_wasm}'; </script>
    <script type="module">

        const binary_string = atob(data);
        var len = binary_string.length;
        var bytes = new Uint8Array(len);
        for (var i = 0; i < len; i++) {{
            bytes[i] = binary_string.charCodeAt(i);
        }}
        __wbg_init(bytes.buffer);
    </script>
    "#
    );

    html_text = html_text.replace(format!(
        r#"<script type="module">import init from '/{js_file_name}';init('/{wasm_file_name}');</script>"#
    ).as_str(), &rep2);

    html_text = html_text.replace(
        format!(r#"<link rel="modulepreload" href="/{js_file_name}">"#).as_str(),
        "",
    );

    fs::write(html_file_path, html_text.as_bytes())?;

    fs::remove_file(wasm_file_path.clone())?;
    fs::remove_file(js_file_path.clone())?;
    Ok(())
}

struct Files {
    wasm_file_path: PathBuf,
    html_file_path: PathBuf,
    js_file_path: PathBuf,
}

impl Files {
    pub fn try_new() -> Result<Self, anyhow::Error> {
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

        Ok(Self {
            wasm_file_path,
            html_file_path,
            js_file_path,
        })
    }
}

#[cfg(test)]
mod tests {
    use std::{
        fs,
        io::{Read, Write},
        time::Instant,
    };

    use sevenz_rust::lzma::*;

    #[test]
    #[ignore]
    pub fn test_different_compression_levels() -> Result<(), anyhow::Error> {
        let wasm_data =
            fs::read(r#"C:\Source\rust\steks\steks_ad\dist\steks_ad-9ae83a620010ed5_bg.wasm"#)?;
        println!("Wasm data is {} bytes", wasm_data.len());
        for preset in 0..=9 {
            let mut compressed_wasm: Vec<u8> = vec![];

            let mut lzma_writer = LZMAWriter::new_use_header(
                CountingWriter::new(&mut compressed_wasm),
                &LZMA2Options::with_preset(preset),
                None,
            )
            .unwrap();
            lzma_writer.write_all(&wasm_data)?;
            lzma_writer.finish()?;

            //println!("{compressed_wasm:?}");

            let mut reader = LZMAReader::new_mem_limit(compressed_wasm.as_slice(), u32::MAX, None)?;

            let mut decompressed = Vec::new();
            let now = Instant::now();
            reader.read_to_end(&mut decompressed)?;

            assert_eq!(wasm_data, decompressed);
            let elapsed = now.elapsed();

            let encoded_wasm_bytes = base91::slice_encode(&compressed_wasm);

            let encoded_wasm = String::from_utf8_lossy(&encoded_wasm_bytes);
            println!("Preset {preset:2} Compressed Wasm data is {bytes} bytes, encoding to {chars} base91 chars decode time: {elapsed}ms", bytes = compressed_wasm.len(), chars = encoded_wasm.chars().count(), elapsed = elapsed.as_millis());
        }

        Ok(())
    }

    #[test]
    pub fn test_lzma() -> Result<(), anyhow::Error> {
        let wasm_data = "Hello, world.".as_bytes();

        println!("Wasm data is {} bytes", wasm_data.len());
        let mut compressed_wasm: Vec<u8> = vec![];

        let mut lzma_writer = LZMAWriter::new_use_header(
            CountingWriter::new(&mut compressed_wasm),
            &LZMA2Options::with_preset(9),
            None,
        )
        .unwrap();
        lzma_writer.write_all(wasm_data)?;

        lzma_writer.finish()?;

        println!("Compressed Wasm data is {} bytes", compressed_wasm.len());
        println!("{compressed_wasm:?}");

        Ok(())
    }
}
