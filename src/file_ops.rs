use std::str::from_utf8;
use lightningcss::printer::PrinterOptions;
use lightningcss::stylesheet::{MinifyOptions, ParserOptions, StyleSheet};
use lightningcss::targets::Browsers;
use memmap2::Mmap;
use minify_js::{Session, TopLevelMode};
use oxipng::Options;
use rimage::{ImageData, OutputFormat};
use ilgi_core::error::IResult;
use crate::config::IlgiConfig;

pub fn optimize_static_file(config: &IlgiConfig, ext: &str, file: Mmap) -> IResult<Box<dyn AsRef<[u8]>>> {
    match ext {
        "png" => {
            if config.build.statics.minify_png {
                Ok(oxipng::optimize_from_memory(&file, &Options::from_preset(config.build.statics.minify_png_preset))?.into())
            }
        }
        "svg" => {
            if config.build.statics.minify_svg {
                let mut document = svgcleaner::cleaner::parse_data(
                    from_utf8(&file)?)?;
                svgcleaner::cleaner::clean_doc(
                    &mut document,
                    &svgcleaner::CleaningOptions::default(),
                    &svgcleaner::WriteOptions::default(),
                )?;
                Ok(
                    document.to_string().into_boxed_bytes()
                )
            }
        }
        "webp" => {
            if config.build.statics.minify_webp {
                let cfg = rimage::Config::build(config.build.statics.minify_webp_quality,
                OutputFormat::WebP, None, None, None)?;
                let img = imagesize::blob_size(&file)?;
                let optimized = rimage::Encoder::new(&cfg, ImageData::new(
                    img.width, img.height, Vec::from_slice(&file)
                )).encode()?;

                Ok(optimized.into())
            }
        }
        "jpg" | "jpeg" => {
            if config.build.statics.minify_jpeg {
                let cfg = rimage::Config::build(config.build.statics.minify_jpeg_quality,
                                                OutputFormat::MozJpeg, None, None, None)?;
                let img = imagesize::blob_size(&file)?;
                let optimized = rimage::Encoder::new(&cfg, ImageData::new(
                    img.width, img.height, Vec::from_slice(&file)
                )).encode()?;

                Ok(optimized.into())
            }
        }
        "html" => {
            if config.build.html.minify {
                let minified = minify_html::minify(&file, &minify_html::Cfg::default());
                Ok(minified.into())
            }
        }
        "js" => {
            if config.build.javascript.minify {
                let mut output = Vec::with_capacity(file.len());
                minify_js::minify(&Session::new(), TopLevelMode::Global, &file, &mut output)?;
                output.shrink_to_fit();
                Ok(output.into())
            }
        }
        "css" => {
            if config.build.css.minify {
                let minify_options_css = Browsers::from_browserslist(config.build.css.targets.as_ref())?;
                let mut sheet = StyleSheet::parse(from_utf8(&file)?, ParserOptions::default()).unwrap();
                sheet.minify(MinifyOptions { targets: minify_options_css, unused_symbols: config.build.css.unknown_symbols.clone() })?;
                let out = sheet.to_css(PrinterOptions {
                    minify: config.build.css.minify,
                    source_map: None,
                    project_root: None,
                    targets: minify_options_css,
                    analyze_dependencies: None,
                    pseudo_classes: None,
                })?.code;

                Ok(
                    out.to_string().into_boxed_bytes()
                )
            }
        }
        _ => {}
    }
    Ok(Box::new(file.into()))
}

pub fn add_hash_filename(filename: impl AsRef<str>, data: impl AsRef<[u8]>) -> String {
    match filename.as_ref().rsplit_once(".") {
        Some((b, e)) => {
            format!("{b}-{}.{e}", seahash::hash(data.as_ref()))
        }
        None => {
            format!("{}-{}", filename.as_ref(), seahash::hash(data.as_ref()))
        }
    }
}

pub fn decode_hash_filename(filename: impl AsRef<str>) -> Option<(String, u64)> {
    match filename.as_ref().rsplit_once("-") {
        Some((b, e)) => {
            match e.rsplit_once(".") {
                Some((hash, ext)) => {
                    hash.parse::<u64>().ok().map(|x| {
                        (format!("{b}.{ext}"), x)
                    })
                }
                None => None,
            }
        }
        None => None,
    }
}
