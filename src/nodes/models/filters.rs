use crate::models::{FilterOperator, FilterQuery};

use super::NodeType;

#[derive(Debug, Clone)]
pub enum NodesFilter {
    Name(FilterOperator, String),
    Type(FilterOperator, NodeTypes),
    Encrypted(FilterOperator, bool),
    BranchVersion(FilterOperator, u64),
    TimestampCreation(FilterOperator, String),
    TimestampModification(FilterOperator, String),
    ReferenceId(FilterOperator, u64),
    // missing: perm, childPerm
    // TODO: add permission model enum in api/models.rs
}

#[derive(Debug, Clone)]
pub struct NodeTypes(Vec<NodeType>);

impl From<&NodeTypes> for String {
    fn from(node_types: &NodeTypes) -> Self {
        node_types
            .0
            .iter()
            .map(String::from)
            .collect::<Vec<_>>()
            .join(":")
    }
}

impl From<Vec<NodeType>> for NodeTypes {
    fn from(node_types: Vec<NodeType>) -> Self {
        NodeTypes(node_types)
    }
}

impl FilterQuery for NodesFilter {
    fn to_filter_string(&self) -> String {
        match self {
            NodesFilter::Name(op, value) => {
                let op = String::from(op);
                format!("name:{}:{}", op, value)
            }
            NodesFilter::Type(op, value) => {
                let op = String::from(op);
                let node_type = String::from(value);
                format!("type:{}:{}", op, node_type)
            }
            NodesFilter::Encrypted(op, value) => {
                let op = String::from(op);
                format!("encrypted:{}:{}", op, value)
            }
            NodesFilter::BranchVersion(op, value) => {
                let op = String::from(op);
                format!("branchVersion:{}:{}", op, value)
            }
            NodesFilter::TimestampCreation(op, value) => {
                let op = String::from(op);
                format!("timestampCreation:{}:{}", op, value)
            }
            NodesFilter::TimestampModification(op, value) => {
                let op = String::from(op);
                format!("timestampModification:{}:{}", op, value)
            }
            NodesFilter::ReferenceId(op, value) => {
                let op = String::from(op);
                format!("referenceId:{}:{}", op, value)
            }
        }
    }
}

impl NodesFilter {
    pub fn name_equals(val: impl Into<String>) -> Self {
        NodesFilter::Name(FilterOperator::Eq, val.into())
    }

    pub fn name_contains(val: impl Into<String>) -> Self {
        NodesFilter::Name(FilterOperator::Cn, val.into())
    }

    pub fn is_encrypted(val: bool) -> Self {
        NodesFilter::Encrypted(FilterOperator::Eq, val)
    }

    pub fn reference_id_equals(val: u64) -> Self {
        NodesFilter::ReferenceId(FilterOperator::Eq, val)
    }

    pub fn created_before(val: impl Into<String>) -> Self {
        NodesFilter::TimestampCreation(FilterOperator::Le, val.into())
    }

    pub fn created_after(val: impl Into<String>) -> Self {
        NodesFilter::TimestampCreation(FilterOperator::Ge, val.into())
    }

    pub fn modified_before(val: impl Into<String>) -> Self {
        NodesFilter::TimestampModification(FilterOperator::Le, val.into())
    }

    pub fn modified_after(val: impl Into<String>) -> Self {
        NodesFilter::TimestampModification(FilterOperator::Ge, val.into())
    }

    pub fn branch_version_before(val: u64) -> Self {
        NodesFilter::BranchVersion(FilterOperator::Le, val)
    }

    pub fn branch_version_after(val: u64) -> Self {
        NodesFilter::BranchVersion(FilterOperator::Ge, val)
    }

    pub fn is_file() -> Self {
        NodesFilter::Type(FilterOperator::Eq, vec![NodeType::File].into())
    }

    pub fn is_folder() -> Self {
        NodesFilter::Type(FilterOperator::Eq, vec![NodeType::Folder].into())
    }

    pub fn is_room() -> Self {
        NodesFilter::Type(FilterOperator::Eq, vec![NodeType::Room].into())
    }

    pub fn is_types(val: Vec<NodeType>) -> Self {
        let node_types = NodeTypes(val);
        NodesFilter::Type(FilterOperator::Eq, node_types)
    }
}

impl From<NodesFilter> for Box<dyn FilterQuery> {
    fn from(filter: NodesFilter) -> Self {
        Box::new(filter)
    }
}

#[derive(Debug, Clone)]
pub enum NodesSearchFilter {
    Type(FilterOperator, NodeTypes),
    FileType(FilterOperator, String),
    Classification(FilterOperator, u8),
    CreatedBy(FilterOperator, String),
    UpdatedBy(FilterOperator, String),
    CreatedById(FilterOperator, u64),
    UpdatedById(FilterOperator, u64),
    CreatedAt(FilterOperator, String),
    UpdatedAt(FilterOperator, String),
    ExpireAt(FilterOperator, String),
    Size(FilterOperator, u64),
    IsFavorite(FilterOperator, bool),
    BranchVersion(FilterOperator, u64),
    ParentPath(FilterOperator, String),
    TimestampCreation(FilterOperator, String),
    TimestampModification(FilterOperator, String),
    ReferenceId(FilterOperator, u64),
}

impl NodesSearchFilter {
    pub fn is_file() -> Self {
        NodesSearchFilter::Type(FilterOperator::Eq, vec![NodeType::File].into())
    }

    pub fn is_folder() -> Self {
        NodesSearchFilter::Type(FilterOperator::Eq, vec![NodeType::Folder].into())
    }

    pub fn is_room() -> Self {
        NodesSearchFilter::Type(FilterOperator::Eq, vec![NodeType::Room].into())
    }

    pub fn is_types(val: Vec<NodeType>) -> Self {
        let node_types = NodeTypes(val);
        NodesSearchFilter::Type(FilterOperator::Eq, node_types)
    }

    pub fn is_favorite(val: bool) -> Self {
        NodesSearchFilter::IsFavorite(FilterOperator::Eq, val)
    }

    pub fn parent_path_equals(val: impl Into<String>) -> Self {
        NodesSearchFilter::ParentPath(FilterOperator::Eq, val.into())
    }

    pub fn parent_path_contains(val: impl Into<String>) -> Self {
        NodesSearchFilter::ParentPath(FilterOperator::Cn, val.into())
    }

    pub fn size_greater_equals(val: u64) -> Self {
        NodesSearchFilter::Size(FilterOperator::Ge, val)
    }

    pub fn size_less_equals(val: u64) -> Self {
        NodesSearchFilter::Size(FilterOperator::Le, val)
    }

    pub fn branch_version_before(val: u64) -> Self {
        NodesSearchFilter::BranchVersion(FilterOperator::Le, val)
    }

    pub fn branch_version_after(val: u64) -> Self {
        NodesSearchFilter::BranchVersion(FilterOperator::Ge, val)
    }

    pub fn created_at_before(val: impl Into<String>) -> Self {
        NodesSearchFilter::CreatedAt(FilterOperator::Le, val.into())
    }

    pub fn created_at_after(val: impl Into<String>) -> Self {
        NodesSearchFilter::CreatedAt(FilterOperator::Ge, val.into())
    }

    pub fn updated_at_before(val: impl Into<String>) -> Self {
        NodesSearchFilter::UpdatedAt(FilterOperator::Le, val.into())
    }

    pub fn updated_at_after(val: impl Into<String>) -> Self {
        NodesSearchFilter::UpdatedAt(FilterOperator::Ge, val.into())
    }

    pub fn expire_at_before(val: impl Into<String>) -> Self {
        NodesSearchFilter::ExpireAt(FilterOperator::Le, val.into())
    }

    pub fn expire_at_after(val: impl Into<String>) -> Self {
        NodesSearchFilter::ExpireAt(FilterOperator::Ge, val.into())
    }

    pub fn classification_equals(val: u8) -> Self {
        NodesSearchFilter::Classification(FilterOperator::Eq, val)
    }

    pub fn file_type_equals(val: impl Into<String>) -> Self {
        NodesSearchFilter::FileType(FilterOperator::Eq, val.into())
    }

    pub fn file_type_contains(val: impl Into<String>) -> Self {
        NodesSearchFilter::FileType(FilterOperator::Cn, val.into())
    }
}

impl From<NodesSearchFilter> for Box<dyn FilterQuery> {
    fn from(filter: NodesSearchFilter) -> Self {
        Box::new(filter)
    }
}

impl FilterQuery for NodesSearchFilter {
    fn to_filter_string(&self) -> String {
        match self {
            NodesSearchFilter::BranchVersion(op, val) => {
                let op = String::from(op);
                format!("branchVersion:{}:{}", op, val)
            }
            NodesSearchFilter::Type(op, val) => {
                let op = String::from(op);
                let node_type: String = val.into();
                format!("type:{}:{}", op, node_type)
            }
            NodesSearchFilter::FileType(op, val) => {
                let op = String::from(op);
                format!("fileType:{}:{}", op, val)
            }
            NodesSearchFilter::Classification(op, val) => {
                let op = String::from(op);
                format!("classification:{}:{}", op, val)
            }
            NodesSearchFilter::CreatedBy(op, val) => {
                let op = String::from(op);
                format!("createdBy:{}:{}", op, val)
            }
            NodesSearchFilter::UpdatedBy(op, val) => {
                let op = String::from(op);
                format!("updatedBy:{}:{}", op, val)
            }
            NodesSearchFilter::CreatedById(op, val) => {
                let op = String::from(op);
                format!("createdById:{}:{}", op, val)
            }
            NodesSearchFilter::UpdatedById(op, val) => {
                let op = String::from(op);
                format!("updatedById:{}:{}", op, val)
            }
            NodesSearchFilter::CreatedAt(op, val) => {
                let op = String::from(op);
                format!("createdAt:{}:{}", op, val)
            }
            NodesSearchFilter::UpdatedAt(op, val) => {
                let op = String::from(op);
                format!("updatedAt:{}:{}", op, val)
            }
            NodesSearchFilter::ExpireAt(op, val) => {
                let op = String::from(op);
                format!("expireAt:{}:{}", op, val)
            }
            NodesSearchFilter::Size(op, val) => {
                let op = String::from(op);
                format!("size:{}:{}", op, val)
            }
            NodesSearchFilter::IsFavorite(op, val) => {
                let op = String::from(op);
                format!("isFavorite:{}:{}", op, val)
            }
            NodesSearchFilter::ParentPath(op, val) => {
                let op = String::from(op);
                format!("parentPath:{}:{}", op, val)
            }
            NodesSearchFilter::TimestampCreation(op, val) => {
                let op = String::from(op);
                format!("timestampCreation:{}:{}", op, val)
            }
            NodesSearchFilter::TimestampModification(op, val) => {
                let op = String::from(op);
                format!("timestampModification:{}:{}", op, val)
            }
            NodesSearchFilter::ReferenceId(op, val) => {
                let op = String::from(op);
                format!("referenceId:{}:{}", op, val)
            }
        }
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[test]
    fn test_nodes_filter_name_eq() {
        let filter = NodesFilter::name_equals("test");
        assert_eq!(filter.to_filter_string(), "name:eq:test");
    }

    #[test]
    fn test_nodes_filter_name_contains() {
        let filter = NodesFilter::name_contains("test");
        assert_eq!(filter.to_filter_string(), "name:cn:test");
    }

    #[test]
    fn test_nodes_filter_is_file() {
        let filter = NodesFilter::is_file();
        assert_eq!(filter.to_filter_string(), "type:eq:file");
    }

    #[test]
    fn test_nodes_filter_is_room() {
        let filter = NodesFilter::is_room();
        assert_eq!(filter.to_filter_string(), "type:eq:room");
    }

    #[test]
    fn test_nodes_filter_is_folder() {
        let filter = NodesFilter::is_folder();
        assert_eq!(filter.to_filter_string(), "type:eq:folder");
    }

    #[test]
    fn test_nodes_filter_is_folder_room() {
        let filter = NodesFilter::is_types(vec![NodeType::Folder, NodeType::Room]);
        assert_eq!(filter.to_filter_string(), "type:eq:folder:room");
    }

    #[test]
    fn test_nodes_filter_is_encrypted() {
        let filter = NodesFilter::is_encrypted(true);
        let neg_filter = NodesFilter::is_encrypted(false);
        assert_eq!(filter.to_filter_string(), "encrypted:eq:true");
        assert_eq!(neg_filter.to_filter_string(), "encrypted:eq:false");
    }

    #[test]
    fn test_nodes_filter_created_before() {
        let filter = NodesFilter::created_before("2020-01-01T00:00:00.000Z");
        assert_eq!(
            filter.to_filter_string(),
            "timestampCreation:le:2020-01-01T00:00:00.000Z"
        );
    }

    #[test]
    fn test_nodes_filter_created_after() {
        let filter = NodesFilter::created_after("2020-01-01T00:00:00.000Z");
        assert_eq!(
            filter.to_filter_string(),
            "timestampCreation:ge:2020-01-01T00:00:00.000Z"
        );
    }

    #[test]
    fn test_nodes_filter_modified_before() {
        let filter = NodesFilter::modified_before("2020-01-01T00:00:00.000Z");
        assert_eq!(
            filter.to_filter_string(),
            "timestampModification:le:2020-01-01T00:00:00.000Z"
        );
    }

    #[test]
    fn test_nodes_filter_modified_after() {
        let filter = NodesFilter::modified_after("2020-01-01T00:00:00.000Z");
        assert_eq!(
            filter.to_filter_string(),
            "timestampModification:ge:2020-01-01T00:00:00.000Z"
        );
    }

    #[test]
    fn test_nodes_filter_branch_version_before() {
        let filter = NodesFilter::branch_version_before(1);
        assert_eq!(filter.to_filter_string(), "branchVersion:le:1");
    }

    #[test]
    fn test_nodes_filter_branch_version_after() {
        let filter = NodesFilter::branch_version_after(1);
        assert_eq!(filter.to_filter_string(), "branchVersion:ge:1");
    }

    #[test]
    fn test_nodes_search_filter_is_folder() {
        let filter = NodesSearchFilter::is_folder();
        assert_eq!(filter.to_filter_string(), "type:eq:folder");
    }

    #[test]
    fn test_nodes_search_filter_is_file() {
        let filter = NodesSearchFilter::is_file();
        assert_eq!(filter.to_filter_string(), "type:eq:file");
    }

    #[test]
    fn test_nodes_search_filter_is_room() {
        let filter = NodesSearchFilter::is_room();
        assert_eq!(filter.to_filter_string(), "type:eq:room");
    }

    #[test]
    fn test_nodes_search_filter_is_favorite() {
        let filter = NodesSearchFilter::is_favorite(true);
        let neg_filter = NodesSearchFilter::is_favorite(false);
        assert_eq!(filter.to_filter_string(), "isFavorite:eq:true");
        assert_eq!(neg_filter.to_filter_string(), "isFavorite:eq:false");
    }

    #[test]
    fn test_nodes_search_filter_parent_path_equals() {
        let filter = NodesSearchFilter::parent_path_equals("test");
        assert_eq!(filter.to_filter_string(), "parentPath:eq:test");
    }

    #[test]
    fn test_nodes_search_filter_parent_path_contains() {
        let filter = NodesSearchFilter::parent_path_contains("test");
        assert_eq!(filter.to_filter_string(), "parentPath:cn:test");
    }

    #[test]
    fn test_nodes_search_filter_size_greater_equals() {
        let filter = NodesSearchFilter::size_greater_equals(1);
        assert_eq!(filter.to_filter_string(), "size:ge:1");
    }

    #[test]
    fn test_nodes_search_filter_size_less_equals() {
        let filter = NodesSearchFilter::size_less_equals(1);
        assert_eq!(filter.to_filter_string(), "size:le:1");
    }

    #[test]
    fn test_nodess_search_filter_branch_version_before() {
        let filter = NodesSearchFilter::branch_version_before(1);
        assert_eq!(filter.to_filter_string(), "branchVersion:le:1");
    }

    #[test]
    fn test_nodes_search_filter_branch_version_after() {
        let filter = NodesSearchFilter::branch_version_after(1);
        assert_eq!(filter.to_filter_string(), "branchVersion:ge:1");
    }

    #[test]
    fn test_nodes_search_filter_created_at_before() {
        let filter = NodesSearchFilter::created_at_before("2021-02-01T00:00:00.000Z");
        assert_eq!(
            filter.to_filter_string(),
            "createdAt:le:2021-02-01T00:00:00.000Z"
        );
    }

    #[test]
    fn test_nodes_search_filter_created_at_after() {
        let filter = NodesSearchFilter::created_at_after("2021-02-01T00:00:00.000Z");
        assert_eq!(
            filter.to_filter_string(),
            "createdAt:ge:2021-02-01T00:00:00.000Z"
        );
    }

    #[test]
    fn test_nodes_search_filter_updated_at_before() {
        let filter = NodesSearchFilter::updated_at_before("2021-02-01T00:00:00.000Z");
        assert_eq!(
            filter.to_filter_string(),
            "updatedAt:le:2021-02-01T00:00:00.000Z"
        );
    }

    #[test]
    fn test_nodes_search_filter_updated_at_after() {
        let filter = NodesSearchFilter::updated_at_after("2021-02-01T00:00:00.000Z");
        assert_eq!(
            filter.to_filter_string(),
            "updatedAt:ge:2021-02-01T00:00:00.000Z"
        );
    }

    #[test]
    fn test_nodes_search_filter_expire_at_before() {
        let filter = NodesSearchFilter::expire_at_before("2021-02-01T00:00:00.000Z");
        assert_eq!(
            filter.to_filter_string(),
            "expireAt:le:2021-02-01T00:00:00.000Z"
        );
    }

    #[test]
    fn test_nodes_search_filter_expire_at_after() {
        let filter = NodesSearchFilter::expire_at_after("2021-02-01T00:00:00.000Z");
        assert_eq!(
            filter.to_filter_string(),
            "expireAt:ge:2021-02-01T00:00:00.000Z"
        );
    }

    #[test]
    fn test_nodes_search_filter_classification_equals() {
        let filter = NodesSearchFilter::classification_equals(1);
        assert_eq!(filter.to_filter_string(), "classification:eq:1");
    }

    #[test]
    fn test_nodes_search_filter_file_type_equals() {
        let filter = NodesSearchFilter::file_type_equals("jpg");
        assert_eq!(filter.to_filter_string(), "fileType:eq:jpg");
    }

    #[test]
    fn test_nodes_search_filter_file_type_contains() {
        let filter = NodesSearchFilter::file_type_contains("jpg");
        assert_eq!(filter.to_filter_string(), "fileType:cn:jpg");
    }

    #[test]
    fn test_nodes_search_folder_room() {
        let filter = NodesSearchFilter::is_types(vec![NodeType::Folder, NodeType::Room]);
        assert_eq!(filter.to_filter_string(), "type:eq:folder:room");
    }
}
