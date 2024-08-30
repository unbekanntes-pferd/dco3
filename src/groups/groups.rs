use async_trait::async_trait;
use reqwest::header;

use crate::{
    client::Connected,
    constants::{DRACOON_API_PREFIX, GROUPS_BASE, GROUPS_LAST_ADMIN_ROOMS, GROUPS_USERS},
    models::ListAllParams,
    utils::FromResponse,
    DracoonClientError,
};

use super::models::*;
use super::Groups;

#[async_trait]
impl Groups for GroupsEndpoint<Connected> {
    async fn get_groups(
        &self,
        params: Option<ListAllParams>,
    ) -> Result<GroupList, DracoonClientError> {
        let params = params.unwrap_or_default();
        let url_part = format!("/{DRACOON_API_PREFIX}/{GROUPS_BASE}");

        let mut api_url = self.client().build_api_url(&url_part);

        let filters = params.filter_to_string();
        let sorts = params.sort_to_string();

        api_url
            .query_pairs_mut()
            .extend_pairs(params.limit.map(|v| ("limit", v.to_string())))
            .extend_pairs(params.offset.map(|v| ("offset", v.to_string())))
            .extend_pairs(params.sort.map(|_| ("sort", sorts)))
            .extend_pairs(params.filter.map(|_| ("filter", filters)))
            .finish();

        let response = self
            .client()
            .http
            .get(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .send()
            .await?;

        GroupList::from_response(response).await
    }

    async fn create_group(&self, group: CreateGroupRequest) -> Result<Group, DracoonClientError> {
        let url_part = format!("/{DRACOON_API_PREFIX}/{GROUPS_BASE}");

        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .post(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .header(header::CONTENT_TYPE, "application/json")
            .json(&group)
            .send()
            .await?;

        Group::from_response(response).await
    }

    async fn get_group(&self, group_id: u64) -> Result<Group, DracoonClientError> {
        let url_part = format!("/{DRACOON_API_PREFIX}/{GROUPS_BASE}/{group_id}");

        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .get(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .send()
            .await?;

        Group::from_response(response).await
    }

    async fn update_group(
        &self,
        group_id: u64,
        group: UpdateGroupRequest,
    ) -> Result<Group, DracoonClientError> {
        let url_part = format!("/{DRACOON_API_PREFIX}/{GROUPS_BASE}/{group_id}");

        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .put(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .header(header::CONTENT_TYPE, "application/json")
            .json(&group)
            .send()
            .await?;

        Group::from_response(response).await
    }

    async fn delete_group(&self, group_id: u64) -> Result<(), DracoonClientError> {
        let url_part = format!("/{DRACOON_API_PREFIX}/{GROUPS_BASE}/{group_id}");

        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .delete(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .send()
            .await?;

        if response.status().is_server_error() || response.status().is_client_error() {
            return Err(DracoonClientError::from_response(response)
                .await
                .unwrap_or(DracoonClientError::Unknown));
        }

        Ok(())
    }

    async fn get_group_users(
        &self,
        group_id: u64,
        params: Option<ListAllParams>,
    ) -> Result<GroupUserList, DracoonClientError> {
        let url_part = format!("/{DRACOON_API_PREFIX}/{GROUPS_BASE}/{group_id}/{GROUPS_USERS}");

        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .get(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .send()
            .await?;

        GroupUserList::from_response(response).await
    }

    async fn add_group_users(
        &self,
        group_id: u64,
        user_ids: ChangeGroupMembersRequest,
    ) -> Result<Group, DracoonClientError> {
        let url_part = format!("/{DRACOON_API_PREFIX}/{GROUPS_BASE}/{group_id}/{GROUPS_USERS}");

        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .post(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .header(header::CONTENT_TYPE, "application/json")
            .json(&user_ids)
            .send()
            .await?;

        Group::from_response(response).await
    }

    async fn remove_group_users(
        &self,
        group_id: u64,
        user_ids: ChangeGroupMembersRequest,
    ) -> Result<Group, DracoonClientError> {
        let url_part = format!("/{DRACOON_API_PREFIX}/{GROUPS_BASE}/{group_id}/{GROUPS_USERS}");

        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .delete(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .header(header::CONTENT_TYPE, "application/json")
            .json(&user_ids)
            .send()
            .await?;

        Group::from_response(response).await
    }

    async fn get_group_last_admin_rooms(
        &self,
        group_id: u64,
    ) -> Result<LastAdminGroupRoomList, DracoonClientError> {
        let url_part =
            format!("/{DRACOON_API_PREFIX}/{GROUPS_BASE}/{group_id}/{GROUPS_LAST_ADMIN_ROOMS}");

        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .get(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .send()
            .await?;

        LastAdminGroupRoomList::from_response(response).await
    }
}
