use crate::models::{SortOrder, SortQuery};

#[derive(Debug, Clone)]
pub enum NodesSortBy {
    Name(SortOrder),
    CreatedAt(SortOrder),
    CreatedBy(SortOrder),
    UpdatedAt(SortOrder),
    UpdatedBy(SortOrder),
    FileType(SortOrder),
    Classification(SortOrder),
    Size(SortOrder),
    CntDeletedVersions(SortOrder),
    TimestampCreation(SortOrder),
    TimestampModification(SortOrder),
}

impl From<NodesSortBy> for String {
    fn from(sort_by: NodesSortBy) -> Self {
        match sort_by {
            NodesSortBy::Name(order) => {
                let order = String::from(order);
                format!("name:{}", order)
            }
            NodesSortBy::CreatedAt(order) => {
                let order = String::from(order);
                format!("createdAt:{}", order)
            }
            NodesSortBy::CreatedBy(order) => {
                let order = String::from(order);
                format!("createdBy:{}", order)
            }
            NodesSortBy::UpdatedAt(order) => {
                let order = String::from(order);
                format!("updatedAt:{}", order)
            }
            NodesSortBy::UpdatedBy(order) => {
                let order = String::from(order);
                format!("updatedBy:{}", order)
            }
            NodesSortBy::FileType(order) => {
                let order = String::from(order);
                format!("fileType:{}", order)
            }
            NodesSortBy::Classification(order) => {
                let order = String::from(order);
                format!("classification:{}", order)
            }
            NodesSortBy::Size(order) => {
                let order = String::from(order);
                format!("size:{}", order)
            }
            NodesSortBy::CntDeletedVersions(order) => {
                let order = String::from(order);
                format!("cntDeletedVersions:{}", order)
            }
            NodesSortBy::TimestampCreation(order) => {
                let order = String::from(order);
                format!("timestampCreation:{}", order)
            }
            NodesSortBy::TimestampModification(order) => {
                let order = String::from(order);
                format!("timestampModification:{}", order)
            }
        }
    }
}

impl NodesSortBy {
    pub fn name(order: SortOrder) -> Self {
        NodesSortBy::Name(order)
    }

    pub fn created_at(order: SortOrder) -> Self {
        NodesSortBy::CreatedAt(order)
    }

    pub fn created_by(order: SortOrder) -> Self {
        NodesSortBy::CreatedBy(order)
    }

    pub fn updated_at(order: SortOrder) -> Self {
        NodesSortBy::UpdatedAt(order)
    }

    pub fn updated_by(order: SortOrder) -> Self {
        NodesSortBy::UpdatedBy(order)
    }

    pub fn file_type(order: SortOrder) -> Self {
        NodesSortBy::FileType(order)
    }

    pub fn classification(order: SortOrder) -> Self {
        NodesSortBy::Classification(order)
    }

    pub fn size(order: SortOrder) -> Self {
        NodesSortBy::Size(order)
    }

    pub fn cnt_deleted_versions(order: SortOrder) -> Self {
        NodesSortBy::CntDeletedVersions(order)
    }

    pub fn timestamp_creation(order: SortOrder) -> Self {
        NodesSortBy::TimestampCreation(order)
    }

    pub fn timestamp_modification(order: SortOrder) -> Self {
        NodesSortBy::TimestampModification(order)
    }
}

#[derive(Debug, Clone)]
pub enum NodesSearchSortBy {
    Name(SortOrder),
    CreatedAt(SortOrder),
    CreatedBy(SortOrder),
    UpdatedAt(SortOrder),
    UpdatedBy(SortOrder),
    FileType(SortOrder),
    Classification(SortOrder),
    Size(SortOrder),
    CntDeletedVersions(SortOrder),
    Type(SortOrder),
    ParentPath(SortOrder),
    TimestampCreation(SortOrder),
    TimestampModification(SortOrder),
}

impl From<NodesSearchSortBy> for String {
    fn from(value: NodesSearchSortBy) -> Self {
        match value {
            NodesSearchSortBy::Name(order) => {
                let order = String::from(order);
                format!("name:{}", order)
            }
            NodesSearchSortBy::CreatedAt(order) => {
                let order = String::from(order);
                format!("createdAt:{}", order)
            }
            NodesSearchSortBy::CreatedBy(order) => {
                let order = String::from(order);
                format!("createdBy:{}", order)
            }
            NodesSearchSortBy::UpdatedAt(order) => {
                let order = String::from(order);
                format!("updatedAt:{}", order)
            }
            NodesSearchSortBy::UpdatedBy(order) => {
                let order = String::from(order);
                format!("updatedBy:{}", order)
            }
            NodesSearchSortBy::FileType(order) => {
                let order = String::from(order);
                format!("fileType:{}", order)
            }
            NodesSearchSortBy::Classification(order) => {
                let order = String::from(order);
                format!("classification:{}", order)
            }
            NodesSearchSortBy::Size(order) => {
                let order = String::from(order);
                format!("size:{}", order)
            }
            NodesSearchSortBy::CntDeletedVersions(order) => {
                let order = String::from(order);
                format!("cntDeletedVersions:{}", order)
            }
            NodesSearchSortBy::Type(order) => {
                let order = String::from(order);
                format!("type:{}", order)
            }
            NodesSearchSortBy::ParentPath(order) => {
                let order = String::from(order);
                format!("parentPath:{}", order)
            }
            NodesSearchSortBy::TimestampCreation(order) => {
                let order = String::from(order);
                format!("timestampCreation:{}", order)
            }
            NodesSearchSortBy::TimestampModification(order) => {
                let order = String::from(order);
                format!("timestampModification:{}", order)
            }
        }
    }
}

impl NodesSearchSortBy {
    pub fn parent_path(order: SortOrder) -> Self {
        NodesSearchSortBy::ParentPath(order)
    }

    pub fn type_(order: SortOrder) -> Self {
        NodesSearchSortBy::Type(order)
    }

    pub fn name(order: SortOrder) -> Self {
        NodesSearchSortBy::Name(order)
    }

    pub fn created_at(order: SortOrder) -> Self {
        NodesSearchSortBy::CreatedAt(order)
    }

    pub fn created_by(order: SortOrder) -> Self {
        NodesSearchSortBy::CreatedBy(order)
    }

    pub fn updated_at(order: SortOrder) -> Self {
        NodesSearchSortBy::UpdatedAt(order)
    }

    pub fn updated_by(order: SortOrder) -> Self {
        NodesSearchSortBy::UpdatedBy(order)
    }

    pub fn file_type(order: SortOrder) -> Self {
        NodesSearchSortBy::FileType(order)
    }

    pub fn classification(order: SortOrder) -> Self {
        NodesSearchSortBy::Classification(order)
    }

    pub fn size(order: SortOrder) -> Self {
        NodesSearchSortBy::Size(order)
    }

    pub fn cnt_deleted_versions(order: SortOrder) -> Self {
        NodesSearchSortBy::CntDeletedVersions(order)
    }

    pub fn timestamp_creation(order: SortOrder) -> Self {
        NodesSearchSortBy::TimestampCreation(order)
    }

    pub fn timestamp_modification(order: SortOrder) -> Self {
        NodesSearchSortBy::TimestampModification(order)
    }
}

impl SortQuery for NodesSearchSortBy {
    fn to_sort_string(&self) -> String {
        match self {
            NodesSearchSortBy::Name(order) => {
                let order = String::from(order);
                format!("name:{}", order)
            }
            NodesSearchSortBy::CreatedAt(order) => {
                let order = String::from(order);
                format!("createdAt:{}", order)
            }
            NodesSearchSortBy::CreatedBy(order) => {
                let order = String::from(order);
                format!("createdBy:{}", order)
            }
            NodesSearchSortBy::UpdatedAt(order) => {
                let order = String::from(order);
                format!("updatedAt:{}", order)
            }
            NodesSearchSortBy::UpdatedBy(order) => {
                let order = String::from(order);
                format!("updatedBy:{}", order)
            }
            NodesSearchSortBy::FileType(order) => {
                let order = String::from(order);
                format!("fileType:{}", order)
            }
            NodesSearchSortBy::Classification(order) => {
                let order = String::from(order);
                format!("classification:{}", order)
            }
            NodesSearchSortBy::Size(order) => {
                let order = String::from(order);
                format!("size:{}", order)
            }
            NodesSearchSortBy::CntDeletedVersions(order) => {
                let order = String::from(order);
                format!("cntDeletedVersions:{}", order)
            }
            NodesSearchSortBy::Type(order) => {
                let order = String::from(order);
                format!("type:{}", order)
            }
            NodesSearchSortBy::ParentPath(order) => {
                let order = String::from(order);
                format!("parentPath:{}", order)
            }
            NodesSearchSortBy::TimestampCreation(order) => {
                let order = String::from(order);
                format!("timestampCreation:{}", order)
            }
            NodesSearchSortBy::TimestampModification(order) => {
                let order = String::from(order);
                format!("timestampModification:{}", order)
            }
        }
    }
}

impl From<NodesSearchSortBy> for Box<dyn SortQuery> {
    fn from(value: NodesSearchSortBy) -> Self {
        Box::new(value)
    }
}

impl From<NodesSortBy> for Box<dyn SortQuery> {
    fn from(value: NodesSortBy) -> Self {
        Box::new(value)
    }
}

impl SortQuery for NodesSortBy {
    fn to_sort_string(&self) -> String {
        match self {
            NodesSortBy::Classification(order) => {
                let order = String::from(order);
                format!("classification:{}", order)
            }
            NodesSortBy::CreatedAt(order) => {
                let order = String::from(order);
                format!("createdAt:{}", order)
            }
            NodesSortBy::CreatedBy(order) => {
                let order = String::from(order);
                format!("createdBy:{}", order)
            }
            NodesSortBy::FileType(order) => {
                let order = String::from(order);
                format!("fileType:{}", order)
            }
            NodesSortBy::Name(order) => {
                let order = String::from(order);
                format!("name:{}", order)
            }
            NodesSortBy::Size(order) => {
                let order = String::from(order);
                format!("size:{}", order)
            }
            NodesSortBy::UpdatedAt(order) => {
                let order = String::from(order);
                format!("updatedAt:{}", order)
            }
            NodesSortBy::UpdatedBy(order) => {
                let order = String::from(order);
                format!("updatedBy:{}", order)
            }
            NodesSortBy::CntDeletedVersions(order) => {
                let order = String::from(order);
                format!("cntDeletedVersions:{}", order)
            }
            NodesSortBy::TimestampCreation(order) => {
                let order = String::from(order);
                format!("timestampCreation:{}", order)
            }
            NodesSortBy::TimestampModification(order) => {
                let order = String::from(order);
                format!("timestampModification:{}", order)
            }
        }
    }
}

#[cfg(test)]

mod tests {
    use super::*;

    #[test]
    fn test_nodes_sort_by_name_asc() {
        let sort_by = NodesSortBy::name(SortOrder::Asc);
        assert_eq!(sort_by.to_sort_string(), "name:asc");
    }

    #[test]
    fn test_nodes_sort_by_name_desc() {
        let sort_by = NodesSortBy::name(SortOrder::Desc);
        assert_eq!(sort_by.to_sort_string(), "name:desc");
    }

    #[test]
    fn test_nodes_sort_by_created_at_asc() {
        let sort_by = NodesSortBy::created_at(SortOrder::Asc);
        assert_eq!(sort_by.to_sort_string(), "createdAt:asc");
    }

    #[test]
    fn test_nodes_sort_by_created_at_desc() {
        let sort_by = NodesSortBy::created_at(SortOrder::Desc);
        assert_eq!(sort_by.to_sort_string(), "createdAt:desc");
    }

    #[test]
    fn test_nodes_sort_by_created_by_asc() {
        let sort_by = NodesSortBy::created_by(SortOrder::Asc);
        assert_eq!(sort_by.to_sort_string(), "createdBy:asc");
    }

    #[test]
    fn test_nodes_sort_by_created_by_desc() {
        let sort_by = NodesSortBy::created_by(SortOrder::Desc);
        assert_eq!(sort_by.to_sort_string(), "createdBy:desc");
    }

    #[test]
    fn test_nodes_sort_by_updated_at_asc() {
        let sort_by = NodesSortBy::updated_at(SortOrder::Asc);
        assert_eq!(sort_by.to_sort_string(), "updatedAt:asc");
    }

    #[test]
    fn test_nodes_sort_by_updated_at_desc() {
        let sort_by = NodesSortBy::updated_at(SortOrder::Desc);
        assert_eq!(sort_by.to_sort_string(), "updatedAt:desc");
    }

    #[test]
    fn test_nodes_sort_by_updated_by_asc() {
        let sort_by = NodesSortBy::updated_by(SortOrder::Asc);
        assert_eq!(sort_by.to_sort_string(), "updatedBy:asc");
    }

    #[test]
    fn test_nodes_sort_by_updated_by_desc() {
        let sort_by = NodesSortBy::updated_by(SortOrder::Desc);
        assert_eq!(sort_by.to_sort_string(), "updatedBy:desc");
    }

    #[test]
    fn test_nodes_sort_by_file_type_asc() {
        let sort_by = NodesSortBy::file_type(SortOrder::Asc);
        assert_eq!(sort_by.to_sort_string(), "fileType:asc");
    }

    #[test]
    fn test_nodes_sort_by_file_type_desc() {
        let sort_by = NodesSortBy::file_type(SortOrder::Desc);
        assert_eq!(sort_by.to_sort_string(), "fileType:desc");
    }

    #[test]
    fn test_nodes_sort_by_classification_asc() {
        let sort_by = NodesSortBy::classification(SortOrder::Asc);
        assert_eq!(sort_by.to_sort_string(), "classification:asc");
    }

    #[test]
    fn test_nodes_sort_by_classification_desc() {
        let sort_by = NodesSortBy::classification(SortOrder::Desc);
        assert_eq!(sort_by.to_sort_string(), "classification:desc");
    }

    #[test]
    fn test_nodes_sort_by_size_asc() {
        let sort_by = NodesSortBy::size(SortOrder::Asc);
        assert_eq!(sort_by.to_sort_string(), "size:asc");
    }

    #[test]
    fn test_nodes_sort_by_size_desc() {
        let sort_by = NodesSortBy::size(SortOrder::Desc);
        assert_eq!(sort_by.to_sort_string(), "size:desc");
    }

    #[test]
    fn test_nodes_sort_by_cnt_deleted_versions_asc() {
        let sort_by = NodesSortBy::cnt_deleted_versions(SortOrder::Asc);
        assert_eq!(sort_by.to_sort_string(), "cntDeletedVersions:asc");
    }

    #[test]
    fn test_nodes_sort_by_cnt_deleted_versions_desc() {
        let sort_by = NodesSortBy::cnt_deleted_versions(SortOrder::Desc);
        assert_eq!(sort_by.to_sort_string(), "cntDeletedVersions:desc");
    }

    #[test]
    fn test_nodes_sort_by_timestamp_creation_asc() {
        let sort_by = NodesSortBy::timestamp_creation(SortOrder::Asc);
        assert_eq!(sort_by.to_sort_string(), "timestampCreation:asc");
    }

    #[test]
    fn test_nodes_sort_by_timestamp_creation_desc() {
        let sort_by = NodesSortBy::timestamp_creation(SortOrder::Desc);
        assert_eq!(sort_by.to_sort_string(), "timestampCreation:desc");
    }

    #[test]
    fn test_nodes_sort_by_timestamp_modification_asc() {
        let sort_by = NodesSortBy::timestamp_modification(SortOrder::Asc);
        assert_eq!(sort_by.to_sort_string(), "timestampModification:asc");
    }

    #[test]
    fn test_nodes_sort_by_timestamp_modification_desc() {
        let sort_by = NodesSortBy::timestamp_modification(SortOrder::Desc);
        assert_eq!(sort_by.to_sort_string(), "timestampModification:desc");
    }

    #[test]
    fn test_nodes_search_sort_by_name_asc() {
        let sort_by = NodesSearchSortBy::name(SortOrder::Asc);
        assert_eq!(sort_by.to_sort_string(), "name:asc");
    }

    #[test]
    fn test_nodes_search_sort_by_name_desc() {
        let sort_by = NodesSearchSortBy::name(SortOrder::Desc);
        assert_eq!(sort_by.to_sort_string(), "name:desc");
    }

    #[test]
    fn test_nodes_search_sort_by_created_at_asc() {
        let sort_by = NodesSearchSortBy::created_at(SortOrder::Asc);
        assert_eq!(sort_by.to_sort_string(), "createdAt:asc");
    }

    #[test]
    fn test_nodes_search_sort_by_created_at_desc() {
        let sort_by = NodesSearchSortBy::created_at(SortOrder::Desc);
        assert_eq!(sort_by.to_sort_string(), "createdAt:desc");
    }

    #[test]
    fn test_nodes_search_sort_by_created_by_asc() {
        let sort_by = NodesSearchSortBy::created_by(SortOrder::Asc);
        assert_eq!(sort_by.to_sort_string(), "createdBy:asc");
    }

    #[test]
    fn test_nodes_search_sort_by_created_by_desc() {
        let sort_by = NodesSearchSortBy::created_by(SortOrder::Desc);
        assert_eq!(sort_by.to_sort_string(), "createdBy:desc");
    }

    #[test]
    fn test_nodes_search_sort_by_updated_at_asc() {
        let sort_by = NodesSearchSortBy::updated_at(SortOrder::Asc);
        assert_eq!(sort_by.to_sort_string(), "updatedAt:asc");
    }

    #[test]
    fn test_nodes_search_sort_by_updated_at_desc() {
        let sort_by = NodesSearchSortBy::updated_at(SortOrder::Desc);
        assert_eq!(sort_by.to_sort_string(), "updatedAt:desc");
    }

    #[test]
    fn test_nodes_search_sort_by_updated_by_asc() {
        let sort_by = NodesSearchSortBy::updated_by(SortOrder::Asc);
        assert_eq!(sort_by.to_sort_string(), "updatedBy:asc");
    }

    #[test]
    fn test_nodes_search_sort_by_updated_by_desc() {
        let sort_by = NodesSearchSortBy::updated_by(SortOrder::Desc);
        assert_eq!(sort_by.to_sort_string(), "updatedBy:desc");
    }

    #[test]
    fn test_nodes_search_sort_by_file_type_asc() {
        let sort_by = NodesSearchSortBy::file_type(SortOrder::Asc);
        assert_eq!(sort_by.to_sort_string(), "fileType:asc");
    }

    #[test]
    fn test_nodes_search_sort_by_file_type_desc() {
        let sort_by = NodesSearchSortBy::file_type(SortOrder::Desc);
        assert_eq!(sort_by.to_sort_string(), "fileType:desc");
    }

    #[test]
    fn test_nodes_search_sort_by_classification_asc() {
        let sort_by = NodesSearchSortBy::classification(SortOrder::Asc);
        assert_eq!(sort_by.to_sort_string(), "classification:asc");
    }

    #[test]
    fn test_nodes_search_sort_by_classification_desc() {
        let sort_by = NodesSearchSortBy::classification(SortOrder::Desc);
        assert_eq!(sort_by.to_sort_string(), "classification:desc");
    }

    #[test]
    fn test_nodes_search_sort_by_size_asc() {
        let sort_by = NodesSearchSortBy::size(SortOrder::Asc);
        assert_eq!(sort_by.to_sort_string(), "size:asc");
    }

    #[test]
    fn test_nodes_search_sort_by_size_desc() {
        let sort_by = NodesSearchSortBy::size(SortOrder::Desc);
        assert_eq!(sort_by.to_sort_string(), "size:desc");
    }

    #[test]
    fn test_nodes_search_sort_by_cnt_deleted_versions_asc() {
        let sort_by = NodesSearchSortBy::cnt_deleted_versions(SortOrder::Asc);
        assert_eq!(sort_by.to_sort_string(), "cntDeletedVersions:asc");
    }

    #[test]
    fn test_nodes_search_sort_by_cnt_deleted_versions_desc() {
        let sort_by = NodesSearchSortBy::cnt_deleted_versions(SortOrder::Desc);
        assert_eq!(sort_by.to_sort_string(), "cntDeletedVersions:desc");
    }

    #[test]
    fn test_nodes_search_sort_by_timestamp_creation_asc() {
        let sort_by = NodesSearchSortBy::timestamp_creation(SortOrder::Asc);
        assert_eq!(sort_by.to_sort_string(), "timestampCreation:asc");
    }

    #[test]
    fn test_nodes_search_sort_by_timestamp_creation_desc() {
        let sort_by = NodesSearchSortBy::timestamp_creation(SortOrder::Desc);
        assert_eq!(sort_by.to_sort_string(), "timestampCreation:desc");
    }

    #[test]
    fn test_nodes_search_sort_by_timestamp_modification_asc() {
        let sort_by = NodesSearchSortBy::timestamp_modification(SortOrder::Asc);
        assert_eq!(sort_by.to_sort_string(), "timestampModification:asc");
    }

    #[test]
    fn test_nodes_search_sort_by_timestamp_modification_desc() {
        let sort_by = NodesSearchSortBy::timestamp_modification(SortOrder::Desc);
        assert_eq!(sort_by.to_sort_string(), "timestampModification:desc");
    }
}
