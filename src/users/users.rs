use async_trait::async_trait;
use reqwest::header;

use crate::{
    client::Connected,
    constants::{DRACOON_API_PREFIX, USERS_BASE, USERS_LAST_ADMIN_ROOMS},
    utils::FromResponse,
    DracoonClientError, ListAllParams, Users,
};

use super::{
    CreateUserRequest, LastAdminUserRoomList, UpdateUserRequest, UserData, UserList, UsersEndpoint,
};

#[async_trait]
impl Users for UsersEndpoint<Connected> {
    async fn get_users(
        &self,
        params: Option<ListAllParams>,
        include_roles: Option<bool>,
        include_attributes: Option<bool>,
    ) -> Result<UserList, DracoonClientError> {
        let params = params.unwrap_or_default();
        let url_part = format!("/{DRACOON_API_PREFIX}/{USERS_BASE}");
        let mut api_url = self.client().build_api_url(&url_part);

        let filters = params.filter_to_string();
        let sorts = params.sort_to_string();

        api_url
            .query_pairs_mut()
            .extend_pairs(params.limit.map(|v| ("limit", v.to_string())))
            .extend_pairs(params.offset.map(|v| ("offset", v.to_string())))
            .extend_pairs(params.sort.map(|_| ("sort", sorts)))
            .extend_pairs(params.filter.map(|_| ("filter", filters)))
            .extend_pairs(include_roles.map(|v| ("include_roles", v.to_string())))
            .extend_pairs(include_attributes.map(|v| ("include_attributes", v.to_string())))
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

        UserList::from_response(response).await
    }

    async fn create_user(&self, req: CreateUserRequest) -> Result<UserData, DracoonClientError> {
        let url_part = format!("/{DRACOON_API_PREFIX}/{USERS_BASE}");
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
            .json(&req)
            .send()
            .await?;

        UserData::from_response(response).await
    }

    async fn get_user(
        &self,
        user_id: u64,
        effective_roles: Option<bool>,
    ) -> Result<UserData, DracoonClientError> {
        let url_part = format!("/{DRACOON_API_PREFIX}/{USERS_BASE}/{user_id}");
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

        UserData::from_response(response).await
    }

    async fn update_user(
        &self,
        user_id: u64,
        req: UpdateUserRequest,
    ) -> Result<UserData, DracoonClientError> {
        let url_part = format!("/{DRACOON_API_PREFIX}/{USERS_BASE}/{user_id}");
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
            .json(&req)
            .send()
            .await?;

        UserData::from_response(response).await
    }

    async fn delete_user(&self, user_id: u64) -> Result<(), DracoonClientError> {
        let url_part = format!("/{DRACOON_API_PREFIX}/{USERS_BASE}/{user_id}");
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
                .expect("Could not parse error response"));
        }

        Ok(())
    }

    async fn get_user_last_admin_rooms(
        &self,
        user_id: u64,
    ) -> Result<LastAdminUserRoomList, DracoonClientError> {
        let url_part =
            format!("/{DRACOON_API_PREFIX}/{USERS_BASE}/{user_id}/{USERS_LAST_ADMIN_ROOMS}");
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

        LastAdminUserRoomList::from_response(response).await
    }
}
