//! Vector storage, indexing, and search infrastructure.

mod dense;

pub mod index;
pub mod metadata;
pub mod search;
pub mod storage;

pub use dense::DenseVector;
pub use index::VectorIndex;
pub use metadata::{MetadataStore, VectorMetadata};
pub use search::{SearchResult, SearchResultSet};
pub use storage::{StoredVector, VectorStorage, VectorStorageStatistics};
