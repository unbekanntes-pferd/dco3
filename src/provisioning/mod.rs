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
/// This trait contains all methods for customer provisioning.
/// To use this trait, you need to create a client in `Provisioning` state.
///
/// ```no_run
///
/// use dco3::{Dracoon, OAuth2Flow, CustomerProvisioning};
///
/// #[tokio::main]
/// async fn main() {
/// // the client only requires passing the base url and a provisioning token
/// // other API calls are *not* supported in this state.
/// let dracoon = Dracoon::builder()
///    .with_base_url("https://dracoon.team")
///    .with_provisioning_token("some_token")
///    .build_provisioning()
///    .unwrap();
///
/// // now you can use the client in provisioning state
/// let customers = dracoon.get_customers(None).await.unwrap();
///
/// }
pub trait CustomerProvisioning {
    /// Returns a list of customers
    /// ```no_run
    /// # use dco3::{Dracoon, OAuth2Flow, CustomerProvisioning};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #   .with_base_url("https://dracoon.team")
    /// #   .with_provisioning_token("some_token")
    /// #   .build_provisioning()
    /// #   .unwrap();
    /// let customers = dracoon.get_customers(None).await.unwrap();
    /// # }
    async fn get_customers(
        &self,
        params: Option<ListAllParams>,
    ) -> Result<CustomerList, DracoonClientError>;
    /// Creates a new customer
    /// ```no_run
    /// # use dco3::{Dracoon, OAuth2Flow, CustomerProvisioning, provisioning::{FirstAdminUser, NewCustomerRequest}};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #   .with_base_url("https://dracoon.team")
    /// #   .with_provisioning_token("some_token")
    /// #   .build_provisioning()
    /// #   .unwrap();
    /// let first_admin = FirstAdminUser::new_local("admin", "admin", None, "admin@localhost", None);
    /// let customer = NewCustomerRequest::builder("pay", 100000, 100, first_admin).build();
    /// let customer = dracoon.create_customer(customer).await.unwrap();
    /// # }
    async fn create_customer(
        &self,
        req: NewCustomerRequest,
    ) -> Result<NewCustomerResponse, DracoonClientError>;
    /// Gets a customer by id
    /// ```no_run
    /// # use dco3::{Dracoon, OAuth2Flow, CustomerProvisioning, provisioning::{FirstAdminUser, NewCustomerRequest}};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #   .with_base_url("https://dracoon.team")
    /// #   .with_provisioning_token("some_token")
    /// #   .build_provisioning()
    /// #   .unwrap();
    /// let customer = dracoon.get_customer(123, None).await.unwrap();
    ///
    /// // include attributes
    /// let customer = dracoon.get_customer(123, Some(true)).await.unwrap();
    /// # }
    async fn get_customer(
        &self,
        id: u64,
        include_attributes: Option<bool>,
    ) -> Result<Customer, DracoonClientError>;
    /// Updates a customer by id
    /// ```no_run
    /// # use dco3::{Dracoon, OAuth2Flow, CustomerProvisioning, provisioning::UpdateCustomerRequest};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #   .with_base_url("https://dracoon.team")
    /// #   .with_provisioning_token("some_token")
    /// #   .build_provisioning()
    /// #   .unwrap();
    ///
    /// let update = UpdateCustomerRequest::builder()
    ///    .with_company_name("Foo Inc.")
    ///    .build();

    /// let customer = dracoon.update_customer(123, update).await.unwrap();
    ///
    /// # }
    async fn update_customer(
        &self,
        id: u64,
        req: UpdateCustomerRequest,
    ) -> Result<UpdateCustomerResponse, DracoonClientError>;
    /// Deletes a customer by id
    /// ```no_run
    /// # use dco3::{Dracoon, OAuth2Flow, CustomerProvisioning};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #   .with_base_url("https://dracoon.team")
    /// #   .with_provisioning_token("some_token")
    /// #   .build_provisioning()
    /// #   .unwrap();
    ///
    /// dracoon.delete_customer(123).await.unwrap();
    ///
    /// # }
    async fn delete_customer(&self, id: u64) -> Result<(), DracoonClientError>;
    /// Returns a list of customer users
    /// ```no_run
    /// # use dco3::{Dracoon, OAuth2Flow, CustomerProvisioning};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #   .with_base_url("https://dracoon.team")
    /// #   .with_provisioning_token("some_token")
    /// #   .build_provisioning()
    /// #   .unwrap();
    /// let users = dracoon.get_customer_users(123, None).await.unwrap();
    /// # }
    async fn get_customer_users(
        &self,
        id: u64,
        params: Option<ListAllParams>,
    ) -> Result<UserList, DracoonClientError>;
    /// Returns a list of customer attributes
    /// ```no_run
    /// # use dco3::{Dracoon, OAuth2Flow, CustomerProvisioning};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #   .with_base_url("https://dracoon.team")
    /// #   .with_provisioning_token("some_token")
    /// #   .build_provisioning()
    /// #   .unwrap();
    /// let attributes = dracoon.get_customer_attributes(123, None).await.unwrap();
    /// # }
    async fn get_customer_attributes(
        &self,
        id: u64,
        params: Option<ListAllParams>,
    ) -> Result<AttributesResponse, DracoonClientError>;
    /// Updates / sets customer attributes
    /// ```no_run
    /// # use dco3::{Dracoon, OAuth2Flow, CustomerProvisioning, provisioning::CustomerAttributes};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #   .with_base_url("https://dracoon.team")
    /// #   .with_provisioning_token("some_token")
    /// #   .build_provisioning()
    /// #   .unwrap();
    /// let mut attributes = CustomerAttributes::new();
    /// attributes.add_attribute("foo", "bar");
    /// let customer = dracoon.update_customer_attributes(123, attributes).await.unwrap();
    /// # }
    async fn update_customer_attributes(
        &self,
        id: u64,
        req: CustomerAttributes,
    ) -> Result<Customer, DracoonClientError>;
    /// Deletes customer attribute by key
    /// ```no_run
    /// # use dco3::{Dracoon, OAuth2Flow, CustomerProvisioning};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #   .with_base_url("https://dracoon.team")
    /// #   .with_provisioning_token("some_token")
    /// #   .build_provisioning()
    /// #   .unwrap();
    /// dracoon.delete_customer_attribute(123, "foo".to_string()).await.unwrap();
    /// # }
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
    async fn get_customer_users(
        &self,
        id: u64,
        params: Option<ListAllParams>,
    ) -> Result<UserList, DracoonClientError> {
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
