use anyhow::{Context, Result};
use std::{fs, path::Path};

pub fn compress_file(p: &Path) -> Result<Vec<u8>> {
    let buf = fs::read(p)
        .map_err(anyhow::Error::from)
        .with_context(|| format!("{}", p.to_string_lossy()))?
        .into_boxed_slice();
    let mut out = Vec::<u8>::new();
    let params = brotli::enc::BrotliEncoderParams {
        quality: 11,
        ..Default::default()
    };

    brotli::BrotliCompress(&mut buf.as_ref(), &mut out, &params)?;
    Ok(out)
}
