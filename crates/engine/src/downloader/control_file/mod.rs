mod v1;

use std::{
    io::SeekFrom,
    path::{Path, PathBuf},
};

use snafu::ResultExt;
use tokio::{
    fs::File,
    io::{AsyncSeekExt, AsyncWriteExt},
};

use crate::{downloader::TransferStatus, error, error::Error};

pub struct ControlFile {
    file: File,

    file_path: PathBuf,

    url: reqwest::Url,
}

impl ControlFile {
    pub async fn new<P>(
        file_path: P,
        url: reqwest::Url,
    ) -> Result<(Self, Option<TransferStatus>), Error>
    where
        P: AsRef<Path> + Send,
    {
        let file_path = file_path.as_ref();
        let file_name = PathBuf::from(format!(
            "{name}.{suffix}",
            name = file_path
                .file_name()
                .expect("file_name is available; qed")
                .to_str()
                .expect("file_name is available; qed"),
            suffix = caracal_base::CONTROL_FILE_SUFFIX
        ));
        let file_path = if let Some(parent) = file_path.parent() {
            [parent, &file_name].into_iter().collect()
        } else {
            file_name
        };

        let transfer_status = tokio::fs::read(&file_path).await.map_or(None, |contents| {
            serde_json::from_slice::<v1::Control>(&contents).ok().map(TransferStatus::from)
        });

        let file = tokio::fs::OpenOptions::new()
            .create(true)
            .write(true)
            .read(false)
            .open(&file_path)
            .await
            .context(error::CreateControlFileSnafu { file_path: file_path.clone() })?;

        Ok((Self { file, file_path, url }, transfer_status))
    }

    pub async fn update_progress(&mut self, transfer_status: &TransferStatus) -> Result<(), Error> {
        let control = v1::Control {
            schema: 1,
            uris: vec![self.url.to_string()],
            content_length: Some(transfer_status.content_length),
            chunks: transfer_status.chunks().into_iter().map(Into::into).collect(),
        };

        self.file
            .set_len(0)
            .await
            .with_context(|_| error::ResizeFileSnafu { file_path: self.file_path.clone() })?;

        let _ = self
            .file
            .seek(SeekFrom::Start(0))
            .await
            .with_context(|_| error::SeekFileSnafu { file_path: self.file_path.clone() })?;

        self.file
            .write_all(
                serde_json::to_string(&control).expect("Control is serializable; qed").as_bytes(),
            )
            .await
            .with_context(|_| error::WriteFileSnafu { file_path: self.file_path.clone() })?;

        Ok(())
    }

    pub async fn flush(&mut self) -> Result<(), Error> {
        self.file
            .flush()
            .await
            .with_context(|_| error::FlushFileSnafu { file_path: self.file_path.clone() })?;
        self.file
            .sync_all()
            .await
            .with_context(|_| error::FlushFileSnafu { file_path: self.file_path.clone() })?;
        Ok(())
    }

    pub async fn remove(self) {
        drop(self.file);
        if let Err(err) = tokio::fs::remove_file(&self.file_path).await {
            tracing::warn!(
                "Error occurs while removing control file `{}`, error: {err}",
                self.file_path.display()
            );
        }
    }
}
