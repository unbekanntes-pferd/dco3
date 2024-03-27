//! This module contains all constants, specifically to address API endpoints and default configuration values.

// constants for `grant_type` parameter
pub const GRANT_TYPE_PASSWORD: &str = "password";
pub const GRANT_TYPE_AUTH_CODE: &str = "authorization_code";
pub const GRANT_TYPE_REFRESH_TOKEN: &str = "refresh_token";
pub const TOKEN_TYPE_HINT_ACCESS: &str = "access_token";

// constants for API urls
// AUTH
pub const DRACOON_TOKEN_URL: &str = "oauth/token";
pub const DRACOON_REDIRECT_URL: &str = "oauth/callback";
pub const DRACOON_TOKEN_REVOKE_URL: &str = "oauth/revoke";
pub const TOKEN_TYPE_HINT_ACCESS_TOKEN: &str = "access_token";
pub const TOKEN_TYPE_HINT_REFRESH_TOKEN: &str = "refresh_token";
pub const MAX_TOKEN_COUNT: u8 = 5;
pub const MIN_TOKEN_COUNT: u8 = 1;

// API
pub const DRACOON_API_PREFIX: &str = "api/v4";

// NODES
pub const NODES_BASE: &str = "nodes";
pub const NODES_MOVE: &str = "move_to";
pub const NODES_COPY: &str = "copy_to";
pub const FILES_BASE: &str = "files";
pub const FILES_FILE_KEY: &str = "user_file_key";
pub const FILES_UPLOAD: &str = "uploads";
pub const FILES_S3_URLS: &str = "s3_urls";
pub const FILES_S3_COMPLETE: &str = "s3";
pub const FOLDERS_BASE: &str = "folders";
pub const NODES_DOWNLOAD_URL: &str = "downloads";
pub const NODES_SEARCH: &str = "search";
pub const MISSING_FILE_KEYS: &str = "missingFileKeys";
pub const FILES_KEYS: &str = "keys";
pub const ROOMS_BASE: &str = "rooms";
pub const ROOMS_CONFIG: &str = "config";
pub const ROOMS_ENCRYPT: &str = "encrypt";
pub const ROOMS_USERS: &str = "users";
pub const ROOMS_GROUPS: &str = "groups";
pub const ROOMS_POLICIES: &str = "policies";

// SHARES
pub const SHARES_BASE: &str = "shares";
pub const SHARES_EMAIL: &str = "email";

// SHARES - UPLOAD
pub const SHARES_UPLOAD: &str = "uploads";

// SHARES - DOWNLOAD
pub const SHARES_DOWNLOAD: &str = "downloads";

// DEFAULTS
pub const CHUNK_SIZE: usize = 1024 * 1024 * 32; // 32 MB
                                                // concurrent requests
pub const BATCH_SIZE: usize = 20;
pub const POLLING_START_DELAY: u64 = 300;
// defines how many keys (users) distributed per file on upload
pub const MISSING_KEYS_BATCH: usize = 50;

// USER
pub const USER_BASE: &str = "user";
pub const USER_ACCOUNT: &str = "account";
pub const USER_ACCOUNT_KEYPAIR: &str = "keypair";

// GROUPS
pub const GROUPS_BASE: &str = "groups";
pub const GROUPS_USERS: &str = "users";
pub const GROUPS_LAST_ADMIN_ROOMS: &str = "last_admin_rooms";

// USERS
pub const USERS_BASE: &str = "users";
pub const USERS_LAST_ADMIN_ROOMS: &str = "last_admin_rooms";

// PROVISIONING
pub const PROVISIONING_BASE: &str = "provisioning";
pub const PROVISIONING_CUSTOMERS: &str = "customers";
pub const PROVISIONING_CUSTOMER_ATTRIBUTES: &str = "customerAttributes";
pub const PROVISIONING_CUSTOMER_USERS: &str = "users";
pub const PROVISIONING_TOKEN_HEADER: &str = "X-Sds-Service-Token";

// SETTINGS
pub const SETTINGS_BASE: &str = "settings";
pub const SETTINGS_KEYPAIR: &str = "keypair";

// SYSTEM
pub const SYSTEM_BASE: &str = "system";
pub const SYSTEM_AUTH_BASE: &str = "auth";
pub const SYSTEM_AUTH_OPENID: &str = "openid";
pub const SYSTEM_AUTH_OPENID_IDPS: &str = "idps";
pub const SYSTEM_AUTH_ADS: &str = "ads";

// CONFIG
pub const CONFIG_BASE: &str = "config/info";
pub const CONFIG_GENERAL: &str = "general";
pub const CONFIG_DEFAULTS: &str = "defaults";
pub const CONFIG_INFRASTRUCTURE: &str = "infrastructure";
pub const CONFIG_ALGORITHMS: &str = "algorithms";
pub const CONFIG_POLICIES: &str = "policies";
pub const CONFIG_PASSWORD_POLICIES: &str = "passwords";
pub const CONFIG_PRODUCT_PACKAGES: &str = "product_packages";
pub const CONFIG_S3_TAGS: &str = "s3_tags";
pub const CONFIG_PRODUCT_PACKAGES_CURRENT: &str = "current";
pub const CONFIG_CLASSIFICATION_POLICIES: &str = "classifications";

/// user agent header
pub const APP_USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "|", env!("CARGO_PKG_VERSION"));

// retry config
pub const MAX_RETRIES: u32 = 5;
pub const MIN_RETRY_DELAY: u64 = 600; // in milliseconds (0.6 seconds)
pub const MAX_RETRY_DELAY: u64 = 20 * 1000; // in milliseconds (20 seconds)
