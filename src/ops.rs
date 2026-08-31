//! Convert, resize, remove a flat background, extract a palette.

use std::collections::VecDeque;

use image::imageops::FilterType;
use image::{DynamicImage, GenericImageView, Rgba, RgbaImage};

use crate::compress::{decode_capped, encode_with_quality, OutFormat};

#[derive(Debug, Clone)]
pub struct ImageOut {
    pub bytes: Vec<u8>,
    pub width: u32,
    pub height: u32,
    pub format: OutFormat,
    pub original_bytes: usize,
    pub original_width: u32,
    pub original_height: u32,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FitMode {
    Fit,
    Fill,
    Stretch,
}

impl FitMode {
    pub fn parse(raw: &str) -> Self {
        match raw.trim().to_ascii_lowercase().as_str() {
            "fill" | "cover" | "crop" => Self::Fill,
            "stretch" | "exact" => Self::Stretch,
            _ => Self::Fit,
        }
    }
}

pub fn clamp_quality(q: u32) -> u8 {
    q.clamp(20, 100) as u8
}

pub fn convert(input: &[u8], format: OutFormat, quality: u8) -> Result<ImageOut, String> {
    let (img, ow, oh) = decode_capped(input)?;
    finish(img, format, quality, input.len(), ow, oh)
}

pub fn resize(
    input: &[u8],
    width: Option<u32>,
    height: Option<u32>,
    mode: FitMode,
    format: OutFormat,
    quality: u8,
) -> Result<ImageOut, String> {
    let (img, ow, oh) = decode_capped(input)?;
    let (cw, ch) = img.dimensions();
    let tw = width.filter(|w| *w > 0).unwrap_or(0).min(4096);
    let th = height.filter(|h| *h > 0).unwrap_or(0).min(4096);
    let resized = if tw == 0 && th == 0 {
        img
    } else if tw == 0 {
        let nh = th.max(1);
        let nw = ((cw as u64 * nh as u64) / ch.max(1) as u64).max(1) as u32;
        img.resize(nw.min(4096), nh, FilterType::Triangle)
    } else if th == 0 {
        let nw = tw.max(1);
        let nh = ((ch as u64 * nw as u64) / cw.max(1) as u64).max(1) as u32;
        img.resize(nw, nh.min(4096), FilterType::Triangle)
    } else {
        match mode {
            FitMode::Fit => img.resize(tw, th, FilterType::Triangle),
            FitMode::Fill => img.resize_to_fill(tw, th, FilterType::Triangle),
            FitMode::Stretch => DynamicImage::ImageRgba8(image::imageops::resize(
                &img.to_rgba8(),
                tw,
                th,
                FilterType::Triangle,
            )),
        }
    };
    finish(resized, format, quality, input.len(), ow, oh)
}

pub fn remove_background(
    input: &[u8],
    tolerance: u8,
    format: OutFormat,
) -> Result<ImageOut, String> {
    let (img, ow, oh) = decode_capped(input)?;
    let cut = punch_flat_background(&img.to_rgba8(), tolerance);
    let out = DynamicImage::ImageRgba8(cut);
    let fmt = match format {
        OutFormat::Jpeg => OutFormat::Png,
        other => other,
    };
    finish(out, fmt, 100, input.len(), ow, oh)
}

#[derive(Debug, Clone, serde::Serialize)]
pub struct Swatch {
    pub hex: String,
    pub pct: f32,
    pub r: u8,
    pub g: u8,
    pub b: u8,
}

pub fn extract_colors(input: &[u8], count: usize) -> Result<(Vec<Swatch>, u32, u32), String> {
    let (img, ow, oh) = decode_capped(input)?;
    let n = count.clamp(3, 12);
    Ok((palette(&img, n), ow, oh))
}

fn finish(
    img: DynamicImage,
    format: OutFormat,
    quality: u8,
    original_bytes: usize,
    original_width: u32,
    original_height: u32,
) -> Result<ImageOut, String> {
    let (w, h) = img.dimensions();
    let bytes = encode_with_quality(&img, format, quality)?;
    Ok(ImageOut {
        bytes,
        width: w,
        height: h,
        format,
        original_bytes,
        original_width,
        original_height,
    })
}

fn dist2(a: [u8; 3], b: [u8; 3]) -> u32 {
    let dr = a[0] as i32 - b[0] as i32;
    let dg = a[1] as i32 - b[1] as i32;
    let db = a[2] as i32 - b[2] as i32;
    (dr * dr + dg * dg + db * db) as u32
}

fn rgb_of(p: Rgba<u8>) -> [u8; 3] {
    [p.0[0], p.0[1], p.0[2]]
}

/// Flood-fill from the border: pixels similar to the edge color become transparent.
/// Works on product shots and screenshots with a flat backdrop; not a portrait matte.
fn punch_flat_background(src: &RgbaImage, tolerance: u8) -> RgbaImage {
    let w = src.width() as usize;
    let h = src.height() as usize;
    if w == 0 || h == 0 {
        return src.clone();
    }
    let tol = tolerance.clamp(8, 90) as u32;
    let thresh = tol.saturating_mul(tol).saturating_mul(3);
    let mut bg = [255u8, 255, 255];
    let mut samples: Vec<[u8; 3]> = Vec::new();
    for x in 0..w {
        samples.push(rgb_of(*src.get_pixel(x as u32, 0)));
        samples.push(rgb_of(*src.get_pixel(x as u32, h as u32 - 1)));
    }
    for y in 0..h {
        samples.push(rgb_of(*src.get_pixel(0, y as u32)));
        samples.push(rgb_of(*src.get_pixel(w as u32 - 1, y as u32)));
    }
    if !samples.is_empty() {
        let mut rs = samples.iter().map(|c| c[0] as u32).collect::<Vec<_>>();
        let mut gs = samples.iter().map(|c| c[1] as u32).collect::<Vec<_>>();
        let mut bs = samples.iter().map(|c| c[2] as u32).collect::<Vec<_>>();
        rs.sort_unstable();
        gs.sort_unstable();
        bs.sort_unstable();
        let mid = samples.len() / 2;
        bg = [rs[mid] as u8, gs[mid] as u8, bs[mid] as u8];
    }

    let mut seen = vec![false; w * h];
    let mut q = VecDeque::new();
    let push = |x: usize, y: usize, seen: &mut [bool], q: &mut VecDeque<(usize, usize)>| {
        let i = y * w + x;
        if seen[i] {
            return;
        }
        let p = rgb_of(*src.get_pixel(x as u32, y as u32));
        if dist2(p, bg) > thresh {
            return;
        }
        seen[i] = true;
        q.push_back((x, y));
    };
    for x in 0..w {
        push(x, 0, &mut seen, &mut q);
        push(x, h - 1, &mut seen, &mut q);
    }
    for y in 0..h {
        push(0, y, &mut seen, &mut q);
        push(w - 1, y, &mut seen, &mut q);
    }
    while let Some((x, y)) = q.pop_front() {
        let neigh = [
            (x.wrapping_sub(1), y),
            (x + 1, y),
            (x, y.wrapping_sub(1)),
            (x, y + 1),
        ];
        for (nx, ny) in neigh {
            if nx < w && ny < h {
                push(nx, ny, &mut seen, &mut q);
            }
        }
    }

    let mut out = src.clone();
    let hard = thresh / 2;
    for y in 0..h {
        for x in 0..w {
            if !seen[y * w + x] {
                continue;
            }
            let px = out.get_pixel_mut(x as u32, y as u32);
            let d = dist2(rgb_of(*px), bg);
            let a = if d <= hard {
                0u8
            } else if d >= thresh {
                px.0[3]
            } else {
                let span = (thresh - hard).max(1);
                ((d - hard) * 255 / span) as u8
            };
            px.0[3] = a.min(px.0[3]);
        }
    }
    out
}

fn palette(img: &DynamicImage, count: usize) -> Vec<Swatch> {
    let rgba = img.to_rgba8();
    let total = (rgba.width() as u64).saturating_mul(rgba.height() as u64).max(1);
    let step = ((total / 80_000).max(1)) as u32;
    let mut buckets = [0u32; 512];
    let mut i = 0u32;
    for p in rgba.pixels() {
        i = i.wrapping_add(1);
        if i % step != 0 {
            continue;
        }
        if p.0[3] < 40 {
            continue;
        }
        let idx = ((p.0[0] as usize >> 5) << 6) | ((p.0[1] as usize >> 5) << 3) | (p.0[2] as usize >> 5);
        buckets[idx] += 1;
    }
    let sampled: u32 = buckets.iter().sum::<u32>().max(1);
    let mut ranked: Vec<(usize, u32)> = buckets
        .iter()
        .enumerate()
        .filter(|(_, n)| **n > 0)
        .map(|(i, n)| (i, *n))
        .collect();
    ranked.sort_by(|a, b| b.1.cmp(&a.1));
    let mut picked: Vec<Swatch> = Vec::new();
    for (idx, n) in ranked {
        let r = ((((idx >> 6) & 7) << 5) + 16) as u8;
        let g = ((((idx >> 3) & 7) << 5) + 16) as u8;
        let b = (((idx & 7) << 5) + 16) as u8;
        if picked.iter().any(|s| dist2([s.r, s.g, s.b], [r, g, b]) < 1800) {
            continue;
        }
        picked.push(Swatch {
            hex: format!("#{r:02X}{g:02X}{b:02X}"),
            pct: (n as f32 * 100.0 / sampled as f32 * 10.0).round() / 10.0,
            r,
            g,
            b,
        });
        if picked.len() >= count {
            break;
        }
    }
    picked
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::compress::{encode, OutFormat};
    use image::{Rgb, RgbImage, RgbaImage};

    fn jpeg_block() -> Vec<u8> {
        let mut img = RgbImage::new(120, 80);
        for (x, y, p) in img.enumerate_pixels_mut() {
            *p = Rgb([(x % 200) as u8, 40, (y % 200) as u8]);
        }
        encode(&DynamicImage::ImageRgb8(img), OutFormat::Jpeg, 90).unwrap()
    }

    #[test]
    fn convert_to_webp() {
        let out = convert(&jpeg_block(), OutFormat::Webp, 80).unwrap();
        assert_eq!(&out.bytes[..4], b"RIFF");
        assert_eq!(out.format, OutFormat::Webp);
    }

    #[test]
    fn resize_width_keeps_aspect() {
        let out = resize(
            &jpeg_block(),
            Some(60),
            None,
            FitMode::Fit,
            OutFormat::Jpeg,
            80,
        )
        .unwrap();
        assert_eq!(out.width, 60);
        assert_eq!(out.height, 40);
    }

    #[test]
    fn remove_white_border() {
        let mut img = RgbaImage::new(40, 40);
        for p in img.pixels_mut() {
            *p = image::Rgba([255, 255, 255, 255]);
        }
        for y in 10..30 {
            for x in 10..30 {
                img.put_pixel(x, y, image::Rgba([20, 80, 200, 255]));
            }
        }
        let src = encode(&DynamicImage::ImageRgba8(img), OutFormat::Png, 90).unwrap();
        let out = remove_background(&src, 24, OutFormat::Png).unwrap();
        let decoded = image::load_from_memory(&out.bytes).unwrap().to_rgba8();
        assert!(decoded.get_pixel(0, 0).0[3] < 40);
        assert!(decoded.get_pixel(20, 20).0[3] > 200);
    }

    #[test]
    fn remove_bg_webp_keeps_alpha() {
        let mut img = RgbaImage::new(40, 40);
        for p in img.pixels_mut() {
            *p = image::Rgba([255, 255, 255, 255]);
        }
        for y in 10..30 {
            for x in 10..30 {
                img.put_pixel(x, y, image::Rgba([20, 80, 200, 255]));
            }
        }
        let src = encode(&DynamicImage::ImageRgba8(img), OutFormat::Png, 90).unwrap();
        let out = remove_background(&src, 24, OutFormat::Webp).unwrap();
        assert_eq!(&out.bytes[..4], b"RIFF");
        let decoded = image::load_from_memory(&out.bytes).unwrap().to_rgba8();
        assert!(decoded.get_pixel(0, 0).0[3] < 40);
        assert!(decoded.get_pixel(20, 20).0[3] > 200);
    }

    #[test]
    fn convert_png_alpha_to_lossy_webp() {
        let mut img = RgbaImage::new(24, 24);
        for p in img.pixels_mut() {
            *p = image::Rgba([0, 0, 0, 0]);
        }
        let src = encode(&DynamicImage::ImageRgba8(img), OutFormat::Png, 90).unwrap();
        let out = convert(&src, OutFormat::Webp, 80).unwrap();
        let decoded = image::load_from_memory(&out.bytes).unwrap().to_rgba8();
        assert!(decoded.get_pixel(0, 0).0[3] < 40);
    }

    #[test]
    fn palette_has_colors() {
        let (colors, w, h) = extract_colors(&jpeg_block(), 5).unwrap();
        assert_eq!((w, h), (120, 80));
        assert!(!colors.is_empty());
        assert!(colors[0].hex.starts_with('#'));
    }
}
