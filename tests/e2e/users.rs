use dco3::{
    users::{UpdateUserRequest, UsersFilter},
    ListAllParams, Users,
};

use crate::harness::{Ctx, TestResult};

pub async fn create_get_update_delete(ctx: &Ctx) -> TestResult {
    let user = ctx.create_user().await?;

    let fetched = ctx.dracoon.users().get_user(user.id, None).await?;
    assert_eq!(fetched.user_name, user.user_name);

    // effective roles are only included when requested
    let with_roles = ctx.dracoon.users().get_user(user.id, Some(true)).await?;
    assert!(with_roles.user_roles.is_some());

    let update = UpdateUserRequest::builder()
        .with_first_name("Updated")
        .build();
    let updated = ctx.dracoon.users().update_user(user.id, update).await?;
    assert_eq!(updated.first_name, "Updated");

    ctx.dracoon.users().delete_user(user.id).await?;
    let err = ctx
        .dracoon
        .users()
        .get_user(user.id, None)
        .await
        .unwrap_err();
    assert!(err.is_not_found(), "unexpected error: {err:?}");
    Ok(())
}

pub async fn list_filtered_by_user_name(ctx: &Ctx) -> TestResult {
    let user = ctx.create_user().await?;

    let params = ListAllParams::builder()
        .with_filter(UsersFilter::username_contains(&user.user_name))
        .build();
    let users = ctx
        .dracoon
        .users()
        .get_users(Some(params), None, None)
        .await?;

    assert_eq!(
        users.items.iter().map(|item| item.id).collect::<Vec<_>>(),
        vec![user.id]
    );
    Ok(())
}

pub async fn last_admin_rooms(ctx: &Ctx) -> TestResult {
    let user = ctx.create_user().await?;

    let rooms = ctx
        .dracoon
        .users()
        .get_user_last_admin_rooms(user.id)
        .await?;

    assert!(rooms.items.is_empty());
    Ok(())
}
