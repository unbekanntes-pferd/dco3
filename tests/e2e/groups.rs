use dco3::{
    groups::{GroupUsersFilter, GroupsFilter, UpdateGroupRequest},
    Groups, ListAllParams,
};

use crate::harness::{Ctx, TestResult};

pub async fn create_get_update_delete(ctx: &Ctx) -> TestResult {
    let group = ctx.create_group().await?;

    let fetched = ctx.dracoon.groups().get_group(group.id).await?;
    assert_eq!(fetched.name, group.name);

    let new_name = ctx.unique_name("renamed");
    let updated = ctx
        .dracoon
        .groups()
        .update_group(group.id, UpdateGroupRequest::name(&new_name))
        .await?;
    assert_eq!(updated.name, new_name);

    ctx.dracoon.groups().delete_group(group.id).await?;
    let err = ctx.dracoon.groups().get_group(group.id).await.unwrap_err();
    assert!(err.is_not_found(), "unexpected error: {err:?}");
    Ok(())
}

pub async fn list_filtered_by_name(ctx: &Ctx) -> TestResult {
    let group = ctx.create_group().await?;

    let params = ListAllParams::builder()
        .with_filter(GroupsFilter::name_contains(&group.name))
        .build();
    let groups = ctx.dracoon.groups().get_groups(Some(params)).await?;

    assert_eq!(
        groups.items.iter().map(|item| item.id).collect::<Vec<_>>(),
        vec![group.id]
    );
    Ok(())
}

pub async fn members(ctx: &Ctx) -> TestResult {
    let group = ctx.create_group().await?;
    let mut user_ids = vec![ctx.create_user().await?.id, ctx.create_user().await?.id];
    user_ids.sort_unstable();

    let updated = ctx
        .dracoon
        .groups()
        .add_group_users(group.id, user_ids.clone().into())
        .await?;
    assert_eq!(updated.cnt_users, Some(2));

    let params = ListAllParams::builder()
        .with_filter(GroupUsersFilter::is_member(true))
        .build();
    let members = ctx
        .dracoon
        .groups()
        .get_group_users(group.id, Some(params))
        .await?;
    let mut member_ids: Vec<u64> = members
        .items
        .iter()
        .map(|item| item.user_info.id as u64)
        .collect();
    member_ids.sort_unstable();
    assert_eq!(member_ids, user_ids);

    let updated = ctx
        .dracoon
        .groups()
        .remove_group_users(group.id, user_ids.into())
        .await?;
    assert_eq!(updated.cnt_users, Some(0));
    Ok(())
}

pub async fn last_admin_rooms(ctx: &Ctx) -> TestResult {
    let group = ctx.create_group().await?;

    let rooms = ctx
        .dracoon
        .groups()
        .get_group_last_admin_rooms(group.id)
        .await?;

    assert!(rooms.items.is_empty());
    Ok(())
}
