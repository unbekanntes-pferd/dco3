//! uploads / downloads (unencrypted and encrypted) and live validation of `download_range`

use dco3::{
    constants::MIN_S3_UPLOAD_CHUNK_SIZE,
    nodes::{Node, ResolutionStrategy},
    Download,
};

use crate::harness::{random_hex, test_data, Ctx, TestResult};

/// smallest chunk size valid for both storage types (S3 requires 5 MiB parts)
const CHUNK_SIZE: usize = MIN_S3_UPLOAD_CHUNK_SIZE;
/// 2.5 chunks: several parts and a partial last chunk
const MULTI_CHUNK_SIZE: usize = CHUNK_SIZE * 2 + CHUNK_SIZE / 2;

fn seed() -> u64 {
    u64::from_str_radix(&random_hex(), 16).unwrap_or(42)
}

/// uploads `size` random bytes into `parent`, checks the node and downloads it again
async fn assert_round_trip(
    ctx: &Ctx,
    parent: &Node,
    size: usize,
    chunk_size: Option<usize>,
) -> Result<Node, Box<dyn std::error::Error + Send + Sync>> {
    let data = test_data(size, seed());
    let name = ctx.unique_name("upload.bin");

    let node = ctx
        .upload(parent, &name, data.clone(), chunk_size, None)
        .await?;
    assert_eq!(node.name, name);
    assert_eq!(node.size, Some(size as u64));

    let downloaded = ctx.download(&node).await?;
    assert!(
        downloaded == data,
        "downloaded content differs ({} bytes)",
        downloaded.len()
    );
    Ok(node)
}

pub async fn small_file(ctx: &Ctx) -> TestResult {
    let folder = ctx.create_folder(None).await?;
    assert_round_trip(ctx, &folder, 1024, None).await?;
    Ok(())
}

pub async fn empty_file(ctx: &Ctx) -> TestResult {
    let folder = ctx.create_folder(None).await?;
    assert_round_trip(ctx, &folder, 0, None).await?;
    Ok(())
}

pub async fn multi_chunk_file(ctx: &Ctx) -> TestResult {
    let folder = ctx.create_folder(None).await?;
    assert_round_trip(ctx, &folder, MULTI_CHUNK_SIZE, Some(CHUNK_SIZE)).await?;
    Ok(())
}

pub async fn conflict_autorename(ctx: &Ctx) -> TestResult {
    let folder = ctx.create_folder(None).await?;
    let name = ctx.unique_name("conflict.bin");

    let first = ctx
        .upload(&folder, &name, test_data(64, 1), None, None)
        .await?;
    let second = ctx
        .upload(
            &folder,
            &name,
            test_data(64, 2),
            None,
            Some(ResolutionStrategy::AutoRename),
        )
        .await?;

    assert_ne!(first.id, second.id);
    assert_ne!(first.name, second.name);
    Ok(())
}

/// live validation of byte-range reads (see docs/download-range-plan.md §6)
pub async fn download_range(ctx: &Ctx) -> TestResult {
    let folder = ctx.create_folder(None).await?;
    // multiple MiB, odd size
    let size = 3 * 1024 * 1024 + 13;
    let data = test_data(size, seed());
    let node = ctx
        .upload(
            &folder,
            &ctx.unique_name("range.bin"),
            data.clone(),
            None,
            None,
        )
        .await?;
    let size = size as u64;

    // unaligned ranges at the start, in the middle and across the end of the file
    for (offset, length) in [
        (0u64, 1usize),
        (1, 4096),
        (1_000_003, 4097),
        (size - 10, 10),
        (size - 3, 100),
    ] {
        let bytes = ctx.dracoon.download_range(&node, offset, length).await?;
        let end = (offset + length as u64).min(size) as usize;
        assert!(
            bytes.as_ref() == &data[offset as usize..end],
            "range {offset}+{length}: content differs ({} bytes)",
            bytes.len()
        );
    }

    // at / beyond the end of the file
    assert!(ctx
        .dracoon
        .download_range(&node, size, 10)
        .await?
        .is_empty());
    assert!(ctx
        .dracoon
        .download_range(&node, size + 1, 10)
        .await?
        .is_empty());

    // reassemble the file from 1 MiB ranges
    let mut reassembled = Vec::with_capacity(data.len());
    let mut offset = 0;
    while offset < size {
        let bytes = ctx
            .dracoon
            .download_range(&node, offset, 1024 * 1024)
            .await?;
        assert!(!bytes.is_empty(), "no progress at offset {offset}");
        offset += bytes.len() as u64;
        reassembled.extend_from_slice(&bytes);
    }
    assert!(reassembled == data, "reassembled content differs");
    Ok(())
}

// encrypted room (opt-in: E2E_CRYPTO=1)

async fn encrypted_round_trip(ctx: &Ctx, size: usize, chunk_size: Option<usize>) -> TestResult {
    let folder = ctx.create_folder(Some(ctx.crypto_room().id)).await?;
    let node = assert_round_trip(ctx, &folder, size, chunk_size).await?;
    assert_eq!(node.is_encrypted, Some(true));
    Ok(())
}

pub async fn encrypted_small_file(ctx: &Ctx) -> TestResult {
    encrypted_round_trip(ctx, 1024, None).await
}

pub async fn encrypted_empty_file(ctx: &Ctx) -> TestResult {
    encrypted_round_trip(ctx, 0, None).await
}

pub async fn encrypted_multi_chunk_file(ctx: &Ctx) -> TestResult {
    encrypted_round_trip(ctx, MULTI_CHUNK_SIZE, Some(CHUNK_SIZE)).await
}

pub async fn encrypted_download_range_is_unsupported(ctx: &Ctx) -> TestResult {
    let folder = ctx.create_folder(Some(ctx.crypto_room().id)).await?;
    let node = ctx
        .upload(
            &folder,
            &ctx.unique_name("range.bin"),
            test_data(1024, 7),
            None,
            None,
        )
        .await?;

    let err = ctx.dracoon.download_range(&node, 0, 16).await.unwrap_err();

    assert!(err.is_unsupported(), "unexpected error: {err:?}");
    Ok(())
}
