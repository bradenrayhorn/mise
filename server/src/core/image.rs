use anyhow::{anyhow, Context};

use base64::Engine;
use ring::rand::SecureRandom;
use semver::VersionReq;

use crate::{
    core::Error,
    datastore::Pool,
    domain,
    imagestore::{self, ImageStore},
};

pub async fn upload(
    datastore: &Pool,
    image_store: &ImageStore,
    file: Vec<u8>,
) -> Result<domain::image::Id, Error> {
    let id = domain::image::Id::new();

    image_store
        .upload(
            &original_path(String::from(&id).as_ref()),
            prepare_image(file).await?,
        )
        .await
        .context("Could not upload image.")?;

    datastore
        .create_image(&id)
        .await
        .context("Could not persist image.")?;

    Ok(id)
}

pub async fn get(image_store: &ImageStore, image_id: &str) -> Result<Vec<u8>, Error> {
    image_store
        .get(&original_path(image_id))
        .await
        .map_err(|err| match err {
            imagestore::Error::NotFound(_) => Error::NotFound("Image not found.".into()),
            _ => Error::Other(anyhow!(err).context("Could not get image.")),
        })
}

async fn prepare_image(image_bytes: Vec<u8>) -> Result<Vec<u8>, Error> {
    let working_dir = "/tmp/mise-images";
    if !tokio::fs::try_exists(working_dir)
        .await
        .context("Checking dir exist")?
    {
        tokio::fs::create_dir(working_dir)
            .await
            .context("create dir")?;
    }

    let mut bytes: [u8; 32] = [0; 32];
    ring::rand::SystemRandom::new()
        .fill(&mut bytes)
        .map_err(|_| anyhow!("Could not generate random path."))?;
    let random_file = base64::engine::general_purpose::URL_SAFE
        .encode(bytes)
        .to_string();

    let old = format!("{working_dir}/{random_file}-old");
    let new = format!("{working_dir}/{random_file}-new");

    tokio::fs::write(&old, image_bytes)
        .await
        .context("write old")?;

    let vips_version_out = String::from_utf8(
        std::process::Command::new("vips")
            .arg("--version")
            .output()
            .context("vips version")?
            .stdout,
    )
    .context("vips version to utf8")?;
    let raw_split: Vec<&str> = vips_version_out.trim().split("-").collect();

    let vips_version = semver::Version::parse(raw_split[1]).context("parse vips version")?;

    let has_keep_flag = VersionReq::parse(">=8.15.0")
        .unwrap()
        .matches(&vips_version);

    let mut vips = std::process::Command::new("vips");
    let vips = vips.args([
        "jpegsave",
        &old,
        &new,
        "--Q",
        "75",
        "--optimize-coding",
        "--interlace",
        "--trellis-quant",
        "--overshoot-deringing",
    ]);

    let vips = if has_keep_flag {
        vips.args(["--keep", "none"])
    } else {
        vips.arg("--strip")
    };

    vips.output().context("execute vips")?;

    let result = tokio::fs::read(&new).await.context("read new")?;

    Ok(result)
}

fn original_path(id: &str) -> String {
    format!("{id}-original.jpg")
}
