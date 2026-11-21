//! the e2e user's own account is shared state: read-only except one update that is restored

use dco3::{user::UpdateUserAccountRequest, User};

use crate::harness::{Ctx, TestResult};

pub async fn account_information(ctx: &Ctx) -> TestResult {
    let account = ctx.dracoon.user().get_user_account().await?;

    assert_eq!(account.user_name, ctx.config.username);
    assert_eq!(account.id, ctx.dracoon.get_user_info().await?.id);
    Ok(())
}

pub async fn update_account_information(ctx: &Ctx) -> TestResult {
    let account = ctx.dracoon.user().get_user_account().await?;
    let dracoon = ctx.dracoon.clone();
    let last_name = account.last_name.clone();
    ctx.defer("restore account last name", async move {
        let restore = UpdateUserAccountRequest::builder()
            .with_last_name(last_name)
            .build();
        dracoon
            .user()
            .update_user_account(restore)
            .await
            .map(|_| ())
    });

    let new_name = ctx.unique_name("");
    let update = UpdateUserAccountRequest::builder()
        .with_last_name(&new_name)
        .build();
    let updated = ctx.dracoon.user().update_user_account(update).await?;

    assert_eq!(updated.last_name, new_name);
    Ok(())
}
