use dco3::{
    nodes::Node,
    shares::{
        DownloadSharesFilter, UpdateDownloadShareRequest, UpdateUploadShareRequest,
        UploadSharesFilter,
    },
    DownloadShares, ListAllParams, UploadShares,
};

use crate::harness::{test_data, Ctx, TestResult};

/// folder with a small file for download shares
async fn shared_file(ctx: &Ctx) -> Result<Node, dco3::DracoonClientError> {
    let folder = ctx.create_folder(None).await?;
    ctx.upload(
        &folder,
        &ctx.unique_name("shared.bin"),
        test_data(1024, 3),
        None,
        None,
    )
    .await
}

async fn download_share_ids(ctx: &Ctx, node_id: u64) -> Result<Vec<u64>, dco3::DracoonClientError> {
    let params = ListAllParams::builder()
        .with_filter(DownloadSharesFilter::node_id_equals(node_id))
        .build();
    let mut ids: Vec<u64> = ctx
        .dracoon
        .shares()
        .get_download_shares(Some(params))
        .await?
        .items
        .into_iter()
        .map(|share| share.id)
        .collect();
    ids.sort_unstable();
    Ok(ids)
}

async fn upload_share_ids(ctx: &Ctx, target_id: u64) -> Result<Vec<u64>, dco3::DracoonClientError> {
    let params = ListAllParams::builder()
        .with_filter(UploadSharesFilter::target_id_equals(target_id))
        .build();
    let mut ids: Vec<u64> = ctx
        .dracoon
        .shares()
        .get_upload_shares(Some(params))
        .await?
        .items
        .into_iter()
        .map(|share| share.id)
        .collect();
    ids.sort_unstable();
    Ok(ids)
}

pub async fn download_share_create_get_update_delete(ctx: &Ctx) -> TestResult {
    let file = shared_file(ctx).await?;
    let share = ctx.create_download_share(file.id, None).await?;
    assert_eq!(share.node_id, file.id);

    let fetched = ctx.dracoon.shares().get_download_share(share.id).await?;
    assert_eq!(fetched.name, share.name);
    assert_eq!(fetched.notes.as_deref(), Some("e2e"));

    let update = UpdateDownloadShareRequest::builder()
        .with_max_downloads(5)
        .with_notes("updated")
        .build();
    let updated = ctx
        .dracoon
        .shares()
        .update_download_share(share.id, update)
        .await?;
    assert_eq!(updated.max_downloads, Some(5));
    assert_eq!(updated.notes.as_deref(), Some("updated"));

    ctx.dracoon.shares().delete_download_share(share.id).await?;
    let err = ctx
        .dracoon
        .shares()
        .get_download_share(share.id)
        .await
        .unwrap_err();
    assert!(err.is_not_found(), "unexpected error: {err:?}");
    Ok(())
}

pub async fn download_shares_filtered_by_node(ctx: &Ctx) -> TestResult {
    let file = shared_file(ctx).await?;
    let share = ctx.create_download_share(file.id, None).await?;

    assert_eq!(download_share_ids(ctx, file.id).await?, vec![share.id]);
    Ok(())
}

pub async fn download_shares_bulk_delete(ctx: &Ctx) -> TestResult {
    let file = shared_file(ctx).await?;
    let mut share_ids = vec![
        ctx.create_download_share(file.id, None).await?.id,
        ctx.create_download_share(file.id, None).await?.id,
    ];
    share_ids.sort_unstable();
    assert_eq!(download_share_ids(ctx, file.id).await?, share_ids);

    ctx.dracoon
        .shares()
        .delete_download_shares(share_ids.into())
        .await?;

    assert_eq!(download_share_ids(ctx, file.id).await?, Vec::<u64>::new());
    Ok(())
}

pub async fn download_share_password_protected(ctx: &Ctx) -> TestResult {
    let file = shared_file(ctx).await?;

    let share = ctx
        .create_download_share(file.id, Some("VerySecret123!"))
        .await?;

    assert_eq!(share.is_protected, Some(true));
    Ok(())
}

pub async fn upload_share_create_get_update_delete(ctx: &Ctx) -> TestResult {
    let folder = ctx.create_folder(None).await?;
    let share = ctx.create_upload_share(folder.id, Some(2)).await?;
    assert_eq!(share.target_id, folder.id);

    let fetched = ctx.dracoon.shares().get_upload_share(share.id).await?;
    assert_eq!(fetched.name, share.name);
    assert_eq!(fetched.max_slots, Some(2));

    let update = UpdateUploadShareRequest::builder()
        .with_notes("updated")
        .with_reset_max_slots(true)
        .build();
    let updated = ctx
        .dracoon
        .shares()
        .update_upload_share(share.id, update)
        .await?;
    assert_eq!(updated.notes.as_deref(), Some("updated"));
    assert_eq!(updated.max_slots, None);

    ctx.dracoon.shares().delete_upload_share(share.id).await?;
    let err = ctx
        .dracoon
        .shares()
        .get_upload_share(share.id)
        .await
        .unwrap_err();
    assert!(err.is_not_found(), "unexpected error: {err:?}");
    Ok(())
}

pub async fn upload_shares_filtered_by_target(ctx: &Ctx) -> TestResult {
    let folder = ctx.create_folder(None).await?;
    let share = ctx.create_upload_share(folder.id, None).await?;

    assert_eq!(upload_share_ids(ctx, folder.id).await?, vec![share.id]);
    Ok(())
}

pub async fn upload_shares_bulk_delete(ctx: &Ctx) -> TestResult {
    let folder = ctx.create_folder(None).await?;
    let mut share_ids = vec![
        ctx.create_upload_share(folder.id, None).await?.id,
        ctx.create_upload_share(folder.id, None).await?.id,
    ];
    share_ids.sort_unstable();
    assert_eq!(upload_share_ids(ctx, folder.id).await?, share_ids);

    ctx.dracoon
        .shares()
        .delete_upload_shares(share_ids.into())
        .await?;

    assert_eq!(upload_share_ids(ctx, folder.id).await?, Vec::<u64>::new());
    Ok(())
}
