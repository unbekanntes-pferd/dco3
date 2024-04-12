#[cfg(test)]
mod tests {
    use chrono::DateTime;

    use crate::{
        auth::Provisioning,
        provisioning::{Customer, FirstAdminUser, NewCustomerRequest, UpdateCustomerRequest},
        tests::users::tests::assert_user_item,
        CustomerProvisioning, Dracoon, ListAllParams,
    };

    async fn get_provisioning_client() -> (Dracoon<Provisioning>, mockito::ServerGuard) {
        let mock_server = mockito::Server::new_async().await;
        let base_url = mock_server.url();

        let dracoon = Dracoon::builder()
            .with_base_url(base_url)
            .with_provisioning_token("token")
            .build_provisioning()
            .unwrap();

        (dracoon, mock_server)
    }

    async fn assert_customer(customer: &Customer) {
        assert_eq!(customer.id, 1);
        assert_eq!(customer.company_name, "string");
        assert_eq!(customer.customer_contract_type, "pay");
        assert_eq!(customer.quota_max, 10000000);
        assert_eq!(customer.quota_used, 10);
        assert_eq!(customer.user_max, 100);
        assert_eq!(customer.user_used, 100);
        assert_eq!(customer.cnt_guest_user.unwrap(), 1);
        assert_eq!(customer.cnt_internal_user.unwrap(), 99);
        assert_eq!(
            customer.created_at,
            DateTime::parse_from_rfc3339("2020-01-01T00:00:00.000Z").unwrap()
        );
        assert_eq!(
            customer.updated_at.as_ref().unwrap(),
            &DateTime::parse_from_rfc3339("2020-01-01T00:00:00.000Z").unwrap()
        );
        assert_eq!(customer.trial_days_left.unwrap(), 0);
        assert_eq!(customer.customer_uuid.as_ref().unwrap(), "string");
        assert!(customer.customer_attributes.is_some());
        assert!(!customer
            .customer_attributes
            .as_ref()
            .unwrap()
            .items
            .is_empty());
        assert!(customer
            .customer_attributes
            .as_ref()
            .unwrap()
            .items
            .first()
            .is_some());
        let kv = customer
            .customer_attributes
            .as_ref()
            .unwrap()
            .items
            .first()
            .unwrap();
        assert_eq!(kv.key, "string");
        assert_eq!(kv.value, "string");
        assert!(!customer.is_locked.unwrap());
    }

    #[tokio::test]
    async fn test_get_customers() {
        let (dracoon, mut mock_server) = get_provisioning_client().await;
        let customers_res = include_str!("./responses/provisioning/customers_ok.json");

        let customers_mock = mock_server
            .mock("GET", "/api/v4/provisioning/customers?offset=0")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(customers_res)
            .create();

        let customers = dracoon.provisioning.get_customers(None).await.unwrap();
        assert_eq!(customers.range.total, 1);
        assert_eq!(customers.range.offset, 0);
        assert_eq!(customers.range.limit, 0);
        assert_eq!(customers.items.len(), 1);

        let customer = customers.items.first().unwrap();
        assert_customer(customer).await;

        customers_mock.assert();
    }

    #[tokio::test]
    async fn test_get_customers_with_limit() {
        let (dracoon, mut mock_server) = get_provisioning_client().await;
        let customers_res = include_str!("./responses/provisioning/customers_ok.json");

        let customers_mock = mock_server
            .mock("GET", "/api/v4/provisioning/customers?limit=100&offset=0")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(customers_res)
            .create();

        let params = ListAllParams::builder().with_limit(100).build();

        let customers = dracoon
            .provisioning
            .get_customers(Some(params))
            .await
            .unwrap();
        assert_eq!(customers.range.total, 1);
        assert_eq!(customers.range.offset, 0);
        assert_eq!(customers.range.limit, 0);
        assert_eq!(customers.items.len(), 1);

        let customer = customers.items.first().unwrap();
        assert_customer(customer).await;

        customers_mock.assert();
    }

    #[tokio::test]
    async fn test_get_customers_with_offset() {
        let (dracoon, mut mock_server) = get_provisioning_client().await;
        let customers_res = include_str!("./responses/provisioning/customers_ok.json");

        let customers_mock = mock_server
            .mock("GET", "/api/v4/provisioning/customers?offset=500")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(customers_res)
            .create();

        let params = ListAllParams::builder().with_offset(500).build();

        let customers = dracoon
            .provisioning
            .get_customers(Some(params))
            .await
            .unwrap();
        assert_eq!(customers.range.total, 1);
        assert_eq!(customers.range.offset, 0);
        assert_eq!(customers.range.limit, 0);
        assert_eq!(customers.items.len(), 1);

        let customer = customers.items.first().unwrap();
        assert_customer(customer).await;

        customers_mock.assert();
    }

    #[tokio::test]
    #[ignore = "missing models for filter query"]
    async fn test_get_customers_with_filter() {
        let (dracoon, mut mock_server) = get_provisioning_client().await;
        let customers_res = include_str!("./responses/provisioning/customers_ok.json");

        let customers_mock = mock_server
            // TODO: add filter query
            .mock("GET", "/api/v4/provisioning/customers?offset=0")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(customers_res)
            .create();

        // TODO: add filter query
        let params = ListAllParams::builder().build();

        let customers = dracoon
            .provisioning
            .get_customers(Some(params))
            .await
            .unwrap();
        assert_eq!(customers.range.total, 1);
        assert_eq!(customers.range.offset, 0);
        assert_eq!(customers.range.limit, 0);
        assert_eq!(customers.items.len(), 1);

        let customer = customers.items.first().unwrap();
        assert_customer(customer).await;

        customers_mock.assert();
    }

    #[tokio::test]
    #[ignore = "missing models for sort query"]
    async fn test_get_customers_with_sort() {
        let (dracoon, mut mock_server) = get_provisioning_client().await;
        let customers_res = include_str!("./responses/provisioning/customers_ok.json");

        let customers_mock = mock_server
            // TODO: add sort query
            .mock("GET", "/api/v4/provisioning/customers?offset=0")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(customers_res)
            .create();

        // TODO: add sort query
        let customers = dracoon.provisioning.get_customers(None).await.unwrap();
        assert_eq!(customers.range.total, 1);
        assert_eq!(customers.range.offset, 0);
        assert_eq!(customers.range.limit, 0);
        assert_eq!(customers.items.len(), 1);

        let customer = customers.items.first().unwrap();
        assert_customer(customer).await;

        customers_mock.assert();
    }

    #[tokio::test]
    async fn test_create_customer() {
        let (dracoon, mut mock_server) = get_provisioning_client().await;

        let res = include_str!("./responses/provisioning/new_customer_ok.json");

        let customer_mock = mock_server
            .mock("POST", "/api/v4/provisioning/customers")
            .with_status(201)
            .with_header("content-type", "application/json")
            .with_body(res)
            .create();

        let user = FirstAdminUser::new_local("test", "test", None, "test@localhost", None);

        let customer = NewCustomerRequest::builder("pay", 100000000, 100, user)
            .with_company_name("test")
            .build();

        let customer = dracoon
            .provisioning
            .create_customer(customer)
            .await
            .unwrap();

        assert_eq!(customer.id, 1);
        assert_eq!(customer.company_name, "string");
        assert_eq!(customer.quota_max, 10000000);
        assert_eq!(customer.user_max, 100);
        assert_eq!(customer.customer_contract_type, "pay");
        assert_eq!(customer.first_admin_user.first_name, "string");
        assert_eq!(customer.first_admin_user.last_name, "string");
        assert_eq!(
            customer.first_admin_user.user_name.as_ref().unwrap(),
            "string"
        );
        assert_eq!(customer.first_admin_user.email.as_ref().unwrap(), "string");
        assert_eq!(
            customer.first_admin_user.auth_data.as_ref().unwrap().method,
            "basic"
        );
        assert!(customer.first_admin_user.notify_user.as_ref().unwrap());
        assert_eq!(customer.first_admin_user.phone.as_ref().unwrap(), "string");
        assert_eq!(customer.trial_days.as_ref().unwrap(), &0);
        assert_eq!(customer.provider_customer_id.as_ref().unwrap(), "string");
        assert_eq!(customer.webhooks_max.as_ref().unwrap(), &1);

        assert!(customer.customer_attributes.is_some());
        assert_eq!(
            customer.customer_attributes.as_ref().unwrap().items.len(),
            1
        );
        let kv = customer
            .customer_attributes
            .as_ref()
            .unwrap()
            .items
            .first()
            .unwrap();

        assert_eq!(kv.key, "string");
        assert_eq!(kv.value, "string");

        customer_mock.assert();
    }

    #[tokio::test]
    async fn test_get_customer() {
        let (dracoon, mut mock_server) = get_provisioning_client().await;
        let res = include_str!("./responses/provisioning/customer_ok.json");

        let customer_mock = mock_server
            .mock("GET", "/api/v4/provisioning/customers/1")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(res)
            .create();

        let customer = dracoon.provisioning.get_customer(1, None).await.unwrap();

        assert_customer(&customer).await;

        customer_mock.assert();
    }

    #[tokio::test]
    async fn test_get_customer_including_attributes() {
        let (dracoon, mut mock_server) = get_provisioning_client().await;
        let res = include_str!("./responses/provisioning/customer_ok.json");

        let customer_mock = mock_server
            .mock(
                "GET",
                "/api/v4/provisioning/customers/1?include_attributes=true",
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(res)
            .create();

        let customer = dracoon
            .provisioning
            .get_customer(1, Some(true))
            .await
            .unwrap();

        assert_customer(&customer).await;

        customer_mock.assert();
    }

    #[tokio::test]
    async fn test_update_customer() {
        let (dracoon, mut mock_server) = get_provisioning_client().await;
        let res = include_str!("./responses/provisioning/update_customer_ok.json");

        let customer_mock = mock_server
            .mock("PUT", "/api/v4/provisioning/customers/1")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(res)
            .create();

        let customer = UpdateCustomerRequest::builder()
            .with_quota_max(1000000000)
            .build();

        let customer = dracoon
            .provisioning
            .update_customer(1, customer)
            .await
            .unwrap();

        assert_eq!(customer.id, 1);
        assert_eq!(customer.company_name, "string");
        assert_eq!(customer.quota_max, 10000000);
        assert_eq!(customer.user_max, 100);
        assert_eq!(customer.customer_contract_type, "pay");
        assert_eq!(customer.trial_days.as_ref().unwrap(), &0);
        assert_eq!(customer.provider_customer_id.as_ref().unwrap(), "string");
        assert_eq!(customer.webhooks_max.as_ref().unwrap(), &1);
        assert_eq!(customer.customer_uuid, "string");

        customer_mock.assert();
    }

    #[tokio::test]
    async fn test_delete_customer() {
        let (dracoon, mut mock_server) = get_provisioning_client().await;

        let del_mock = mock_server
            .mock("DELETE", "/api/v4/provisioning/customers/1")
            .with_status(204)
            .create();

        let res = dracoon.provisioning.delete_customer(1).await;
        assert!(res.is_ok());
    }

    #[tokio::test]
    async fn test_get_customer_users() {
        let (dracoon, mut mock_server) = get_provisioning_client().await;
        let res = include_str!("./responses/users/users_ok.json");

        let customer_mock = mock_server
            .mock("GET", "/api/v4/provisioning/customers/1/users?offset=0")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(res)
            .create();

        let users = dracoon
            .provisioning
            .get_customer_users(1, None)
            .await
            .unwrap();

        assert_eq!(users.items.len(), 1);

        let user = users.items.first().unwrap();

        assert_user_item(user);
    }

    #[tokio::test]
    async fn test_get_customer_users_with_limit() {
        let (dracoon, mut mock_server) = get_provisioning_client().await;
        let res = include_str!("./responses/users/users_ok.json");

        let customer_mock = mock_server
            .mock(
                "GET",
                "/api/v4/provisioning/customers/1/users?limit=100&offset=0",
            )
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(res)
            .create();

        let params = ListAllParams::builder().with_limit(100).build();

        let users = dracoon
            .provisioning
            .get_customer_users(1, Some(params))
            .await
            .unwrap();

        assert_eq!(users.items.len(), 1);

        let user = users.items.first().unwrap();

        assert_user_item(user);
    }

    #[tokio::test]
    async fn test_get_customer_users_with_offset() {
        let (dracoon, mut mock_server) = get_provisioning_client().await;
        let res = include_str!("./responses/users/users_ok.json");

        let customer_mock = mock_server
            .mock("GET", "/api/v4/provisioning/customers/1/users?offset=500")
            .with_status(200)
            .with_header("content-type", "application/json")
            .with_body(res)
            .create();

        let params = ListAllParams::builder().with_offset(500).build();

        let users = dracoon
            .provisioning
            .get_customer_users(1, Some(params))
            .await
            .unwrap();

        assert_eq!(users.items.len(), 1);

        let user = users.items.first().unwrap();

        assert_user_item(user);
    }
}
