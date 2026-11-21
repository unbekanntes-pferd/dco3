use dco3::{
    nodes::{
        ConfigRoomRequest, NodePermissionsBuilder, NodeType, RoomGroupsAddBatchRequestItem,
        RoomUsersAddBatchRequestItem, UpdateRoomRequest,
    },
    Nodes, Rooms,
};

use crate::harness::{Ctx, TestResult};

pub async fn create_get_update_delete(ctx: &Ctx) -> TestResult {
    let room = ctx.create_room(false).await?;
    assert_eq!(room.node_type, NodeType::Room);
    assert_eq!(room.parent_id, Some(ctx.room.id));

    let fetched = ctx.dracoon.nodes().get_node(room.id).await?;
    assert_eq!(fetched.name, room.name);

    let new_name = ctx.unique_name("renamed");
    let update = UpdateRoomRequest::builder()
        .with_name(&new_name)
        .with_notes("e2e")
        .build();
    let updated = ctx.dracoon.nodes().update_room(room.id, update).await?;
    assert_eq!(updated.name, new_name);
    assert_eq!(updated.notes.as_deref(), Some("e2e"));

    ctx.dracoon.nodes().delete_node(room.id).await?;
    let err = ctx.dracoon.nodes().get_node(room.id).await.unwrap_err();
    assert!(err.is_not_found(), "unexpected error: {err:?}");
    Ok(())
}

pub async fn config(ctx: &Ctx) -> TestResult {
    let room = ctx.create_room(true).await?;

    let config = ConfigRoomRequest::builder()
        .with_recycle_bin_retention_period(1)
        .build();
    let configured = ctx.dracoon.nodes().config_room(room.id, config).await?;

    assert_eq!(configured.recycle_bin_retention_period, Some(1));
    Ok(())
}

pub async fn room_users(ctx: &Ctx) -> TestResult {
    let room = ctx.create_room(true).await?;
    let user = ctx.create_user().await?;

    let permissions = NodePermissionsBuilder::new().with_read(true).build();
    ctx.dracoon
        .nodes()
        .update_room_users(
            room.id,
            vec![RoomUsersAddBatchRequestItem::new(user.id, permissions)].into(),
        )
        .await?;

    let room_users = ctx.dracoon.nodes().get_room_users(room.id, None).await?;
    let room_user = room_users
        .items
        .iter()
        .find(|item| item.user_info.id == user.id as i64)
        .expect("added user is a room user");
    assert!(room_user.is_granted);
    let permissions = room_user
        .permissions
        .as_ref()
        .expect("granted user has permissions");
    assert!(permissions.read);
    assert!(!permissions.manage);

    ctx.dracoon
        .nodes()
        .delete_room_users(room.id, vec![user.id].into())
        .await?;
    let room_users = ctx.dracoon.nodes().get_room_users(room.id, None).await?;
    assert!(!room_users
        .items
        .iter()
        .any(|item| item.user_info.id == user.id as i64 && item.is_granted));
    Ok(())
}

pub async fn room_groups(ctx: &Ctx) -> TestResult {
    let room = ctx.create_room(true).await?;
    let group = ctx.create_group().await?;

    let permissions = NodePermissionsBuilder::new().with_read(true).build();
    ctx.dracoon
        .nodes()
        .update_room_groups(
            room.id,
            vec![RoomGroupsAddBatchRequestItem::new(
                group.id,
                permissions,
                None,
            )]
            .into(),
        )
        .await?;

    let room_groups = ctx.dracoon.nodes().get_room_groups(room.id, None).await?;
    let room_group = room_groups
        .items
        .iter()
        .find(|item| item.id == group.id)
        .expect("added group is a room group");
    assert!(room_group.is_granted);

    ctx.dracoon
        .nodes()
        .delete_room_groups(room.id, vec![group.id].into())
        .await?;
    let room_groups = ctx.dracoon.nodes().get_room_groups(room.id, None).await?;
    assert!(!room_groups
        .items
        .iter()
        .any(|item| item.id == group.id && item.is_granted));
    Ok(())
}
