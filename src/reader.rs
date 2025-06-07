//! Main reader implementation for ROS2 bag files

use crate::error::{ReaderError, Result};
use crate::metadata::BagMetadata;
use crate::storage::{create_storage_reader, StorageReader};
use crate::types::{Connection, Message, MessageDefinition, TopicInfo};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Main reader for ROS2 bag files
pub struct Reader {
    /// Path to the bag directory
    bag_path: PathBuf,
    /// Parsed metadata
    metadata: Option<BagMetadata>,
    /// Storage backend
    storage: Option<Box<dyn StorageReader>>,
    /// Connections (topics) in the bag
    connections: Vec<Connection>,
    /// Whether the reader is currently open
    is_open: bool,
}

impl Reader {
    /// Create a new reader for the given bag path
    pub fn new<P: AsRef<Path>>(bag_path: P) -> Result<Self> {
        let bag_path = bag_path.as_ref().to_path_buf();

        // Check if the bag directory exists
        if !bag_path.exists() {
            return Err(ReaderError::BagNotFound { path: bag_path });
        }

        // Load metadata
        let metadata_path = bag_path.join("metadata.yaml");
        let metadata = BagMetadata::from_file(&metadata_path)?;

        Ok(Self {
            bag_path,
            metadata: Some(metadata),
            storage: None,
            connections: Vec::new(),
            is_open: false,
        })
    }

    /// Open the bag for reading
    pub fn open(&mut self) -> Result<()> {
        if self.is_open {
            return Ok(());
        }

        let metadata = self.metadata.as_ref().unwrap();
        let info = metadata.info();

        // Build connections from metadata
        self.connections = info
            .topics_with_message_count
            .iter()
            .enumerate()
            .map(|(idx, topic)| {
                let qos_profiles = match &topic.topic_metadata.offered_qos_profiles {
                    crate::metadata::QosProfilesField::String(_) => Vec::new(),
                    crate::metadata::QosProfilesField::List(profiles) => profiles.clone(),
                };

                Connection {
                    id: (idx + 1) as u32,
                    topic: topic.topic_metadata.name.clone(),
                    message_type: topic.topic_metadata.message_type.clone(),
                    message_definition: MessageDefinition::default(),
                    type_description_hash: topic.topic_metadata.type_description_hash.clone(),
                    message_count: topic.message_count,
                    serialization_format: topic.topic_metadata.serialization_format.clone(),
                    offered_qos_profiles: qos_profiles,
                }
            })
            .collect();

        // Resolve storage file paths
        let storage_paths: Vec<PathBuf> = info
            .relative_file_paths
            .iter()
            .map(|path| self.bag_path.join(path))
            .collect();

        // Check that all storage files exist
        for path in &storage_paths {
            if !path.exists() {
                return Err(ReaderError::StorageFileNotFound { path: path.clone() });
            }
        }

        // Create storage reader
        let storage_path_refs: Vec<&Path> = storage_paths.iter().map(|p| p.as_path()).collect();
        let mut storage = create_storage_reader(
            &info.storage_identifier,
            storage_path_refs,
            self.connections.clone(),
        )?;

        // Open storage
        storage.open()?;

        // Get actual topics from the storage (this may be more complete than metadata)
        if let Some(sqlite_storage) = storage
            .as_any()
            .downcast_ref::<crate::storage::sqlite::SqliteReader>()
        {
            match sqlite_storage.get_topics_from_database() {
                Ok(db_connections) => {
                    if !db_connections.is_empty() {
                        // Use database connections if available (more reliable)
                        self.connections = db_connections;
                    }
                    // Otherwise keep the metadata-based connections as fallback
                }
                Err(e) => {
                    eprintln!("Warning: Failed to read topics from database: {}", e);
                    // Continue with metadata-based connections
                }
            }
        }

        #[cfg(feature = "mcap")]
        if let Some(mcap_storage) = storage
            .as_any()
            .downcast_ref::<crate::storage::mcap::McapStorageReader>()
        {
            match mcap_storage.get_topics_from_mcap() {
                Ok(mcap_connections) => {
                    if !mcap_connections.is_empty() {
                        // For MCAP, prefer metadata-based message types but use MCAP message counts
                        // This gives us the correct ROS2 message types from metadata.yaml
                        // but accurate message counts from the actual MCAP file
                        for mcap_conn in &mcap_connections {
                            if let Some(metadata_conn) = self
                                .connections
                                .iter_mut()
                                .find(|c| c.topic == mcap_conn.topic)
                            {
                                // Update message count from MCAP (more accurate)
                                metadata_conn.message_count = mcap_conn.message_count;
                            } else {
                                // Topic exists in MCAP but not in metadata - add it
                                self.connections.push(mcap_conn.clone());
                            }
                        }

                        // If metadata had no topics, use MCAP connections as fallback
                        if self.connections.is_empty() {
                            self.connections = mcap_connections;
                        }
                    }
                    // Otherwise keep the metadata-based connections as fallback
                }
                Err(e) => {
                    eprintln!("Warning: Failed to read topics from MCAP: {}", e);
                    // Continue with metadata-based connections
                }
            }
        }

        // Get message definitions from storage and update connections
        let definitions = storage.get_definitions()?;
        for connection in &mut self.connections {
            if let Some(def) = definitions.get(&connection.message_type) {
                connection.message_definition = def.clone();
            }
        }

        self.storage = Some(storage);
        self.is_open = true;

        Ok(())
    }

    /// Close the bag
    pub fn close(&mut self) -> Result<()> {
        if !self.is_open {
            return Ok(());
        }

        if let Some(mut storage) = self.storage.take() {
            storage.close()?;
        }

        self.is_open = false;
        Ok(())
    }

    /// Get the bag duration in nanoseconds
    pub fn duration(&self) -> u64 {
        self.metadata.as_ref().map_or(0, |m| m.duration())
    }

    /// Get the start time in nanoseconds since epoch
    pub fn start_time(&self) -> u64 {
        self.metadata.as_ref().map_or(0, |m| m.start_time())
    }

    /// Get the end time in nanoseconds since epoch
    pub fn end_time(&self) -> u64 {
        self.metadata.as_ref().map_or(0, |m| m.end_time())
    }

    /// Get the total message count
    pub fn message_count(&self) -> u64 {
        self.metadata.as_ref().map_or(0, |m| m.message_count())
    }

    /// Get information about all topics in the bag
    pub fn topics(&self) -> Vec<TopicInfo> {
        if !self.is_open {
            return Vec::new();
        }

        let mut topic_map: HashMap<String, TopicInfo> = HashMap::new();

        for connection in &self.connections {
            let topic_info = topic_map
                .entry(connection.topic.clone())
                .or_insert_with(|| TopicInfo {
                    name: connection.topic.clone(),
                    message_type: connection.message_type.clone(),
                    message_definition: connection.message_definition.clone(),
                    message_count: 0,
                    connections: Vec::new(),
                });

            topic_info.message_count += connection.message_count;
            topic_info.connections.push(connection.clone());
        }

        topic_map.into_values().collect()
    }

    /// Get all connections
    pub fn connections(&self) -> &[Connection] {
        &self.connections
    }

    /// Iterate over all messages in the bag
    pub fn messages(&self) -> Result<Box<dyn Iterator<Item = Result<Message>> + '_>> {
        self.messages_filtered(None, None, None)
    }

    /// Iterate over messages with optional filters
    pub fn messages_filtered(
        &self,
        connections: Option<&[Connection]>,
        start: Option<u64>,
        stop: Option<u64>,
    ) -> Result<Box<dyn Iterator<Item = Result<Message>> + '_>> {
        if !self.is_open {
            return Err(ReaderError::BagNotOpen);
        }

        let storage = self.storage.as_ref().unwrap();
        storage.messages(connections, start, stop)
    }

    /// Check if the bag is currently open
    pub fn is_open(&self) -> bool {
        self.is_open
    }

    /// Get the metadata
    pub fn metadata(&self) -> Option<&BagMetadata> {
        self.metadata.as_ref()
    }
}

impl Drop for Reader {
    fn drop(&mut self) {
        let _ = self.close();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_metadata() -> String {
        r#"
rosbag2_bagfile_information:
  version: 4
  storage_identifier: sqlite3
  relative_file_paths:
    - test.db3
  duration:
    nanoseconds: 1000000000
  starting_time:
    nanoseconds_since_epoch: 1234567890000000000
  message_count: 10
  compression_format: ""
  compression_mode: ""
  topics_with_message_count:
    - topic_metadata:
        name: /test_topic
        type: std_msgs/msg/String
        serialization_format: cdr
        offered_qos_profiles: ""
        type_description_hash: ""
      message_count: 10
"#
        .trim()
        .to_string()
    }

    #[test]
    fn test_reader_creation_with_missing_bag() {
        let result = Reader::new("/nonexistent/path");
        assert!(matches!(result, Err(ReaderError::BagNotFound { .. })));
    }

    #[test]
    fn test_reader_creation_with_missing_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let result = Reader::new(temp_dir.path());
        assert!(matches!(result, Err(ReaderError::MetadataNotFound { .. })));
    }

    #[test]
    fn test_reader_creation_success() {
        let temp_dir = TempDir::new().unwrap();
        let metadata_path = temp_dir.path().join("metadata.yaml");
        fs::write(&metadata_path, create_test_metadata()).unwrap();

        // Create empty database file
        let db_path = temp_dir.path().join("test.db3");
        fs::write(&db_path, b"").unwrap();

        let reader = Reader::new(temp_dir.path());
        assert!(reader.is_ok());

        let reader = reader.unwrap();
        assert!(!reader.is_open());
        assert_eq!(reader.duration(), 1000000000);
        assert_eq!(reader.message_count(), 10);
    }
}
