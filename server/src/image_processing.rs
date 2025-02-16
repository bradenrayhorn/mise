use anyhow::{anyhow, Context};
use base64::Engine;
use ring::rand::SecureRandom;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

pub struct ImageProcessor {
    working_directory: String,
    vips_has_keep_flag: bool,
}

impl ImageProcessor {
    pub async fn new() -> Result<ImageProcessor, Error> {
        let working_dir = "/tmp/mise-images";
        if !tokio::fs::try_exists(working_dir)
            .await
            .context("Checking working dir exists")?
        {
            tokio::fs::create_dir(working_dir)
                .await
                .context("Creating working dir")?;
        }

        let vips_version = ImageProcessor::find_vips_version()?;
        let minimum_vips_version_with_keep_flag = (8, 15);

        let has_keep_flag =
            is_at_least_major_minor(vips_version, minimum_vips_version_with_keep_flag);

        Ok(ImageProcessor {
            working_directory: working_dir.to_owned(),
            vips_has_keep_flag: has_keep_flag,
        })
    }

    fn find_vips_version() -> Result<(i32, i32), Error> {
        let vips_version_out = String::from_utf8(
            std::process::Command::new("vips")
                .arg("--version")
                .output()
                .context("vips version")?
                .stdout,
        )
        .context("vips version to utf8")?;
        let raw_split: Vec<&str> = vips_version_out.trim().split('-').collect();

        let parts: Vec<&str> = raw_split[1].split('.').collect();
        if parts.len() != 3 {
            return Err(anyhow!("invalid vips version: {}", raw_split[1]).into());
        }
        let major: i32 = parts[0].parse().context("expected number")?;
        let minor: i32 = parts[1].parse().context("expected number")?;

        Ok((major, minor))
    }

    pub async fn process_image(&self, image: Vec<u8>) -> Result<Vec<u8>, Error> {
        let mut bytes: [u8; 32] = [0; 32];
        ring::rand::SystemRandom::new()
            .fill(&mut bytes)
            .map_err(|_| anyhow!("Could not generate random path."))?;
        let random_file = base64::engine::general_purpose::URL_SAFE
            .encode(bytes)
            .to_string();

        let file_dir = format!("{}/{random_file}/", self.working_directory);

        tokio::fs::create_dir(&file_dir)
            .await
            .context("Creating working dir for upload")?;

        let file_original = format!("{file_dir}-original");
        let file_jpg = format!("{file_dir}-jpg.jpg");
        let file_rotated = format!("{file_dir}-rotated.jpg");
        let file_stripped = format!("{file_dir}-stripped.jpg");

        tokio::fs::write(&file_original, image)
            .await
            .context("write old")?;

        let mut vips_command = std::process::Command::new("sh");
        /*
         * 1. Convert from any format to jpeg
         * 2. Encode jpeg rotation, iOS adds rotation as metadata and we want to strip that
         * 3. Save as jpeg again, but without metadata
         */
        let vips_command = vips_command.args([
            "-c",
            &format!(
                "vips jpegsave {} {} &&
                     vips autorot {} {} &&
                     vips jpegsave {} {} --Q 75 {}",
                &file_original,
                &file_jpg,
                &file_jpg,
                &file_rotated,
                &file_rotated,
                &file_stripped,
                if self.vips_has_keep_flag {
                    "--keep icc"
                } else {
                    "--strip"
                },
            ),
        ]);

        // execute the command
        let output = vips_command.output().context("execute vips commands")?;
        if !output.status.success() {
            return Err(Error::from(anyhow!(
                "vips command error: {}",
                String::from_utf8(output.stderr).context("stderr not utf8")?,
            )));
        }

        let result = tokio::fs::read(&file_stripped)
            .await
            .context("read final jpeg")?;

        // remove working files
        tokio::fs::remove_dir_all(&file_dir)
            .await
            .context("cleanup file dir")?;

        Ok(result)
    }
}

fn is_at_least_major_minor(installed: (i32, i32), minimum: (i32, i32)) -> bool {
    match installed.0.cmp(&minimum.0) {
        std::cmp::Ordering::Less => false,
        std::cmp::Ordering::Equal => match installed.1.cmp(&minimum.1) {
            std::cmp::Ordering::Less => false,
            std::cmp::Ordering::Equal | std::cmp::Ordering::Greater => true,
        },
        std::cmp::Ordering::Greater => true,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_at_least_major_minor() {
        let test_cases = vec![
            // (installed, minimum, expected)
            ((1, 0), (1, 1), false),
            ((1, 1), (1, 1), true),
            ((2, 0), (1, 1), true),
            ((0, 2), (1, 1), false),
        ];

        for (installed, minimum, expected) in test_cases {
            let result = is_at_least_major_minor(installed, minimum);
            assert_eq!(
                result, expected,
                "installed: {:?}, minimum: {:?}, expected to be: {}",
                installed, minimum, expected
            );
        }
    }
}
