use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::str::{from_utf8, from_utf8_unchecked};
use std::sync::{Arc};
use dashmap::DashMap;
use fallible_iterator::FallibleIterator;
use ignore::{WalkBuilder};
use memmap2::Mmap;
use relative_path::RelativePath;
use rhai::{AST, Engine, Scope};
use tokio::fs::File;
use tokio_stream::StreamExt;
use tracing::{instrument};
use ilgi_core::error::IResult;
use itertools::{process_results};
use lightningcss::stylesheet::{MinifyOptions, ParserOptions, PrinterOptions, StyleSheet};
use lightningcss::targets::Browsers;
use miette::Report;
use rsass::output::{Format, Style};
use tera::Tera;
use ilgi_core::theme::ThemeDefinition;
use upon::{Engine as UponEngine, Value};
use crate::config::{CssStyle, IlgiConfig};
use crate::file_ops::{add_hash_filename, optimize_static_file};

#[derive(Clone, Debug, Default, PartialOrd, PartialEq)]
pub struct DiskTheme {
    pub definition: ThemeDefinition,
    pub statics: DashMap<String, Mmap>,
    pub templates: DashMap<String, Mmap>,
    pub runtime_templates: DashMap<String, Mmap>,
    pub runtime_filters_slice: DashMap<String, Mmap>,
    pub runtime_filters_str: DashMap<String, Mmap>,
    pub runtime_filters_map: DashMap<String, Mmap>,
    pub shortcodes: DashMap<String, Mmap>,
    pub rhai_functions: DashMap<String, Mmap>,
    pub sass: DashMap<String, Mmap>,
}

#[derive(Clone, Debug, Default, PartialOrd, PartialEq)]
pub struct Theme {
    pub definition: ThemeDefinition,
    pub statics: Arc<DashMap<String, Box<dyn AsRef<[u8]>>>>,
    pub tera: Tera,
    pub upon: UponEngine<'static>,
    pub rhai_engine: Engine,
    pub rhai_functions: Arc<DashMap<String, AST>>,
    pub sass: Arc<DashMap<String, String>>,
}

impl DiskTheme {
    #[instrument]
    pub async fn load(self, config: &IlgiConfig) -> IResult<Theme> {
        //

        let mut tera = Tera::default();
        tera.add_raw_templates(
            process_results(
                self.templates.into_iter()
                    .map(|(n, m)| from_utf8(m.as_ref()).map(|s| (n, s))),
                |x| {
                    x
                }
            )?
        )?;

        let minify_options_css = Browsers::from_browserslist(config.build.css.targets.as_ref());
        if config.build.css.minify {
            if let Err(why) = &minify_options_css {
                return Err(Report::from(why))
            }
        }

        let mut sass = Arc::new(
            process_results(
            self.sass.into_iter()
                .map(|(n, m)| {
                    rsass::compile_scss(&m, Format { style: {
                        match config.build.css.style {
                            CssStyle::Expanded => Style::Expanded,
                            CssStyle::Compressed => Style::Compressed,
                            CssStyle::Introspection => Style::Introspection,
                        }
                    }, precision: config.build.css.precision as usize })
                        .map(|x| (n, x))
                }),
            |x| {
                if config.build.css.minify {
                    x.map(|(n, x)| {
                        let mut sheet = StyleSheet::parse(unsafe { from_utf8_unchecked(&x) }, ParserOptions::default()).unwrap();
                        sheet.minify(MinifyOptions { targets: minify_options_css.unwrap(), unused_symbols: config.build.css.unknown_symbols.clone() })
                            .map_err(Report::from)
                            .map(|_| sheet.to_css(PrinterOptions {
                                minify: config.build.css.minify,
                                source_map: None,
                                project_root: None,
                                targets: minify_options_css.unwrap(),
                                analyze_dependencies: None,
                                pseudo_classes: None,
                            }).map_err(Report::from))
                            .flatten()
                            .map(|x| (n, x.code))
                            .map(|(n, x)| {
                                let new_name = add_hash_filename(&n, &x);
                                (new_name, x)
                            })
                    })
                } else {
                    x.map(|(n, x)| {
                        (n, String::from_utf8(x).unwrap())
                    })
                        .map(|(n, x)| {
                            let new_name = add_hash_filename(&n, &x);
                            (new_name, x)
                        })
                }
            }
        )?.collect::<Result<DashMap<String, String>, Report>>()?);

        let mut engine = Engine::new();
        let mut rhai_functions = Arc::new(
            process_results(
                self.rhai_functions.into_iter().map(|(n, a)| {
                    from_utf8(a.as_ref()).map(|x| (n, x))
                }),
                |x| {
                    x
                }
            )?.map(|(n, s)| {
                engine.compile(s).map(|x| (n, x))
            }).collect::<Result<DashMap<String, AST>, _>>()?
        );

        let mut upon = UponEngine::new();
        let _ = process_results(
            self.runtime_templates.into_iter().map(|(s, n)| {
                from_utf8(n.as_ref()).map(|x| (s, x))
            }),
            |x| {
                x.map(|(n, s)| {
                    upon.add_template(n, s)
                })
            }
        )?;

        let _ = process_results(process_results(
            self.runtime_filters_str.into_iter().map(|(n, a)| {
                from_utf8(a.as_ref()).map(|x| (n, x))
            }),
            |x| {
                x
            }
        )?.map(|(n, s)| {
            engine.compile(s).map(|x| (n, x))
        }).map(|(n, ast)| {
            upon.add_template(n, move |s: &str| {
                let engine = Engine::new();
                let ast = ast;
                match engine.call_fn::<String>(&mut Scope::new(), &ast, "filter", &[s]) {
                    Ok(s) => s,
                    Err(why) => why.to_string(),
                }
            })
        }), |x| x)?;

        let _ = process_results(process_results(
            self.runtime_filters_slice.into_iter().map(|(n, a)| {
                from_utf8(a.as_ref()).map(|x| (n, x))
            }),
            |x| {
                x
            }
        )?.map(|(n, s)| {
            engine.compile(s).map(|x| (n, x))
        }).map(|(n, ast)| {
            upon.add_template(n, move |s: &[Value]| {
                let engine = Engine::new();
                let ast = ast;
                match engine.call_fn::<String>(&mut Scope::new(), &ast, "filter", &[s]) {
                    Ok(s) => s,
                    Err(why) => why.to_string(),
                }
            })
        }), |x| x)?;

        let _ = process_results(process_results(
            self.runtime_filters_map.into_iter().map(|(n, a)| {
                from_utf8(a.as_ref()).map(|x| (n, x))
            }),
            |x| {
                x
            }
        )?.map(|(n, s)| {
            engine.compile(s).map(|x| (n, x))
        }).map(|(n, ast)| {
            upon.add_template(n, move |s: &BTreeMap<String, Value>| {
                let engine = Engine::new();
                let ast = ast;
                match engine.call_fn::<String>(&mut Scope::new(), &ast, "filter", &[s]) {
                    Ok(s) => s,
                    Err(why) => why.to_string(),
                }
            })
        }), |x| x)?;

        let statics = Arc::new(self.statics.into_iter().map(|(name, data)| {
            // get file extension
            let extension = match &name.rsplit_once(".") {
                Some(s) => s.1,
                None => ""
            };
            (name, extension, data)
        })
            .map(|(n, e, data)| {
                optimize_static_file(config, e, data).map(|x| {
                    let new_name = add_hash_filename(&n, &x);
                    (new_name, x)
                })
            })
            .collect::<IResult<DashMap<String, Box<dyn AsRef<[u8]>>>>>()?);


        Ok(
            Theme {
                definition: self.definition,
                statics,
                tera,
                upon,
                rhai_engine: engine,
                rhai_functions,
                sass,
            }
        )
    }
}

#[instrument]
pub async fn parse_theme(directory: impl AsRef<Path>, config: &IlgiConfig) -> IResult<DiskTheme> {
    let mut path = Rc::new(PathBuf::from(directory.as_ref()));
    let theme_def = {
        let theme_toml = unsafe { Mmap::map(File::open(path.clone() + "theme.toml").await?)? };
        toml::from_str::<ThemeDefinition>(from_utf8(theme_toml.as_ref())?)?
    };

    let static_files = map_dir_to_named_mem(path.clone() + "static")?.collect::<DashMap<_, _>>();

    let sass = map_dir_to_named_mem(path.clone() + "sass")?.collect::<DashMap<_, _>>();

    let templates = map_dir_to_named_mem(path.clone() + "templates")?.collect::<DashMap<_, _>>();
    let shortcodes = map_dir_to_named_mem(path.clone() + "shortcodes")?.collect::<DashMap<_, _>>();

    let runtime_templates = map_dir_to_named_mem(path.clone() + "runtime/templates")?.collect::<DashMap<_, _>>();
    let runtime_filters_slice = map_dir_to_named_mem(path.clone() + "runtime/filters/slice")?.collect::<DashMap<_, _>>();
    let runtime_filters_str = map_dir_to_named_mem(path.clone() + "runtime/filters/str")?.collect::<DashMap<_, _>>();
    let runtime_filters_map = map_dir_to_named_mem(path.clone() + "runtime/filters/map")?.collect::<DashMap<_, _>>();

    let rhai_functions = map_dir_to_named_mem(path.clone() + "rhai")?.collect::<DashMap<_, _>>();


    Ok(
        DiskTheme {
            definition: theme_def,
            statics: static_files,
            templates,
            runtime_templates,
            runtime_filters_slice,
            runtime_filters_str,
            runtime_filters_map,
            shortcodes,
            rhai_functions,
            sass,
        }
    )
}

fn load_dir(path: impl AsRef<Path>) -> IResult<impl Iterator<Item = PathBuf>> {
    Ok(
        process_results(
            WalkBuilder::new(path.as_ref()).add_custom_ignore_filename(".ilgi_ignore")
                .build()
                .into_iter()
            ,
            |file| {
                file.map(|x| {
                    x.into_path()
                })
            }
        )?
    )
}

fn map_dir_to_named_mem(path: impl AsRef<Path>) -> IResult<impl Iterator<Item = (String, Mmap)>> {
    Ok(process_results(
        process_results(process_results(
            load_dir(path.as_ref())?
                .map(|fp| {
                    RelativePath::new(&fp)
                        .strip_prefix(path.as_ref())
                        .map(|rp| (rp.to_string(), fp))
                }),
            |x| x
                .map(|(name, path)| {
                    (name, std::fs::File::open(path))
                })
        )?.map(|(n, x)| {
            x.map(|x| (n, x))
        }), |x| x
            .map(|(n, f)| {
                unsafe {
                    Mmap::map(f)
                }.map(|m| (n, m))
            }))?,
        |x| x
    )?)
}


