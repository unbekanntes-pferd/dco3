use async_trait::async_trait;
pub use models::{
    EventStatus, EventlogEndpoint, EventlogParams, EventlogSortBy, LogEventList, LogOperationList,
};
use reqwest::header;

use crate::utils::FromResponse;
use crate::{auth::Connected, DracoonClientError};

mod models;

use crate::constants::{DRACOON_API_PREFIX, EVENTLOG_BASE, EVENTLOG_EVENTS, EVENTLOG_OPERATIONS};

#[async_trait]
pub trait Eventlog {
    /// Get a list of events from eventlog.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Eventlog, eventlog::{EventlogParams, EventlogSortBy, EventStatus}, SortOrder};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    /// // Params are optional
    /// let params = EventlogParams::builder()
    ///    .with_limit(10)
    ///    .with_offset(0)
    ///    .with_sort(EventlogSortBy::Time(SortOrder::Desc))
    ///    .with_user_id(1)
    ///    .with_operation_type(1)
    ///    .with_status(EventStatus::Success)
    ///    .with_date_end(chrono::Utc::now())
    ///    .build();
    /// // pass EventlogParams::default() if you don't want to use any params
    /// let event_list = dracoon.eventlog.get_events(params).await.unwrap();
    ///
    /// # }
    /// ```
    async fn get_events(&self, params: EventlogParams) -> Result<LogEventList, DracoonClientError>;

    /// Get a list of event operations.
    /// ```no_run
    /// # use dco3::{Dracoon, auth::OAuth2Flow, Eventlog};
    /// # #[tokio::main]
    /// # async fn main() {
    /// # let dracoon = Dracoon::builder()
    /// #  .with_base_url("https://dracoon.team")
    /// #  .with_client_id("client_id")
    /// #  .with_client_secret("client_secret")
    /// #  .build()
    /// #  .unwrap()
    /// #  .connect(OAuth2Flow::PasswordFlow("username".into(), "password".into()))
    /// #  .await
    /// #  .unwrap();
    /// let operations = dracoon.eventlog.get_event_operations().await.unwrap();
    /// # }
    /// ```
    async fn get_event_operations(&self) -> Result<LogOperationList, DracoonClientError>;
}

#[async_trait]
impl Eventlog for EventlogEndpoint<Connected> {
    async fn get_events(&self, params: EventlogParams) -> Result<LogEventList, DracoonClientError> {
        let url_part = format!("{DRACOON_API_PREFIX}/{EVENTLOG_BASE}/{EVENTLOG_EVENTS}");
        let mut api_url = self.client().build_api_url(&url_part);

        if !params.is_empty() {
            api_url
                .query_pairs_mut()
                .extend_pairs(params.limit.map(|v| ("limit", v.to_string())))
                .extend_pairs(params.offset.map(|v| ("offset", v.to_string())))
                .extend_pairs(params.sort.map(|v| ("sort", v.to_sort_string())))
                .extend_pairs(params.user_id.map(|v| ("user_id", v.to_string())))
                .extend_pairs(params.operation_type.map(|v| ("type", v.to_string())))
                .extend_pairs(params.status.map(|v| ("status", (v as i64).to_string())))
                .extend_pairs(params.date_start.map(|v| {
                    (
                        "date_start",
                        v.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                    )
                }))
                .extend_pairs(params.date_end.map(|v| {
                    (
                        "date_end",
                        v.to_rfc3339_opts(chrono::SecondsFormat::Secs, true),
                    )
                }))
                .finish();
        };

        let response = self
            .client()
            .http
            .get(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        LogEventList::from_response(response).await
    }

    async fn get_event_operations(&self) -> Result<LogOperationList, DracoonClientError> {
        let url_part =
            format!("{DRACOON_API_PREFIX}/{EVENTLOG_BASE}/{EVENTLOG_EVENTS}/{EVENTLOG_OPERATIONS}");
        let api_url = self.client().build_api_url(&url_part);

        let response = self
            .client()
            .http
            .get(api_url)
            .header(
                header::AUTHORIZATION,
                self.client().get_auth_header().await?,
            )
            .header(header::CONTENT_TYPE, "application/json")
            .send()
            .await?;

        LogOperationList::from_response(response).await
    }
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use mockito::Matcher;

    use crate::{
        eventlog::{
            models::{EventStatus, EventlogSortBy},
            Eventlog, EventlogParams,
        },
        tests::dracoon::get_connected_client,
        SortOrder,
    };

    #[tokio::test]
    async fn test_get_events() {
        let (client, mut mock_server) = get_connected_client().await;

        let response = include_str!("../tests/responses/eventlog/events_ok.json");

        let events_mock = mock_server
            .mock("GET", "/api/v4/eventlog/events")
            .with_status(200)
            .with_body(response)
            .with_header("content-type", "application/json")
            .expect(1)
            .create_async()
            .await;

        let events = client.eventlog.get_events(EventlogParams::default()).await;

        events_mock.assert();

        assert!(events.is_ok());

        let events = events.unwrap();

        assert_eq!(events.items.len(), 1);
    }

    #[tokio::test]
    async fn test_get_events_with_offset() {
        let (client, mut mock_server) = get_connected_client().await;

        let response = include_str!("../tests/responses/eventlog/events_ok.json");

        let events_mock = mock_server
            .mock("GET", "/api/v4/eventlog/events")
            .match_query(Matcher::UrlEncoded("offset".into(), "1".into()))
            .with_status(200)
            .with_body(response)
            .with_header("content-type", "application/json")
            .expect(1)
            .create_async()
            .await;

        let params = EventlogParams::builder().with_offset(1).build();

        let events = client.eventlog.get_events(params).await;

        events_mock.assert();

        assert!(events.is_ok());

        let events = events.unwrap();

        assert_eq!(events.items.len(), 1);
    }

    #[tokio::test]
    async fn test_get_events_with_limit() {
        let (client, mut mock_server) = get_connected_client().await;

        let response = include_str!("../tests/responses/eventlog/events_ok.json");

        let events_mock = mock_server
            .mock("GET", "/api/v4/eventlog/events")
            .match_query(Matcher::UrlEncoded("limit".into(), "1".into()))
            .with_status(200)
            .with_body(response)
            .with_header("content-type", "application/json")
            .expect(1)
            .create_async()
            .await;

        let params = EventlogParams::builder().with_limit(1).build();

        let events = client.eventlog.get_events(params).await;

        events_mock.assert();

        assert!(events.is_ok());

        let events = events.unwrap();

        assert_eq!(events.items.len(), 1);
    }

    #[tokio::test]
    async fn test_get_events_with_sort() {
        let (client, mut mock_server) = get_connected_client().await;

        let response = include_str!("../tests/responses/eventlog/events_ok.json");

        let events_mock = mock_server
            .mock("GET", "/api/v4/eventlog/events")
            .match_query(Matcher::UrlEncoded("sort".into(), "time:desc".into()))
            .with_status(200)
            .with_body(response)
            .with_header("content-type", "application/json")
            .expect(1)
            .create_async()
            .await;

        let params = EventlogParams::builder()
            .with_sort(EventlogSortBy::Time(SortOrder::Desc))
            .build();

        let events = client.eventlog.get_events(params).await;

        events_mock.assert();

        assert!(events.is_ok());

        let events = events.unwrap();

        assert_eq!(events.items.len(), 1);
    }

    #[tokio::test]
    async fn test_get_events_with_user_id() {
        let (client, mut mock_server) = get_connected_client().await;

        let response = include_str!("../tests/responses/eventlog/events_ok.json");

        let events_mock = mock_server
            .mock("GET", "/api/v4/eventlog/events")
            .match_query(Matcher::UrlEncoded("user_id".into(), "1".into()))
            .with_status(200)
            .with_body(response)
            .with_header("content-type", "application/json")
            .expect(1)
            .create_async()
            .await;

        let params = EventlogParams::builder().with_user_id(1).build();

        let events = client.eventlog.get_events(params).await;

        events_mock.assert();

        assert!(events.is_ok());

        let events = events.unwrap();

        assert_eq!(events.items.len(), 1);
    }

    #[tokio::test]
    async fn test_get_events_with_operation_type() {
        let (client, mut mock_server) = get_connected_client().await;

        let response = include_str!("../tests/responses/eventlog/events_ok.json");

        let events_mock = mock_server
            .mock("GET", "/api/v4/eventlog/events")
            .match_query(Matcher::UrlEncoded("type".into(), "1".into()))
            .with_status(200)
            .with_body(response)
            .with_header("content-type", "application/json")
            .expect(1)
            .create_async()
            .await;

        let params = EventlogParams::builder().with_operation_type(1).build();

        let events = client.eventlog.get_events(params).await;

        events_mock.assert();

        assert!(events.is_ok());

        let events = events.unwrap();

        assert_eq!(events.items.len(), 1);
    }

    #[tokio::test]
    async fn test_get_events_with_status_success() {
        let (client, mut mock_server) = get_connected_client().await;

        let response = include_str!("../tests/responses/eventlog/events_ok.json");

        let events_mock = mock_server
            .mock("GET", "/api/v4/eventlog/events")
            .match_query(Matcher::UrlEncoded("status".into(), "0".into()))
            .with_status(200)
            .with_body(response)
            .with_header("content-type", "application/json")
            .expect(1)
            .create_async()
            .await;

        let params = EventlogParams::builder()
            .with_status(EventStatus::Success)
            .build();

        let events = client.eventlog.get_events(params).await;

        events_mock.assert();

        assert!(events.is_ok());

        let events = events.unwrap();

        assert_eq!(events.items.len(), 1);
    }

    #[tokio::test]
    async fn test_get_events_with_date_start() {
        let (client, mut mock_server) = get_connected_client().await;

        let response = include_str!("../tests/responses/eventlog/events_ok.json");

        let events_mock = mock_server
            .mock("GET", "/api/v4/eventlog/events")
            .match_query(Matcher::UrlEncoded(
                "date_start".into(),
                "2021-01-01T00:00:00Z".into(),
            ))
            .with_status(200)
            .with_body(response)
            .with_header("content-type", "application/json")
            .expect(1)
            .create_async()
            .await;

        let date = DateTime::parse_from_rfc3339("2021-01-01T00:00:00Z")
            .unwrap()
            .to_utc();
        let params = EventlogParams::builder().with_date_start(date).build();

        let events = client.eventlog.get_events(params).await;

        events_mock.assert();

        assert!(events.is_ok());

        let events = events.unwrap();

        assert_eq!(events.items.len(), 1);
    }

    #[tokio::test]
    async fn test_get_events_with_date_end() {
        let (client, mut mock_server) = get_connected_client().await;

        let response = include_str!("../tests/responses/eventlog/events_ok.json");

        let events_mock = mock_server
            .mock("GET", "/api/v4/eventlog/events")
            .match_query(Matcher::UrlEncoded(
                "date_end".into(),
                "2021-01-01T00:00:00Z".into(),
            ))
            .with_status(200)
            .with_body(response)
            .with_header("content-type", "application/json")
            .expect(1)
            .create_async()
            .await;

        let date = DateTime::parse_from_rfc3339("2021-01-01T00:00:00Z")
            .unwrap()
            .to_utc();

        let params = EventlogParams::builder().with_date_end(date).build();

        let events = client.eventlog.get_events(params).await;

        events_mock.assert();

        assert!(events.is_ok());

        let events = events.unwrap();

        assert_eq!(events.items.len(), 1);
    }

    #[tokio::test]
    async fn test_get_event_operations() {
        let (client, mut mock_server) = get_connected_client().await;

        let response = include_str!("../tests/responses/eventlog/event_operations_ok.json");

        let operations_mock = mock_server
            .mock("GET", "/api/v4/eventlog/events/operations")
            .with_status(200)
            .with_body(response)
            .with_header("content-type", "application/json")
            .expect(1)
            .create_async()
            .await;

        let operations = client.eventlog.get_event_operations().await.unwrap();
        assert_eq!(operations.operation_list.len(), 1);

        operations_mock.assert();

        let operation = operations.operation_list.first().unwrap();

        assert_eq!(operation.id, 1);
        assert_eq!(operation.name, "string");
        assert!(!operation.is_deprecated);
    }
}
