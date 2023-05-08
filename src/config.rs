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
}

#[derive(Copy, Clone, Debug, PartialEq, Config)]
pub struct Static {
    #[config(default = true)]
    pub minify_png: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Config)]
pub struct Html {
    #[config(default = true)]
    pub minify: bool,
    #[config(default = true)]
    pub do_not_minify_doctype: bool,
    #[config(default = true)]
    pub ensure_spec_compliant_unquoted_attribute_values: bool,
    #[config(default = false)]
    pub keep_closing_tags: bool,
    #[config(default = false)]
    pub keep_html_and_head_opening_tags: bool,
    #[config(default = true)]
    pub keep_spaces_between_attributes: bool,
    #[config(default = false)]
    pub keep_comments: bool,
    #[config(nested, default = HtmlMinifyCssLevel::Lowest)]
    pub minify_css_level: HtmlMinifyCssLevel,
    #[config(default = true)]
    pub minify_js: bool,
    #[config(default = false)]
    pub remove_bangs: bool,
    #[config(default = false)]
    pub remove_processing_instructions: bool,
}

#[derive(Copy, Clone, Debug, PartialEq, Config)]
pub enum HtmlMinifyCssLevel {
    Off,
    Lowest,
    Middle,
    Highest,
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
    pub git_repo: Option<String>,
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
    Webhook,
}
