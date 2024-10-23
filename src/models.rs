//! This module implments basic models for the DRACOON API.
use std::fmt::Debug;

use chrono::{DateTime, Utc};
use dco3_crypto::PlainUserKeyPairContainer;
use secrecy::{zeroize::Zeroize, CloneableSecret};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::{
    client::DracoonClient, config::ConfigEndpoint, eventlog::EventlogEndpoint,
    groups::GroupsEndpoint, nodes::NodesEndpoint, provisioning::ProvisioningEndpoint,
    public::PublicEndpoint, roles::RolesEndpoint, settings::SettingsEndpoint,
    shares::SharesEndpoint, system::SystemEndpoint, user::UserEndpoint, users::UsersEndpoint,
};

use super::client::errors::DracoonClientError;

pub trait ConnectedClient {}

// struct for internal mutability
#[derive(Debug, Clone, Default)]
pub struct Container<T: Clone> {
    data: Arc<RwLock<Option<T>>>,
}

impl<T: Clone> Container<T> {
    pub fn new_from(data: T) -> Self {
        Self {
            data: Arc::new(RwLock::new(Some(data))),
        }
    }

    pub fn new() -> Self {
        Self {
            data: Arc::new(RwLock::new(None)),
        }
    }

    pub async fn set(&self, data: T) {
        let mut lock = self.data.write().await;
        *lock = Some(data);
    }

    pub async fn get(&self) -> Option<T> {
        let lock = self.data.read().await;
        lock.clone()
    }

    pub async fn is_some(&self) -> bool {
        let lock = self.data.read().await;
        lock.is_some()
    }

    pub async fn is_none(&self) -> bool {
        let lock = self.data.read().await;
        lock.is_none()
    }
}

#[derive(Debug)]
pub struct ListAllParams {
    pub offset: Option<u64>,
    pub limit: Option<u64>,
    pub filter: Option<FilterQueries>,
    pub sort: Option<SortQueries>,
}

impl Default for ListAllParams {
    fn default() -> Self {
        Self {
            offset: Some(0),
            limit: None,
            filter: None,
            sort: None,
        }
    }
}

/// This is a struct to pass params for listing requests (GET)
/// - offset: offset of the list
/// - limit: limit of the list
/// - filter: filter queries
/// - sort: sort queries
///
/// To build filters, use appropiate filters in relevant modules
/// To build sorts, use appropiate sorts in relevant modules
///
/// Example:
///
/// ´´´
/// use dco3::models::ListAllParams;
///
/// let params = ListAllParams::builder()
///    .with_offset(0)
///   .with_limit(100)
///   .with_filter(dco3::nodes::models::NodesFilter::name_equals("test"))
///   .with_sort(dco3::nodes::models::NodesSort::name_asc())
///  .build();
///
/// ´´´
///   
impl ListAllParams {
    /// Creates a builder to pass filters, sorting, offset and limit
    pub fn builder() -> ListAllParamsBuilder {
        ListAllParamsBuilder::new()
    }

    pub fn filter_to_string(&self) -> String {
        match self.filter.as_deref() {
            Some(filters) => filters
                .iter()
                .map(|filter| filter.to_filter_string())
                .collect::<Vec<String>>()
                .join("|"),
            None => String::new(),
        }
    }

    pub fn sort_to_string(&self) -> String {
        match self.sort.as_deref() {
            Some(sorts) => sorts
                .iter()
                .map(|sort| sort.to_sort_string())
                .collect::<Vec<String>>()
                .join("|"),
            None => String::new(),
        }
    }

    pub fn is_empty(&self) -> bool {
        self.offset.is_none()
            && self.limit.is_none()
            && self.filter.is_none()
            && self.sort.is_none()
    }
}

pub struct ListAllParamsBuilder {
    params: ListAllParams,
}

impl ListAllParamsBuilder {
    pub fn new() -> Self {
        Self {
            params: ListAllParams::default(),
        }
    }
    pub fn with_offset(mut self, offset: u64) -> Self {
        self.params.offset = Some(offset);
        self
    }

    pub fn with_limit(mut self, limit: u64) -> Self {
        self.params.limit = Some(limit);
        self
    }

    pub fn with_filter<F>(mut self, filter: F) -> Self
    where
        F: Into<Box<dyn FilterQuery>>,
    {
        match self.params.filter {
            Some(mut filters) => {
                filters.push(filter.into());
                self.params.filter = Some(filters);
            }
            None => {
                let filters = vec![filter.into()];
                self.params.filter = Some(filters);
            }
        }
        self
    }

    pub fn with_sort<S>(mut self, sort: S) -> Self
    where
        S: Into<Box<dyn SortQuery>>,
    {
        match self.params.sort {
            Some(mut sorts) => {
                sorts.push(sort.into());
                self.params.sort = Some(sorts);
            }
            None => {
                let sorts = vec![sort.into()];
                self.params.sort = Some(sorts);
            }
        }

        self
    }

    pub fn build(self) -> ListAllParams {
        self.params
    }
}

impl Default for ListAllParamsBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl From<ListAllParams> for String {
    fn from(value: ListAllParams) -> Self {
        let params = format!("?offset={}", value.offset.unwrap_or(0));

        let filters = value.filter_to_string();
        let sorts = value.sort_to_string();

        let params = value
            .filter
            .map(|_| format!("{params}&filter={}", filters))
            .unwrap_or(params);
        let params = value
            .sort
            .map(|_| format!("{params}&sort={}", sorts))
            .unwrap_or(params);

        value
            .limit
            .map(|limit| format!("{params}&limit={limit}"))
            .unwrap_or(params)
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Range {
    pub offset: u64,
    pub limit: u64,
    pub total: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, Default)]
#[serde(rename_all = "camelCase")]
pub struct ObjectExpiration {
    pub enable_expiration: bool,
    pub expire_at: Option<String>,
}

impl ObjectExpiration {
    pub fn new(expire_at: DateTime<Utc>) -> Self {
        Self {
            enable_expiration: true,
            expire_at: Some(expire_at.to_rfc3339()),
        }
    }
}

impl AsRef<ObjectExpiration> for ObjectExpiration {
    fn as_ref(&self) -> &Self {
        self
    }
}

impl From<DateTime<Utc>> for ObjectExpiration {
    fn from(value: DateTime<Utc>) -> Self {
        Self::new(value)
    }
}

#[derive(Deserialize, Debug, Clone)]
pub struct RangedItems<T> {
    pub range: Range,
    pub items: Vec<T>,
}

impl<T> IntoIterator for RangedItems<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.items.into_iter()
    }
}

pub trait FilterQuery: Debug + Send + Sync {
    fn to_filter_string(&self) -> String;

    fn builder() -> FilterQueryBuilder
    where
        Self: Sized,
    {
        FilterQueryBuilder::new()
    }
}

pub trait SortQuery: Debug + Send + Sync {
    fn to_sort_string(&self) -> String;

    fn builder() -> SortQueryBuilder
    where
        Self: Sized,
    {
        SortQueryBuilder::new()
    }
}

pub type FilterQueries = Vec<Box<dyn FilterQuery>>;
pub type SortQueries = Vec<Box<dyn SortQuery>>;

#[derive(Debug, Clone)]
pub enum FilterOperator {
    Eq,
    Cn,
    Neq,
    Ge,
    Le,
}

impl From<FilterOperator> for String {
    fn from(value: FilterOperator) -> Self {
        match value {
            FilterOperator::Eq => "eq".to_string(),
            FilterOperator::Cn => "cn".to_string(),
            FilterOperator::Neq => "neq".to_string(),
            FilterOperator::Ge => "ge".to_string(),
            FilterOperator::Le => "le".to_string(),
        }
    }
}

impl From<&FilterOperator> for String {
    fn from(value: &FilterOperator) -> Self {
        match value {
            FilterOperator::Eq => "eq".to_string(),
            FilterOperator::Cn => "cn".to_string(),
            FilterOperator::Neq => "neq".to_string(),
            FilterOperator::Ge => "ge".to_string(),
            FilterOperator::Le => "le".to_string(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum SortOrder {
    Asc,
    Desc,
}

impl From<SortOrder> for String {
    fn from(value: SortOrder) -> Self {
        match value {
            SortOrder::Asc => "asc".to_string(),
            SortOrder::Desc => "desc".to_string(),
        }
    }
}

impl From<&SortOrder> for String {
    fn from(value: &SortOrder) -> Self {
        match value {
            SortOrder::Asc => "asc".to_string(),
            SortOrder::Desc => "desc".to_string(),
        }
    }
}

#[derive(Default)]
pub struct FilterQueryBuilder {
    field: Option<String>,
    operator: Option<FilterOperator>,
    value: Option<String>,
}

impl FilterQueryBuilder {
    pub fn new() -> Self {
        Self {
            field: None,
            operator: None,
            value: None,
        }
    }

    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    pub fn with_operator(mut self, operator: FilterOperator) -> Self {
        self.operator = Some(operator);
        self
    }

    pub fn with_value(mut self, value: impl Into<String>) -> Self {
        self.value = Some(value.into());
        self
    }

    pub fn try_build(self) -> Result<String, DracoonClientError> {
        let field = self.field.ok_or(DracoonClientError::MissingArgument)?;
        let operator = self.operator.ok_or(DracoonClientError::MissingArgument)?;
        let operator: String = operator.into();
        let value = self.value.ok_or(DracoonClientError::MissingArgument)?;

        Ok(format!("{}:{}:{}", field, operator, value))
    }
}

#[derive(Default)]
pub struct SortQueryBuilder {
    field: Option<String>,
    order: Option<SortOrder>,
}

impl SortQueryBuilder {
    pub fn new() -> Self {
        Self {
            field: None,
            order: None,
        }
    }

    pub fn with_field(mut self, field: impl Into<String>) -> Self {
        self.field = Some(field.into());
        self
    }

    pub fn with_order(mut self, order: SortOrder) -> Self {
        self.order = Some(order);
        self
    }

    pub fn try_build(self) -> Result<String, DracoonClientError> {
        let field = self.field.ok_or(DracoonClientError::MissingArgument)?;
        let order = self.order.ok_or(DracoonClientError::MissingArgument)?;
        let order: String = order.into();

        Ok(format!("{}:{}", field, order))
    }
}

impl FilterQuery for String {
    fn to_filter_string(&self) -> String {
        self.clone()
    }
}

impl SortQuery for String {
    fn to_sort_string(&self) -> String {
        self.clone()
    }
}

impl From<String> for Box<dyn FilterQuery> {
    fn from(value: String) -> Self {
        Box::new(value)
    }
}

impl From<String> for Box<dyn SortQuery> {
    fn from(value: String) -> Self {
        Box::new(value)
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct KeyValueEntry {
    pub key: String,
    pub value: String,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_filter_query_builder() {
        let query = FilterQueryBuilder::new()
            .with_field("field")
            .with_operator(FilterOperator::Eq)
            .with_value("value")
            .try_build()
            .unwrap();

        assert_eq!(query, "field:eq:value");

        let params = ListAllParams::builder()
            .with_filter(query.to_filter_string())
            .build();

        assert_eq!(params.filter_to_string(), "field:eq:value");
    }

    #[test]
    fn test_sort_query_builder() {
        let query = SortQueryBuilder::new()
            .with_field("field")
            .with_order(SortOrder::Asc)
            .try_build()
            .unwrap();

        assert_eq!(query, "field:asc");

        let params = ListAllParams::builder()
            .with_sort(query.to_sort_string())
            .build();

        assert_eq!(params.sort_to_string(), "field:asc");
    }

    #[test]
    fn test_filter_query_builder_missing_field() {
        let query = FilterQueryBuilder::new()
            .with_operator(FilterOperator::Eq)
            .with_value("value")
            .try_build();

        assert!(query.is_err());
    }

    #[test]
    fn test_filter_query_builder_missing_operator() {
        let query = FilterQueryBuilder::new()
            .with_field("field")
            .with_value("value")
            .try_build();

        assert!(query.is_err());
    }

    #[test]
    fn test_filter_query_builder_missing_value() {
        let query = FilterQueryBuilder::new()
            .with_field("field")
            .with_operator(FilterOperator::Eq)
            .try_build();

        assert!(query.is_err());
    }

    #[test]
    fn test_sort_query_builder_missing_field() {
        let query = SortQueryBuilder::new()
            .with_order(SortOrder::Asc)
            .try_build();

        assert!(query.is_err());
    }

    #[test]
    fn test_sort_query_builder_missing_order() {
        let query = SortQueryBuilder::new().with_field("field").try_build();

        assert!(query.is_err());
    }
}

#[derive(Clone)]
pub struct Endpoints<S> {
    pub user: UserEndpoint<S>,
    pub public: PublicEndpoint<S>,
    pub shares: SharesEndpoint<S>,
    pub users: UsersEndpoint<S>,
    pub groups: GroupsEndpoint<S>,
    pub settings: SettingsEndpoint<S>,
    pub provisioning: ProvisioningEndpoint<S>,
    pub config: ConfigEndpoint<S>,
    pub system: SystemEndpoint<S>,
    pub nodes: NodesEndpoint<S>,
    pub eventlog: EventlogEndpoint<S>,
    pub roles: RolesEndpoint<S>,
}

impl<S> From<&Arc<DracoonClient<S>>> for Endpoints<S> {
    fn from(client: &Arc<DracoonClient<S>>) -> Self {
        Self {
            user: UserEndpoint::new(client.clone()),
            public: PublicEndpoint::new(client.clone()),
            shares: SharesEndpoint::new(client.clone()),
            users: UsersEndpoint::new(client.clone()),
            groups: GroupsEndpoint::new(client.clone()),
            settings: SettingsEndpoint::new(client.clone()),
            provisioning: ProvisioningEndpoint::new(client.clone()),
            config: ConfigEndpoint::new(client.clone()),
            system: SystemEndpoint::new(client.clone()),
            nodes: NodesEndpoint::new(client.clone()),
            eventlog: EventlogEndpoint::new(client.clone()),
            roles: RolesEndpoint::new(client.clone()),
        }
    }
}

#[derive(Clone)]
pub struct WrappedUserKeypair(PlainUserKeyPairContainer);

impl CloneableSecret for WrappedUserKeypair {}

impl Zeroize for WrappedUserKeypair {
    fn zeroize(&mut self) {
        self.0.private_key_container.private_key.zeroize();
    }
}

impl WrappedUserKeypair {
    pub fn new(keypair: PlainUserKeyPairContainer) -> Self {
        Self(keypair)
    }

    pub fn keypair(&self) -> &PlainUserKeyPairContainer {
        &self.0
    }
}
