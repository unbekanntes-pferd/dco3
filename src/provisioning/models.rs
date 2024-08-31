use std::sync::Arc;

use async_trait::async_trait;
use chrono::{DateTime, Utc};
use dco3_derive::FromResponse;
use reqwest::Response;
use serde::{Deserialize, Serialize};

use crate::{
    client::{DracoonClient, DracoonErrorResponse},
    user::UserAuthData,
    utils::{parse_body, FromResponse},
    DracoonClientError, KeyValueEntry, RangedItems,
};

#[derive(Clone)]
pub struct ProvisioningEndpoint<S> {
    client: Arc<DracoonClient<S>>,
    state: std::marker::PhantomData<S>,
}

impl<S> ProvisioningEndpoint<S> {
    pub fn new(client: Arc<DracoonClient<S>>) -> Self {
        Self {
            client,
            state: std::marker::PhantomData,
        }
    }

    pub fn client(&self) -> &Arc<DracoonClient<S>> {
        &self.client
    }
}

#[derive(Debug, Serialize, Deserialize, Default, Clone)]
pub struct CustomerAttributes {
    pub items: Vec<KeyValueEntry>,
}

impl CustomerAttributes {
    pub fn new() -> CustomerAttributes {
        CustomerAttributes::default()
    }

    pub fn add_attribute(&mut self, key: impl Into<String>, value: impl Into<String>) {
        let attrib = KeyValueEntry {
            key: key.into(),
            value: value.into(),
        };
        self.items.push(attrib);
    }
}

pub type AttributesResponse = RangedItems<KeyValueEntry>;

#[derive(Debug, Deserialize, FromResponse, Clone)]
#[serde(rename_all = "camelCase")]
pub struct Customer {
    pub id: u64,
    pub company_name: String,
    pub customer_contract_type: String,
    pub quota_max: u64,
    pub quota_used: u64,
    pub user_max: u64,
    pub user_used: u64,
    pub created_at: DateTime<Utc>,
    pub customer_attributes: Option<CustomerAttributes>,
    pub updated_at: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub trial_days_left: Option<i32>,
    pub is_locked: Option<bool>,
    pub customer_uuid: Option<String>,
    pub cnt_internal_user: Option<u64>,
    pub cnt_guest_user: Option<u64>,
}

pub type CustomerList = RangedItems<Customer>;

#[async_trait]
impl FromResponse for CustomerList {
    async fn from_response(response: Response) -> Result<Self, DracoonClientError> {
        parse_body::<Self, DracoonErrorResponse>(response).await
    }
}

#[async_trait]
impl FromResponse for AttributesResponse {
    async fn from_response(response: Response) -> Result<Self, DracoonClientError> {
        parse_body::<Self, DracoonErrorResponse>(response).await
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct FirstAdminUser {
    pub first_name: String,
    pub last_name: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub user_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_data: Option<UserAuthData>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub receiver_language: Option<String>,
    pub notify_user: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub phone: Option<String>,
}

impl FirstAdminUser {
    pub fn new_local(
        first_name: impl Into<String>,
        last_name: impl Into<String>,
        user_name: Option<String>,
        email: impl Into<String>,
        receiver_language: Option<String>,
    ) -> FirstAdminUser {
        let auth_data = UserAuthData::new_basic(None, None);

        FirstAdminUser {
            first_name: first_name.into(),
            last_name: last_name.into(),
            user_name,
            auth_data: Some(auth_data),
            receiver_language,
            notify_user: None,
            email: Some(email.into()),
            phone: None,
        }
    }
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewCustomerRequest {
    pub customer_contract_type: String,
    pub quota_max: u64,
    pub user_max: u64,
    pub first_admin_user: FirstAdminUser,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub company_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trial_days: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub is_locked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub customer_attributes: Option<CustomerAttributes>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub provider_customer_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub webhooks_max: Option<u64>,
}

impl NewCustomerRequest {
    pub fn builder(
        customer_contract_type: impl Into<String>,
        quota_max: u64,
        user_max: u64,
        first_admin_user: FirstAdminUser,
    ) -> NewCustomerRequestBuilder {
        NewCustomerRequestBuilder::new(
            customer_contract_type.into(),
            quota_max,
            user_max,
            first_admin_user,
        )
    }
}

pub struct NewCustomerRequestBuilder {
    customer_contract_type: String,
    quota_max: u64,
    user_max: u64,
    first_admin_user: FirstAdminUser,
    company_name: Option<String>,
    trial_days: Option<u64>,
    is_locked: Option<bool>,
    customer_attributes: Option<CustomerAttributes>,
    provider_customer_id: Option<String>,
    webhooks_max: Option<u64>,
}

impl NewCustomerRequestBuilder {
    pub fn new(
        customer_contract_type: String,
        quota_max: u64,
        user_max: u64,
        first_admin_user: FirstAdminUser,
    ) -> NewCustomerRequestBuilder {
        NewCustomerRequestBuilder {
            customer_contract_type,
            quota_max,
            user_max,
            first_admin_user,
            company_name: None,
            trial_days: None,
            is_locked: None,
            customer_attributes: None,
            provider_customer_id: None,
            webhooks_max: None,
        }
    }

    pub fn with_company_name(
        mut self,
        company_name: impl Into<String>,
    ) -> NewCustomerRequestBuilder {
        self.company_name = Some(company_name.into());
        self
    }

    pub fn with_trial_days(mut self, trial_days: u64) -> NewCustomerRequestBuilder {
        self.trial_days = Some(trial_days);
        self
    }

    pub fn with_is_locked(mut self, is_locked: bool) -> NewCustomerRequestBuilder {
        self.is_locked = Some(is_locked);
        self
    }

    pub fn with_customer_attributes(
        mut self,
        customer_attributes: CustomerAttributes,
    ) -> NewCustomerRequestBuilder {
        self.customer_attributes = Some(customer_attributes);
        self
    }

    pub fn with_provider_customer_id(
        mut self,
        provider_customer_id: String,
    ) -> NewCustomerRequestBuilder {
        self.provider_customer_id = Some(provider_customer_id);
        self
    }

    pub fn with_webhooks_max(mut self, webhooks_max: u64) -> NewCustomerRequestBuilder {
        self.webhooks_max = Some(webhooks_max);
        self
    }

    pub fn build(self) -> NewCustomerRequest {
        NewCustomerRequest {
            customer_contract_type: self.customer_contract_type,
            quota_max: self.quota_max,
            user_max: self.user_max,
            first_admin_user: self.first_admin_user,
            company_name: self.company_name,
            trial_days: self.trial_days,
            is_locked: self.is_locked,
            customer_attributes: self.customer_attributes,
            provider_customer_id: self.provider_customer_id,
            webhooks_max: self.webhooks_max,
        }
    }
}

#[derive(Debug, Deserialize, FromResponse, Clone)]
#[serde(rename_all = "camelCase")]
pub struct NewCustomerResponse {
    pub id: u64,
    pub company_name: String,
    pub customer_contract_type: String,
    pub quota_max: u64,
    pub user_max: u64,
    pub is_locked: Option<bool>,
    pub trial_days: Option<u64>,
    pub created_at: Option<DateTime<Utc>>,
    pub first_admin_user: FirstAdminUser,
    pub customer_attributes: Option<CustomerAttributes>,
    pub provider_customer_id: Option<String>,
    pub webhooks_max: Option<u64>,
}

#[derive(Debug, Deserialize, FromResponse, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCustomerResponse {
    pub id: u64,
    pub company_name: String,
    pub customer_contract_type: String,
    pub quota_max: u64,
    pub user_max: u64,
    pub customer_uuid: String,
    pub is_locked: Option<bool>,
    pub trial_days: Option<u64>,
    pub created_at: Option<DateTime<Utc>>,
    pub updated_at: Option<DateTime<Utc>>,
    pub customer_attributes: Option<CustomerAttributes>,
    pub provider_customer_id: Option<String>,
    pub webhooks_max: Option<u64>,
}

#[derive(Debug, Serialize, Clone)]
#[serde(rename_all = "camelCase")]
pub struct UpdateCustomerRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    company_name: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    customer_contract_type: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    quota_max: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    user_max: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    is_locked: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    provider_customer_id: Option<u64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    webhooks_max: Option<u64>,
}

impl UpdateCustomerRequest {
    pub fn builder() -> UpdateCustomerRequestBuilder {
        UpdateCustomerRequestBuilder::new()
    }
}

#[derive(Debug, Default)]
pub struct UpdateCustomerRequestBuilder {
    company_name: Option<String>,
    customer_contract_type: Option<String>,
    quota_max: Option<u64>,
    user_max: Option<u64>,
    is_locked: Option<bool>,
    provider_customer_id: Option<u64>,
    webhooks_max: Option<u64>,
}

impl UpdateCustomerRequestBuilder {
    pub fn new() -> Self {
        UpdateCustomerRequestBuilder::default()
    }

    pub fn with_company_name(mut self, company_name: impl Into<String>) -> Self {
        self.company_name = Some(company_name.into());
        self
    }

    pub fn with_customer_contract_type(
        mut self,
        customer_contract_type: impl Into<String>,
    ) -> Self {
        self.customer_contract_type = Some(customer_contract_type.into());
        self
    }

    pub fn with_quota_max(mut self, quota_max: u64) -> Self {
        self.quota_max = Some(quota_max);
        self
    }

    pub fn with_user_max(mut self, user_max: u64) -> Self {
        self.user_max = Some(user_max);
        self
    }

    pub fn with_is_locked(mut self, is_locked: bool) -> Self {
        self.is_locked = Some(is_locked);
        self
    }

    pub fn with_provider_customer_id(mut self, provider_customer_id: u64) -> Self {
        self.provider_customer_id = Some(provider_customer_id);
        self
    }

    pub fn with_webhooks_max(mut self, webhooks_max: u64) -> Self {
        self.webhooks_max = Some(webhooks_max);
        self
    }

    pub fn build(self) -> UpdateCustomerRequest {
        UpdateCustomerRequest {
            company_name: self.company_name,
            customer_contract_type: self.customer_contract_type,
            quota_max: self.quota_max,
            user_max: self.user_max,
            is_locked: self.is_locked,
            provider_customer_id: self.provider_customer_id,
            webhooks_max: self.webhooks_max,
        }
    }
}
