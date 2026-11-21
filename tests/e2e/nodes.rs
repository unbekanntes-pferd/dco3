use dco3::{
    nodes::{NodeType, NodesFilter, TransferNodesRequest, UpdateFolderRequest},
    Folders, ListAllParams, Nodes,
};

use crate::harness::{test_data, Ctx, TestResult};

async fn child_ids(ctx: &Ctx, parent_id: u64) -> Result<Vec<u64>, dco3::DracoonClientError> {
    let mut ids: Vec<u64> = ctx
        .dracoon
        .nodes()
        .get_nodes(Some(parent_id), None, None)
        .await?
        .items
        .into_iter()
        .map(|node| node.id)
        .collect();
    ids.sort_unstable();
    Ok(ids)
}

pub async fn create_get_update_delete(ctx: &Ctx) -> TestResult {
    let parent = ctx.create_folder(None).await?;

    let folder = ctx.create_folder(Some(parent.id)).await?;
    assert_eq!(folder.node_type, NodeType::Folder);
    assert_eq!(folder.parent_id, Some(parent.id));

    let fetched = ctx.dracoon.nodes().get_node(folder.id).await?;
    assert_eq!(fetched.name, folder.name);

    let new_name = ctx.unique_name("renamed");
    let update = UpdateFolderRequest::builder()
        .with_name(&new_name)
        .with_notes("e2e")
        .build();
    let updated = ctx.dracoon.nodes().update_folder(folder.id, update).await?;
    assert_eq!(updated.name, new_name);
    assert_eq!(updated.notes.as_deref(), Some("e2e"));

    ctx.dracoon.nodes().delete_node(folder.id).await?;
    let err = ctx.dracoon.nodes().get_node(folder.id).await.unwrap_err();
    assert!(err.is_not_found(), "unexpected error: {err:?}");
    Ok(())
}

pub async fn list_children(ctx: &Ctx) -> TestResult {
    let parent = ctx.create_folder(None).await?;
    let mut expected = Vec::new();
    for _ in 0..3 {
        expected.push(ctx.create_folder(Some(parent.id)).await?.id);
    }
    expected.sort_unstable();

    assert_eq!(child_ids(ctx, parent.id).await?, expected);

    // name filter
    let child = ctx.dracoon.nodes().get_node(expected[1]).await?;
    let params = ListAllParams::builder()
        .with_filter(NodesFilter::name_equals(&child.name))
        .build();
    let filtered = ctx
        .dracoon
        .nodes()
        .get_nodes(Some(parent.id), None, Some(params))
        .await?;
    assert_eq!(
        filtered
            .items
            .iter()
            .map(|node| node.id)
            .collect::<Vec<_>>(),
        vec![child.id]
    );
    Ok(())
}

pub async fn get_node_from_path(ctx: &Ctx) -> TestResult {
    let parent = ctx.create_folder(None).await?;
    let folder = ctx.create_folder(Some(parent.id)).await?;
    // a similar name next to the exact one must not hide it
    let file = ctx
        .upload(
            &parent,
            &format!("{}.bak", folder.name),
            test_data(16, 1),
            None,
            None,
        )
        .await?;

    let parent_path = folder
        .parent_path
        .clone()
        .expect("folder has a parent path");
    let path = format!("{parent_path}{}/", folder.name);
    let node = ctx.dracoon.nodes().get_node_from_path(&path).await?;
    assert_eq!(node.map(|node| node.id), Some(folder.id));

    let file_path = format!("{parent_path}{}", file.name);
    let node = ctx.dracoon.nodes().get_node_from_path(&file_path).await?;
    assert_eq!(node.map(|node| node.id), Some(file.id));
    Ok(())
}

pub async fn search(ctx: &Ctx) -> TestResult {
    let parent = ctx.create_folder(None).await?;
    let folder = ctx.create_folder(Some(parent.id)).await?;

    let result = ctx
        .dracoon
        .nodes()
        .search_nodes(&folder.name, Some(parent.id), None, None)
        .await?;

    assert_eq!(
        result.items.iter().map(|node| node.id).collect::<Vec<_>>(),
        vec![folder.id]
    );
    Ok(())
}

pub async fn copy_and_move(ctx: &Ctx) -> TestResult {
    let source = ctx.create_folder(None).await?;
    let copy_target = ctx.create_folder(None).await?;
    let move_target = ctx.create_folder(None).await?;
    let file = ctx
        .upload(
            &source,
            &ctx.unique_name("file.bin"),
            test_data(1024, 2),
            None,
            None,
        )
        .await?;

    ctx.dracoon
        .nodes()
        .copy_nodes(TransferNodesRequest::from(vec![file.id]), copy_target.id)
        .await?;
    let copied = ctx
        .dracoon
        .nodes()
        .get_nodes(Some(copy_target.id), None, None)
        .await?;
    assert_eq!(
        copied
            .items
            .iter()
            .map(|node| node.name.as_str())
            .collect::<Vec<_>>(),
        vec![file.name.as_str()]
    );

    ctx.dracoon
        .nodes()
        .move_nodes(TransferNodesRequest::from(vec![file.id]), move_target.id)
        .await?;
    assert_eq!(child_ids(ctx, source.id).await?, Vec::<u64>::new());
    assert_eq!(child_ids(ctx, move_target.id).await?, vec![file.id]);
    Ok(())
}

pub async fn delete_nodes(ctx: &Ctx) -> TestResult {
    let parent = ctx.create_folder(None).await?;
    let first = ctx.create_folder(Some(parent.id)).await?;
    let second = ctx.create_folder(Some(parent.id)).await?;

    ctx.dracoon
        .nodes()
        .delete_nodes(vec![first.id, second.id].into())
        .await?;

    assert_eq!(child_ids(ctx, parent.id).await?, Vec::<u64>::new());
    Ok(())
}
