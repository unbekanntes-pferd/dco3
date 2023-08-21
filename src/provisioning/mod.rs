use async_trait::async_trait;

mod models;

pub use self::models::*;

use crate::{
    auth::Provisioning,
    constants::{
        DRACOON_API_PREFIX, PROVISIONING_BASE, PROVISIONING_CUSTOMERS,
        PROVISIONING_CUSTOMER_ATTRIBUTES, PROVISIONING_CUSTOMER_USERS, PROVISIONING_TOKEN_HEADER,
    },
    users::UserList,
    utils::FromResponse,
    Dracoon, DracoonClientError, ListAllParams,
};

#[async_trait]
pub trait CustomerProvisioning {
    async fn get_customers(
        &self,
        params: Option<ListAllParams>,
    ) -> Result<CustomerList, DracoonClientError>;
    async fn create_customer(
        &self,
        req: NewCustomerRequest,
    ) -> Result<NewCustomerResponse, DracoonClientError>;
    async fn get_customer(
        &self,
        id: u64,
        include_attributes: Option<bool>,
    ) -> Result<Customer, DracoonClientError>;
    async fn update_customer(
        &self,
        id: u64,
        req: UpdateCustomerRequest,
    ) -> Result<UpdateCustomerResponse, DracoonClientError>;
    async fn delete_customer(&self, id: u64) -> Result<(), DracoonClientError>;
    async fn get_customer_users(&self, id: u64, params: Option<ListAllParams>) -> Result<UserList, DracoonClientError>;
    async fn get_customer_attributes(
        &self,
        id: u64,
        params: Option<ListAllParams>,
    ) -> Result<AttributesResponse, DracoonClientError>;
    async fn update_customer_attributes(
        &self,
        id: u64,
        req: CustomerAttributes,
    ) -> Result<Customer, DracoonClientError>;
    async fn delete_customer_attribute(
        &self,
        id: u64,
        key: String,
    ) -> Result<(), DracoonClientError>;
}

#[async_trait]
impl CustomerProvisioning for Dracoon<Provisioning> {
    async fn get_customers(
        &self,
        params: Option<ListAllParams>,
    ) -> Result<CustomerList, DracoonClientError> {
        let params = params.unwrap_or_default();
        let url_part = format!("{DRACOON_API_PREFIX}/{PROVISIONING_BASE}/{PROVISIONING_CUSTOMERS}");

        let mut api_url = self.build_api_url(&url_part);

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
            .client
            .http
            .get(api_url)
            .header(PROVISIONING_TOKEN_HEADER, self.get_service_token())
            .send()
            .await?;

        CustomerList::from_response(response).await
    }
    async fn create_customer(
        &self,
        req: NewCustomerRequest,
    ) -> Result<NewCustomerResponse, DracoonClientError> {
        let url_part = format!("{DRACOON_API_PREFIX}/{PROVISIONING_BASE}/{PROVISIONING_CUSTOMERS}");
        let api_url = self.build_api_url(&url_part);

        let response = self
            .client
            .http
            .post(api_url)
            .header(PROVISIONING_TOKEN_HEADER, self.get_service_token())
            .json(&req)
            .send()
            .await?;

        NewCustomerResponse::from_response(response).await
    }
    async fn get_customer(
        &self,
        id: u64,
        include_attributes: Option<bool>,
    ) -> Result<Customer, DracoonClientError> {
        let url_part =
            format!("{DRACOON_API_PREFIX}/{PROVISIONING_BASE}/{PROVISIONING_CUSTOMERS}/{id}");

        let mut api_url = self.build_api_url(&url_part);

        if include_attributes.is_some() {
            api_url
                .query_pairs_mut()
                .extend_pairs(include_attributes.map(|v| ("include_attributes", v.to_string())))
                .finish();
        }

        let response = self
            .client
            .http
            .get(api_url)
            .header(PROVISIONING_TOKEN_HEADER, self.get_service_token())
            .send()
            .await?;

        Customer::from_response(response).await
    }
    async fn update_customer(
        &self,
        id: u64,
        req: UpdateCustomerRequest,
    ) -> Result<UpdateCustomerResponse, DracoonClientError> {
        let url_part =
            format!("{DRACOON_API_PREFIX}/{PROVISIONING_BASE}/{PROVISIONING_CUSTOMERS}/{id}");
        let api_url = self.build_api_url(&url_part);

        let response = self
            .client
            .http
            .put(api_url)
            .header(PROVISIONING_TOKEN_HEADER, self.get_service_token())
            .json(&req)
            .send()
            .await?;

        UpdateCustomerResponse::from_response(response).await
    }

    async fn delete_customer(&self, id: u64) -> Result<(), DracoonClientError> {
        let url_part =
            format!("{DRACOON_API_PREFIX}/{PROVISIONING_BASE}/{PROVISIONING_CUSTOMERS}/{id}");

        let api_url = self.build_api_url(&url_part);

        let response = self
            .client
            .http
            .delete(api_url)
            .header(PROVISIONING_TOKEN_HEADER, self.get_service_token())
            .send()
            .await?;

        if response.status().is_server_error() || response.status().is_client_error() {
            return Err(DracoonClientError::from_response(response)
                .await
                .expect("Could not parse error response"));
        }

        Ok(())
    }
    async fn get_customer_users(&self, id: u64, params: Option<ListAllParams>) -> Result<UserList, DracoonClientError> {
        let params = params.unwrap_or_default();
        let url_part = format!("{DRACOON_API_PREFIX}/{PROVISIONING_BASE}/{PROVISIONING_CUSTOMERS}/{id}/{PROVISIONING_CUSTOMER_USERS}");

        let mut api_url = self.build_api_url(&url_part);

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
            .client
            .http
            .get(api_url)
            .header(PROVISIONING_TOKEN_HEADER, self.get_service_token())
            .send()
            .await?;

        UserList::from_response(response).await
    }
    async fn get_customer_attributes(
        &self,
        id: u64,
        params: Option<ListAllParams>,
    ) -> Result<AttributesResponse, DracoonClientError> {
        let params = params.unwrap_or_default();
        let url_part = format!("{DRACOON_API_PREFIX}/{PROVISIONING_BASE}/{PROVISIONING_CUSTOMERS}/{id}/{PROVISIONING_CUSTOMER_ATTRIBUTES}");

        let mut api_url = self.build_api_url(&url_part);

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
            .client
            .http
            .get(api_url)
            .header(PROVISIONING_TOKEN_HEADER, self.get_service_token())
            .send()
            .await?;

        AttributesResponse::from_response(response).await
    }
    async fn update_customer_attributes(
        &self,
        id: u64,
        req: CustomerAttributes,
    ) -> Result<Customer, DracoonClientError> {
        let url_part = format!("{DRACOON_API_PREFIX}/{PROVISIONING_BASE}/{PROVISIONING_CUSTOMERS}/{id}/{PROVISIONING_CUSTOMER_ATTRIBUTES}");

        let api_url = self.build_api_url(&url_part);

        let response = self
            .client
            .http
            .put(api_url)
            .header(PROVISIONING_TOKEN_HEADER, self.get_service_token())
            .json(&req)
            .send()
            .await?;

        Customer::from_response(response).await
    }
    async fn delete_customer_attribute(
        &self,
        id: u64,
        key: String,
    ) -> Result<(), DracoonClientError> {
        let url_part = format!("{DRACOON_API_PREFIX}/{PROVISIONING_BASE}/{PROVISIONING_CUSTOMERS}/{id}/{PROVISIONING_CUSTOMER_ATTRIBUTES}/{key}");

        let api_url = self.build_api_url(&url_part);

        let response = self
            .client
            .http
            .delete(api_url)
            .header(PROVISIONING_TOKEN_HEADER, self.get_service_token())
            .send()
            .await?;

        if response.status().is_server_error() || response.status().is_client_error() {
            return Err(DracoonClientError::from_response(response)
                .await
                .expect("Could not parse error response"));
        }

        Ok(())
    }
}
