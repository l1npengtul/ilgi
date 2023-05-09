use std::collections::HashSet;
use confique::Config;
use std::default::Default;

#[derive(Clone, Debug, PartialEq, Config)]
pub struct IlgiConfig {
    #[config(default = "en")]
    pub default_language: String,

    #[config(nested)]
    pub build: Build,
}

pub struct Serve {
    #[config(default = true)]
    pub rss_feed: bool,
    #[config(default = true)]
    pub atom_feed: bool,
}

#[derive(Clone, Debug, PartialEq, Config)]
pub struct Build {
    #[config(nested)]
    pub git: Git,
    #[config(nested)]
    pub statics: Static,
    #[config(nested)]
    pub html: Html,
    #[config(nested)]
    pub javascript: Js,
    #[config(nested)]
    pub css: Css,
    pub theme: Option<String>,
}

#[derive(Copy, Clone, Debug, PartialEq, Config)]
pub struct Static {
    #[config(default = true)]
    pub minify_png: bool,
    #[config(default = 3)]
    pub minify_png_preset: u8,
    #[config(default = true)]
    pub minify_svg: bool,
    #[config(default = true)]
    pub minify_jpeg: bool,
    #[config(default = 0.75)]
    pub minify_jpeg_quality: f32,
    #[config(default = true)]
    pub minify_webp: bool,
    #[config(default = 0.75)]
    pub minify_webp_quality: f32,
}

#[derive(Copy, Clone, Debug, PartialEq, Config)]
pub struct Html {
    #[config(default = true)]
    pub minify: bool,
}

#[derive(Clone, Debug, PartialEq, Config)]
pub struct Js {
    #[config(default = true)]
    pub minify: bool,
}

#[derive(Clone, Debug, PartialEq, Config)]
pub struct Css {
    #[config(nested, default = CssStyle::Compressed)]
    pub style: CssStyle,
    #[config(default = 2)]
    pub precision: u32,
    #[config(default = true)]
    pub minify: bool,
    #[config(default = Default::default())]
    pub unknown_symbols: HashSet<String>,
    #[config(nested, default = Default::default())]
    pub targets: Vec<String>
}

#[derive(Copy, Clone, Debug, PartialEq, Config)]
pub enum CssStyle {
    Expanded,
    Compressed,
    Introspection
}

#[derive(Clone, Debug, PartialEq, Config)]
pub struct Git {
    pub git_repo: String,
    #[config(default = "main")]
    pub git_branch: String,
    #[config(nested, default = GitAuth::None)]
    pub auth: GitAuth,
    #[config(nested, default = GitUpdate::Polling)]
    pub update: GitUpdate,
    #[config(default = true)]
    pub recursive_clone: bool,
    #[config(default = false)]
    pub lfs_clone: bool,
}

#[derive(Clone, Debug, PartialEq, Config)]
pub enum GitAuth {
    None,
    Ssh {
        public_key: String,
        private_key: String,
    },
    UsernamePassword {
        username: String,
        password: String,
    }
}

#[derive(Clone, Debug, PartialEq, Config)]
pub enum GitUpdate {
    Polling,
    Webhook {
        secret: Option<String>,
    },
}
