//! Shrink an image until it fits a byte budget.

use std::cell::Cell;
use std::io::Cursor;
use std::time::Instant;

use image::codecs::jpeg::JpegEncoder;
use image::codecs::png::{CompressionType, FilterType as PngFilter, PngEncoder};
use image::imageops::FilterType;
use image::{
    ColorType, DynamicImage, GenericImageView, ImageDecoder, ImageEncoder, ImageReader,
    Limits, Rgb, RgbImage,
};

pub const MAX_UPLOAD_BYTES: usize = 50 * 1024 * 1024;
pub const MIN_TARGET_BYTES: usize = 8 * 1024;
pub const MAX_TARGET_BYTES: usize = 5 * 1024 * 1024;
pub(crate) const MAX_EDGE: u32 = 4096;
pub(crate) const MAX_PIXELS: u64 = 12_000_000;
pub(crate) const CPU_TIMEOUT_MSG: &str = "That image took too long. Try a smaller file.";

thread_local! {
    static CPU_DEADLINE: Cell<Option<Instant>> = const { Cell::new(None) };
}

pub(crate) fn with_cpu_deadline<R>(until: Instant, job: impl FnOnce() -> R) -> R {
    struct Clear;
    impl Drop for Clear {
        fn drop(&mut self) {
            CPU_DEADLINE.with(|d| d.set(None));
        }
    }
    CPU_DEADLINE.with(|d| d.set(Some(until)));
    let _clear = Clear;
    job()
}

pub(crate) fn cpu_deadline_hit() -> bool {
    CPU_DEADLINE.with(|d| d.get().is_some_and(|until| Instant::now() >= until))
}

pub(crate) fn check_cpu_deadline() -> Result<(), String> {
    if cpu_deadline_hit() {
        Err(CPU_TIMEOUT_MSG.into())
    } else {
        Ok(())
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutFormat {
    Jpeg,
    Webp,
    Png,
}

impl OutFormat {
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "webp" => Self::Webp,
            "png" => Self::Png,
            _ => Self::Jpeg,
        }
    }

    pub fn mime(self) -> &'static str {
        match self {
            Self::Jpeg => "image/jpeg",
            Self::Webp => "image/webp",
            Self::Png => "image/png",
        }
    }

    pub fn ext(self) -> &'static str {
        match self {
            Self::Jpeg => "jpg",
            Self::Webp => "webp",
            Self::Png => "png",
        }
    }

    fn lossy(self) -> bool {
        matches!(self, Self::Jpeg | Self::Webp)
    }

    pub fn parse_prefer_webp(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "jpeg" | "jpg" => Self::Jpeg,
            "png" => Self::Png,
            _ => Self::Webp,
        }
    }

    pub fn parse_prefer_png(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "jpeg" | "jpg" => Self::Jpeg,
            "webp" => Self::Webp,
            _ => Self::Png,
        }
    }
}

#[derive(Debug, Clone)]
pub struct CompressResult {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: OutFormat,
    pub original_bytes: usize,
    pub original_width: u32,
    pub original_height: u32,
    pub over_budget: bool,
}

pub fn clamp_target(kb: u32) -> usize {
    let bytes = (kb as usize).saturating_mul(1024);
    bytes.clamp(MIN_TARGET_BYTES, MAX_TARGET_BYTES)
}

pub fn compress(
    input: &[u8],
    target_bytes: usize,
    format: OutFormat,
) -> Result<CompressResult, String> {
    if input.is_empty() {
        return Err("Empty file.".into());
    }
    if input.len() > MAX_UPLOAD_BYTES {
        return Err("File is over 50 MB. Pick a smaller image.".into());
    }
    reject_unsupported(input)?;
    let target = target_bytes.clamp(MIN_TARGET_BYTES, MAX_TARGET_BYTES);
    let img = load_oriented(input)?;
    let (ow, oh) = img.dimensions();
    if ow == 0 || oh == 0 {
        return Err("That image has no pixels.".into());
    }
    // Always re-encode so GPS/EXIF never leave with the download.

    check_cpu_deadline()?;
    let mut work = cap_dimensions(img);
    let mut quality: u8 = 86;
    let mut best = encode_with_quality(&work, format, quality)?;
    for _ in 0..48 {
        if best.len() <= target {
            return done(&work, best, format, input.len(), ow, oh, false);
        }
        if cpu_deadline_hit() {
            return done(&work, best, format, input.len(), ow, oh, true);
        }
        if format.lossy() && quality > 40 {
            quality = quality.saturating_sub(8);
            best = encode_with_quality(&work, format, quality)?;
            continue;
        }
        let (w, h) = work.dimensions();
        if w.max(h) < 96 {
            break;
        }
        let ratio = (target as f32 / best.len() as f32).sqrt().clamp(0.45, 0.88);
        let nw = ((w as f32) * ratio).round().max(64.0) as u32;
        let nh = ((h as f32) * ratio).round().max(64.0) as u32;
        if nw >= w && nh >= h {
            break;
        }
        work = work.resize(nw, nh, FilterType::Triangle);
        if format.lossy() {
            quality = 72;
        }
        best = encode_with_quality(&work, format, quality)?;
    }
    let mut extra = 0;
    while best.len() > target && extra < 10 {
        if cpu_deadline_hit() {
            return done(&work, best, format, input.len(), ow, oh, true);
        }
        extra += 1;
        let (w, h) = work.dimensions();
        if w.max(h) < 48 {
            break;
        }
        let scale = (target as f32 / best.len().max(1) as f32)
            .sqrt()
            .clamp(0.2, 0.7);
        let nw = ((w as f32) * scale).round().max(32.0) as u32;
        let nh = ((h as f32) * scale).round().max(32.0) as u32;
        if nw >= w && nh >= h {
            break;
        }
        work = work.resize(nw, nh, FilterType::Triangle);
        quality = if format.lossy() { 42 } else { quality };
        best = encode_with_quality(&work, format, quality)?;
    }
    let over_budget = best.len() > target;
    done(
        &work,
        best,
        format,
        input.len(),
        ow,
        oh,
        over_budget,
    )
}

fn done(
    work: &DynamicImage,
    bytes: Vec<u8>,
    format: OutFormat,
    original_bytes: usize,
    original_width: u32,
    original_height: u32,
    over_budget: bool,
) -> Result<CompressResult, String> {
    let (w, h) = work.dimensions();
    Ok(CompressResult {
        bytes,
        width: w,
        height: h,
        format,
        original_bytes,
        original_width,
        original_height,
        over_budget,
    })
}

fn decode_err() -> String {
    "Could not read that image. Use JPG, PNG, WebP, or GIF.".into()
}

pub(crate) fn reject_unsupported(input: &[u8]) -> Result<(), String> {
    if looks_like_svg(input) {
        return Err("SVG is not supported. Export a PNG or JPG first.".into());
    }
    if looks_like_heif(input) {
        return Err(
            "HEIC/HEIF is not supported as a raw upload. Convert to JPG in the browser (Safari) or export JPG/PNG."
                .into(),
        );
    }
    if looks_like_avif(input) {
        return Err("AVIF is not supported yet. Use JPG, PNG, WebP, or GIF.".into());
    }
    Ok(())
}

fn looks_like_svg(input: &[u8]) -> bool {
    let lower: Vec<u8> = input
        .iter()
        .take(512)
        .copied()
        .skip_while(|b| b.is_ascii_whitespace())
        .take(256)
        .map(|b| b.to_ascii_lowercase())
        .collect();
    lower.starts_with(b"<svg")
        || (lower.starts_with(b"<?xml") && lower.windows(4).any(|w| w == b"<svg"))
}

fn looks_like_heif(input: &[u8]) -> bool {
    if input.len() < 12 || &input[4..8] != b"ftyp" {
        return false;
    }
    matches!(
        &input[8..12],
        b"heic" | b"heix" | b"heif" | b"mif1" | b"msf1" | b"hevc"
    )
}

fn looks_like_avif(input: &[u8]) -> bool {
    input.len() >= 12
        && &input[4..8] == b"ftyp"
        && matches!(&input[8..12], b"avif" | b"avis")
}

pub(crate) fn decode_capped(input: &[u8]) -> Result<(DynamicImage, u32, u32), String> {
    if input.is_empty() {
        return Err("Empty file.".into());
    }
    if input.len() > MAX_UPLOAD_BYTES {
        return Err("File is over 50 MB. Pick a smaller image.".into());
    }
    reject_unsupported(input)?;
    let img = load_oriented(input)?;
    let (ow, oh) = img.dimensions();
    if ow == 0 || oh == 0 {
        return Err("That image has no pixels.".into());
    }
    check_cpu_deadline()?;
    Ok((cap_dimensions(img), ow, oh))
}

fn load_oriented(input: &[u8]) -> Result<DynamicImage, String> {
    let mut reader = ImageReader::new(Cursor::new(input))
        .with_guessed_format()
        .map_err(|_| decode_err())?;
    if reader.format().is_none() {
        return Err(decode_err());
    }
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_EDGE);
    limits.max_image_height = Some(MAX_EDGE);
    limits.max_alloc = Some(64 * 1024 * 1024);
    reader.limits(limits);
    let mut decoder = reader.into_decoder().map_err(map_decode_err)?;
    let (w, h) = decoder.dimensions();
    if w == 0 || h == 0 {
        return Err("That image has no pixels.".into());
    }
    if (w as u64).saturating_mul(h as u64) > MAX_PIXELS {
        return Err(too_many_pixels());
    }
    let orientation = decoder
        .orientation()
        .unwrap_or(image::metadata::Orientation::NoTransforms);
    let mut img = DynamicImage::from_decoder(decoder).map_err(map_decode_err)?;
    img.apply_orientation(orientation);
    Ok(img)
}

fn too_many_pixels() -> String {
    "That photo has too many pixels. The web page resizes it in the browser first, or export a smaller JPG.".into()
}

fn map_decode_err(e: image::ImageError) -> String {
    let s = e.to_string().to_ascii_lowercase();
    if s.contains("limit") {
        too_many_pixels()
    } else {
        decode_err()
    }
}

fn cap_dimensions(img: DynamicImage) -> DynamicImage {
    let (w, h) = img.dimensions();
    let pixels = w as u64 * h as u64;
    if w <= MAX_EDGE && h <= MAX_EDGE && pixels <= MAX_PIXELS {
        return img;
    }
    let edge_scale = MAX_EDGE as f32 / w.max(h) as f32;
    let px_scale = ((MAX_PIXELS as f32) / (pixels as f32)).sqrt();
    let scale = edge_scale.min(px_scale).min(1.0);
    let nw = ((w as f32) * scale).round().max(1.0) as u32;
    let nh = ((h as f32) * scale).round().max(1.0) as u32;
    img.resize(nw, nh, FilterType::Triangle)
}

pub(crate) fn encode(img: &DynamicImage, format: OutFormat, quality: u8) -> Result<Vec<u8>, String> {
    check_cpu_deadline()?;
    match format {
        OutFormat::Jpeg => encode_jpeg(img, quality),
        OutFormat::Png => encode_png(img),
        OutFormat::Webp => encode_webp_lossless(img),
    }
}

pub(crate) fn encode_with_quality(
    img: &DynamicImage,
    format: OutFormat,
    quality: u8,
) -> Result<Vec<u8>, String> {
    check_cpu_deadline()?;
    match format {
        OutFormat::Jpeg => encode_jpeg(img, quality),
        OutFormat::Png => encode_png(img),
        OutFormat::Webp => encode_webp_quality(img, quality),
    }
}

/// Composite onto white so transparent PNGs do not become a black rectangle.
fn flatten_white(img: &DynamicImage) -> RgbImage {
    let rgba = img.to_rgba8();
    let mut rgb = RgbImage::new(rgba.width(), rgba.height());
    for (src, dst) in rgba.pixels().zip(rgb.pixels_mut()) {
        let a = src.0[3] as u16;
        if a == 255 {
            *dst = Rgb([src.0[0], src.0[1], src.0[2]]);
        } else if a == 0 {
            *dst = Rgb([255, 255, 255]);
        } else {
            let inv = 255 - a;
            dst.0[0] = ((src.0[0] as u16 * a + 255 * inv) / 255) as u8;
            dst.0[1] = ((src.0[1] as u16 * a + 255 * inv) / 255) as u8;
            dst.0[2] = ((src.0[2] as u16 * a + 255 * inv) / 255) as u8;
        }
    }
    rgb
}

fn encode_jpeg(img: &DynamicImage, quality: u8) -> Result<Vec<u8>, String> {
    let rgb = flatten_white(img);
    let mut out = Vec::new();
    let mut enc = JpegEncoder::new_with_quality(&mut out, quality);
    enc.encode(
        rgb.as_raw(),
        rgb.width(),
        rgb.height(),
        ColorType::Rgb8.into(),
    )
    .map_err(|e| e.to_string())?;
    Ok(out)
}

fn encode_png(img: &DynamicImage) -> Result<Vec<u8>, String> {
    let rgba = img.to_rgba8();
    let mut out = Vec::new();
    let enc = PngEncoder::new_with_quality(&mut out, CompressionType::Default, PngFilter::Adaptive);
    enc.write_image(
        rgba.as_raw(),
        rgba.width(),
        rgba.height(),
        ColorType::Rgba8.into(),
    )
    .map_err(|e| e.to_string())?;
    Ok(out)
}

fn encode_webp_quality(img: &DynamicImage, quality: u8) -> Result<Vec<u8>, String> {
    if quality >= 98 {
        return encode_webp_lossless(img);
    }
    encode_webp_lossy(img, quality)
}

fn encode_webp_lossless(img: &DynamicImage) -> Result<Vec<u8>, String> {
    use image::codecs::webp::WebPEncoder;
    let mut out = Vec::new();
    let enc = WebPEncoder::new_lossless(&mut out);
    if img.color().has_alpha() {
        let rgba = img.to_rgba8();
        enc.encode(
            rgba.as_raw(),
            rgba.width(),
            rgba.height(),
            ColorType::Rgba8.into(),
        )
        .map_err(|e| e.to_string())?;
    } else {
        let rgb = img.to_rgb8();
        enc.encode(
            rgb.as_raw(),
            rgb.width(),
            rgb.height(),
            ColorType::Rgb8.into(),
        )
        .map_err(|e| e.to_string())?;
    }
    Ok(out)
}

/// Lossy VP8 via libwebp. Falls back to lossless if the encoder is unavailable.
fn encode_webp_lossy(img: &DynamicImage, quality: u8) -> Result<Vec<u8>, String> {
    let q = quality.clamp(1, 100) as f32;
    let rgba = img.to_rgba8();
    let encoder = webp::Encoder::from_rgba(rgba.as_raw(), rgba.width(), rgba.height());
    let mem = encoder.encode(q);
    let out = Vec::from(&*mem);
    if out.len() < 12 || &out[..4] != b"RIFF" {
        return encode_webp_lossless(img);
    }
    Ok(out)
}

pub fn stem_filename(name: &str) -> String {
    let base = name.rsplit(['/', '\\']).next().unwrap_or(name);
    let stem = base.rsplit_once('.').map(|(s, _)| s).unwrap_or(base);
    let cleaned: String = stem
        .chars()
        .map(|c| {
            if c.is_ascii_alphanumeric() || c == '-' || c == '_' {
                c
            } else {
                '-'
            }
        })
        .take(48)
        .collect();
    if cleaned.is_empty() {
        "image".into()
    } else {
        cleaned
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use image::{Rgba, RgbaImage};

    fn sample_jpeg() -> Vec<u8> {
        let mut img = RgbImage::new(640, 480);
        for (x, y, p) in img.enumerate_pixels_mut() {
            *p = Rgb([
                (x % 255) as u8,
                (y % 255) as u8,
                ((x + y) % 255) as u8,
            ]);
        }
        let dynimg = DynamicImage::ImageRgb8(img);
        encode_jpeg(&dynimg, 92).expect("jpeg")
    }

    #[test]
    fn clamps_target() {
        assert_eq!(clamp_target(1), MIN_TARGET_BYTES);
        assert_eq!(clamp_target(200), 200 * 1024);
    }

    #[test]
    fn jpeg_fits_budget() {
        let src = sample_jpeg();
        let out = compress(&src, 40 * 1024, OutFormat::Jpeg).expect("compress");
        assert!(out.bytes.len() <= 40 * 1024, "got {} bytes", out.bytes.len());
        assert!(out.bytes.len() > 800);
        assert!(!out.over_budget);
    }

    #[test]
    fn already_small_jpeg_is_reencoded() {
        let src = sample_jpeg();
        let tiny = compress(&src, 8 * 1024, OutFormat::Jpeg).unwrap();
        let again = compress(&tiny.bytes, 5 * 1024 * 1024, OutFormat::Jpeg).unwrap();
        assert!(!again.over_budget);
        assert!(!again.bytes.is_empty());
    }

    #[test]
    fn webp_encodes() {
        let src = sample_jpeg();
        let out = compress(&src, 80 * 1024, OutFormat::Webp).expect("webp");
        assert!(!out.bytes.is_empty());
        assert_eq!(&out.bytes[..4], b"RIFF");
        assert!(
            out.bytes.len() <= 80 * 1024,
            "lossy webp should hit the budget, got {}",
            out.bytes.len()
        );
        assert!(!out.over_budget);
    }

    #[test]
    fn noisy_jpeg_hits_50kb() {
        let mut img = RgbImage::new(800, 600);
        for (x, y, p) in img.enumerate_pixels_mut() {
            let n = x
                .wrapping_mul(1_103_515_245)
                .wrapping_add(y.wrapping_mul(12_345))
                % 251;
            *p = Rgb([n as u8, (n as u8).wrapping_add(41), (n as u8).wrapping_add(97)]);
        }
        let src = encode_jpeg(&DynamicImage::ImageRgb8(img), 92).expect("jpeg");
        let out = compress(&src, 50 * 1024, OutFormat::Jpeg).expect("compress");
        assert!(
            out.bytes.len() <= 50 * 1024,
            "got {} bytes",
            out.bytes.len()
        );
        assert!(!out.over_budget);
    }

    #[test]
    fn transparent_png_becomes_white_on_jpeg() {
        let mut img = RgbaImage::new(48, 48);
        for p in img.pixels_mut() {
            *p = Rgba([0, 0, 0, 0]);
        }
        let src = encode_png(&DynamicImage::ImageRgba8(img)).expect("png");
        let out = compress(&src, 50 * 1024, OutFormat::Jpeg).expect("compress");
        let decoded = image::load_from_memory(&out.bytes).unwrap().to_rgb8();
        assert_eq!(decoded.get_pixel(0, 0), &Rgb([255, 255, 255]));
    }

    #[test]
    fn transparent_png_keeps_alpha_on_webp() {
        let mut img = RgbaImage::new(48, 48);
        for p in img.pixels_mut() {
            *p = Rgba([0, 0, 0, 0]);
        }
        let src = encode_png(&DynamicImage::ImageRgba8(img)).expect("png");
        let out = compress(&src, 50 * 1024, OutFormat::Webp).expect("webp");
        let decoded = image::load_from_memory(&out.bytes).unwrap().to_rgba8();
        assert_eq!(decoded.get_pixel(0, 0).0[3], 0);
    }

    #[test]
    fn rejects_heic_magic() {
        let mut b = vec![0u8; 24];
        b[4..8].copy_from_slice(b"ftyp");
        b[8..12].copy_from_slice(b"heic");
        let err = compress(&b, 20_000, OutFormat::Jpeg).unwrap_err();
        assert!(err.contains("HEIC"), "{err}");
    }

    #[test]
    fn rejects_svg() {
        let err = compress(b"<svg xmlns='http://www.w3.org/2000/svg'></svg>", 20_000, OutFormat::Jpeg)
            .unwrap_err();
        assert!(err.contains("SVG"), "{err}");
    }

    #[test]
    fn rejects_garbage() {
        assert!(compress(b"not-an-image", 20_000, OutFormat::Jpeg).is_err());
    }

    #[test]
    fn filename_stem_safe() {
        assert_eq!(stem_filename("../../weird photo!.PNG"), "weird-photo-");
    }

    #[test]
    fn expired_deadline_fails_encode() {
        let img = DynamicImage::ImageRgb8(RgbImage::new(8, 8));
        let err = with_cpu_deadline(Instant::now(), || encode(&img, OutFormat::Jpeg, 80))
            .unwrap_err();
        assert_eq!(err, CPU_TIMEOUT_MSG);
    }
}
