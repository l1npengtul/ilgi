use std::path::Path;
use gix::remote::fetch::Shallow;
use tempfile::tempdir_in;
use ilgi_core::error::IResult;
use crate::config::IlgiConfig;

pub async fn clone_git(config: &IlgiConfig, dir: impl AsRef<Path>) -> IResult<()> {
    let tempdir = tempdir_in(dir.as_ref())?;

    let mut fetch = gix::prepare_clone(
        &config.build.git.git_repo,
        tempdir.path()
    )?;

    fetch.with_shallow(Shallow::DepthAtRemote(2.into()))
        .with_fetch_options(Options::default())
        .configure_connection(|connection| {
            connection.set_credentials(|cred| {
                cred.
            })
        })

}