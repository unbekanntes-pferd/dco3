#[cfg(test)]
mod tests {
    use schemars::schema_for;
    use serde_json::Value;

    fn schema_json<T: schemars::JsonSchema>() -> Value {
        serde_json::to_value(schema_for!(T)).unwrap()
    }

    #[test]
    fn test_create_user_request_schema_uses_serde_field_names() {
        let schema = schema_json::<crate::users::CreateUserRequest>();
        let properties = schema["properties"].as_object().unwrap();
        let required = schema["required"].as_array().unwrap();

        assert!(properties.contains_key("firstName"));
        assert!(properties.contains_key("receiverLanguage"));
        assert!(properties.contains_key("mfaConfig"));
        assert!(required.contains(&Value::String("firstName".to_string())));
        assert!(required.contains(&Value::String("lastName".to_string())));
        assert!(!required.contains(&Value::String("receiverLanguage".to_string())));
    }

    #[test]
    fn test_group_schema_contains_nested_defs_and_datetime() {
        let schema = schema_json::<crate::groups::Group>();
        let defs = schema["$defs"].as_object().unwrap();

        assert!(defs.contains_key("UserInfo"));
        assert!(defs.contains_key("RoleList"));
        assert_eq!(schema["properties"]["createdAt"]["type"], "string");
        assert_eq!(schema["properties"]["createdAt"]["format"], "date-time");
    }

    #[test]
    fn test_group_list_schema_contains_range_and_items() {
        let schema = schema_json::<crate::groups::GroupList>();
        let properties = schema["properties"].as_object().unwrap();

        assert!(properties.contains_key("range"));
        assert!(properties.contains_key("items"));
        assert!(schema["$defs"].as_object().unwrap().contains_key("Group"));
    }

    #[test]
    fn test_http_error_schema_is_available() {
        let schema = schema_json::<crate::client::models::DracoonErrorResponse>();

        assert_eq!(schema["properties"]["code"]["type"], "integer");
        assert!(schema["properties"]
            .as_object()
            .unwrap()
            .contains_key("debugInfo"));
    }

    #[test]
    fn test_public_download_share_schema_contains_crypto_defs() {
        let schema = schema_json::<crate::public::PublicDownloadShare>();
        let defs = schema["$defs"].as_object().unwrap();

        assert!(defs.contains_key("FileKey"));
        assert!(defs.contains_key("PrivateKeyContainer"));
    }

    #[test]
    fn test_create_download_share_request_schema_contains_keypair() {
        let schema = schema_json::<crate::shares::CreateDownloadShareRequest>();
        let properties = schema["properties"].as_object().unwrap();
        let defs = schema["$defs"].as_object().unwrap();

        assert!(properties.contains_key("keypair"));
        assert!(defs.contains_key("UserKeyPairContainer"));
    }
}
