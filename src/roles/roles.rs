use async_trait::async_trait;
use reqwest::header;

use crate::{
    auth::Connected,
    constants::{DRACOON_API_PREFIX, ROLES_BASE, ROLES_GROUPS, ROLES_USERS},
    utils::FromResponse,
    DracoonClientError, ListAllParams,
};

use super::{
    AssignRoleBatchRequest, RevokeRoleBatchRequest, RoleGroupList, RoleList, RoleUserList, Roles,
    RolesEndpoint,
};

#[async_trait]
impl Roles for RolesEndpoint<Connected> {
    async fn get_roles(&self) -> Result<RoleList, DracoonClientError> {
        let url_part = format!("{DRACOON_API_PREFIX}/{ROLES_BASE}");

        let url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .get(url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        RoleList::from_response(response).await
    }

    async fn get_all_groups_with_role(
        &self,
        role_id: u64,
        params: Option<ListAllParams>,
    ) -> Result<RoleGroupList, DracoonClientError> {
        let url_part = format!("{DRACOON_API_PREFIX}/{ROLES_BASE}/{role_id}/{ROLES_GROUPS}");

        let url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .get(url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        RoleGroupList::from_response(response).await
    }

    async fn assign_role_to_groups(
        &self,
        role_id: u64,
        group_ids: AssignRoleBatchRequest,
    ) -> Result<RoleGroupList, DracoonClientError> {
        let url_part = format!("{DRACOON_API_PREFIX}/{ROLES_BASE}/{role_id}/{ROLES_GROUPS}");

        let url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .post(url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .header(header::CONTENT_TYPE, "application/json")
            .json(&group_ids)
            .send()
            .await?;

        RoleGroupList::from_response(response).await
    }

    async fn revoke_role_from_groups(
        &self,
        role_id: u64,
        group_ids: RevokeRoleBatchRequest,
    ) -> Result<RoleGroupList, DracoonClientError> {
        let url_part = format!("{DRACOON_API_PREFIX}/{ROLES_BASE}/{role_id}/{ROLES_GROUPS}");

        let url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .delete(url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .header(header::CONTENT_TYPE, "application/json")
            .json(&group_ids)
            .send()
            .await?;

        RoleGroupList::from_response(response).await
    }

    async fn get_all_users_with_role(
        &self,
        role_id: u64,
        params: Option<ListAllParams>,
    ) -> Result<RoleUserList, DracoonClientError> {
        let url_part = format!("{DRACOON_API_PREFIX}/{ROLES_BASE}/{role_id}/{ROLES_USERS}");

        let url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .get(url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        RoleUserList::from_response(response).await
    }

    async fn assign_role_to_users(
        &self,
        role_id: u64,
        user_ids: AssignRoleBatchRequest,
    ) -> Result<RoleUserList, DracoonClientError> {
        let url_part = format!("{DRACOON_API_PREFIX}/{ROLES_BASE}/{role_id}/{ROLES_USERS}");

        let url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .post(url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .header(header::CONTENT_TYPE, "application/json")
            .json(&user_ids)
            .send()
            .await?;

        RoleUserList::from_response(response).await
    }

    async fn revoke_role_from_users(
        &self,
        role_id: u64,
        user_ids: RevokeRoleBatchRequest,
    ) -> Result<RoleUserList, DracoonClientError> {
        let url_part = format!("{DRACOON_API_PREFIX}/{ROLES_BASE}/{role_id}/{ROLES_USERS}");

        let url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .delete(url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .header(header::CONTENT_TYPE, "application/json")
            .json(&user_ids)
            .send()
            .await?;

        RoleUserList::from_response(response).await
    }
}
