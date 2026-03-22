use bytes::BytesMut;
use dco3_crypto::{DracoonCrypto, DracoonCryptoError, FileEncryptor, PlainFileKey};
use tokio::io::{AsyncRead, AsyncReadExt, BufReader};

use crate::client::errors::DracoonClientError;

pub(crate) enum EncryptionState {
    Encrypting(FileEncryptor),
    Finalized,
}

pub(crate) struct StreamingEncryptedUpload<R> {
    reader: BufReader<R>,
    state: EncryptionState,
    read_buffer: Vec<u8>,
    pending_ciphertext: BytesMut,
    plain_file_key: Option<PlainFileKey>,
    emitted_any_chunk: bool,
}

impl<R: AsyncRead + Unpin> StreamingEncryptedUpload<R> {
    pub(crate) fn new(
        reader: BufReader<R>,
        read_buffer_len: usize,
    ) -> Result<Self, DracoonClientError> {
        Ok(Self {
            reader,
            state: EncryptionState::Encrypting(DracoonCrypto::file_encryptor()?),
            read_buffer: vec![0u8; read_buffer_len],
            pending_ciphertext: BytesMut::with_capacity(read_buffer_len),
            plain_file_key: None,
            emitted_any_chunk: false,
        })
    }

    async fn fill_pending_ciphertext(
        &mut self,
        target_len: usize,
    ) -> Result<(), DracoonClientError> {
        while self.pending_ciphertext.len() < target_len {
            match &mut self.state {
                EncryptionState::Encrypting(encryptor) => {
                    let chunk = self
                        .reader
                        .read(&mut self.read_buffer)
                        .await
                        .map_err(|_| DracoonClientError::IoError)?;

                    if chunk == 0 {
                        self.finalize_encryptor()?;
                        break;
                    }

                    let encrypted = encryptor.update(&self.read_buffer[..chunk])?;
                    self.pending_ciphertext.extend_from_slice(&encrypted);
                }
                EncryptionState::Finalized => break,
            }
        }

        Ok(())
    }

    fn finalize_encryptor(&mut self) -> Result<(), DracoonClientError> {
        let state = std::mem::replace(&mut self.state, EncryptionState::Finalized);

        match state {
            EncryptionState::Encrypting(encryptor) => {
                let finalized = encryptor.finalize()?;
                self.pending_ciphertext
                    .extend_from_slice(&finalized.final_chunk);
                self.plain_file_key = Some(finalized.plain_file_key);
            }
            EncryptionState::Finalized => {}
        }

        Ok(())
    }

    pub(crate) async fn next_chunk(
        &mut self,
        target_len: usize,
    ) -> Result<Option<bytes::Bytes>, DracoonClientError> {
        self.fill_pending_ciphertext(target_len).await?;

        if self.pending_ciphertext.is_empty() {
            if !self.emitted_any_chunk && self.plain_file_key.is_some() {
                self.emitted_any_chunk = true;
                return Ok(Some(bytes::Bytes::new()));
            }
            return Ok(None);
        }

        let emit_len = self.pending_ciphertext.len().min(target_len);
        self.emitted_any_chunk = true;
        Ok(Some(self.pending_ciphertext.split_to(emit_len).freeze()))
    }

    pub(crate) fn plain_file_key(&self) -> Option<&PlainFileKey> {
        self.plain_file_key.as_ref()
    }

    pub(crate) fn into_plain_file_key(self) -> Result<PlainFileKey, DracoonClientError> {
        if !matches!(self.state, EncryptionState::Finalized) || !self.pending_ciphertext.is_empty()
        {
            return Err(DracoonClientError::CryptoError(
                DracoonCryptoError::CrypterOperationFailed(
                    "Plain file key is only available after the encrypted upload stream has been fully drained."
                        .to_string(),
                ),
            ));
        }

        self.plain_file_key.ok_or_else(|| {
            DracoonClientError::CryptoError(DracoonCryptoError::CrypterOperationFailed(
                "Encrypted upload stream finished without producing a plain file key.".to_string(),
            ))
        })
    }
}
