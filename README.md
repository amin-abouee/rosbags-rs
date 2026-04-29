# rosbags-rs

> ⚠️ **Notice: This repository is archived.**
> The code has been merged into a monorepo. Active development now takes place at [amin-abouee/apex-solver/crates/apex-io](https://github.com/amin-abouee/apex-solver/crates/apex-io).

[![CI](https://github.com/your-org/rosbags-rs/workflows/CI/badge.svg)](https://github.com/your-org/rosbags-rs/actions)
[![Crates.io](https://img.shields.io/crates/v/rosbags-rs.svg)](https://crates.io/crates/rosbags-rs)
[![Documentation](https://docs.rs/rosbags-rs/badge.svg)](https://docs.rs/rosbags-rs)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue.svg)](LICENSE)

A high-performance Rust library for reading and writing ROS2 bag files with **full Python rosbags compatibility**. This library provides comprehensive functionality to read and write ROS2 bag files in both SQLite3 and MCAP formats, with guaranteed byte-for-byte identical results compared to the Python rosbags library.

## 🚀 Features

- ✅ **Complete ROS2 bag reading and writing** - SQLite3 and MCAP formats
- ✅ **94+ ROS2 message types** - Full support across all major categories
- ✅ **Python rosbags compatibility** - Byte-for-byte identical results
- ✅ **High performance** - Zero-copy message reading where possible
- ✅ **Comprehensive CDR deserialization** - All standard ROS2 message types
- ✅ **Advanced filtering** - By topic, time range, and message type
- ✅ **Compression support** - zstd compressed bags
- ✅ **Type-safe error handling** - Comprehensive error types
- ✅ **Self-contained tests** - No external dependencies required
- ✅ **Production ready** - Extensive test coverage and CI/CD

## 🎯 Supported ROS2 Versions

- **ROS2 Jazzy Jalopy** (LTS)
- **ROS2 Humble Hawksbill** (LTS)
- **ROS2 Foxy Fitzroy** (LTS)

## 📦 Installation

Add this to your `Cargo.toml`:

```toml
[dependencies]
rosbags-rs = "0.2.0"
```

## 🔧 Command Line Tools

This library includes several command-line utilities for working with ROS2 bag files:

### `bag_filter` - Copy and filter bag files
High-performance bag copying with topic and time filtering:

```bash
# Copy entire bag
cargo run --bin bag_filter -- /path/to/input_bag /path/to/output_bag

# Copy specific topics only
cargo run --bin bag_filter -- /path/to/input_bag /path/to/output_bag --topics /imu/data,/camera/image_raw

# Copy with time filtering
cargo run --bin bag_filter -- /path/to/input_bag /path/to/output_bag --start 1000000000 --end 2000000000

# List available topics
cargo run --bin bag_filter -- /path/to/input_bag /path/to/output_bag --list-topics

# Use verbose output
cargo run --bin bag_filter -- /path/to/input_bag /path/to/output_bag --verbose
```

### `bag_info` - Display bag information
Show metadata and statistics about bag files:

```bash
cargo run --bin bag_info -- /path/to/rosbag2_directory
```

### `extract_topic_data` - Extract topic data to files
Extract specific topic data and save to appropriate file formats:

```bash
cargo run --bin extract_topic_data -- /path/to/bag /topic_name /output/directory
```

### `write_dummy_bag` - Create test bags
Generate test bag files with sample data for testing:

```bash
cargo run --bin write_dummy_bag -- /path/to/output_bag
```

## 🗂️ Supported ROS2 Bag Formats

### Storage Formats

- ✅ **SQLite3** - Primary storage format for ROS2 bags
- ✅ **MCAP** - Modern container format with high performance

### Compression

- ✅ **None** - Uncompressed bags
- ✅ **zstd** - File-level and message-level compression
- ❌ **lz4** - Not currently supported

### Bag Versions

- ✅ **Version 1-9** - All current ROS2 bag versions supported

## 🏗️ Architecture

The library is structured into several modules:

- **`reader`** - Main `Reader` struct for opening and reading bags
- **`writer`** - Main `Writer` struct for creating and writing bags
- **`metadata`** - Parsing and validation of `metadata.yaml` files
- **`storage`** - Storage backend implementations (SQLite3, MCAP)
- **`types`** - Core data structures (Connection, Message, TopicInfo, etc.)
- **`error`** - Comprehensive error handling
- **`cdr`** - CDR message deserialization
- **`messages`** - ROS2 message type definitions

## 🛡️ Error Handling

The library uses the `thiserror` crate for structured error handling:

```rust
use rosbags_rs::{Reader, ReaderError};

match Reader::new("/path/to/bag") {
    Ok(reader) => { /* success */ },
    Err(ReaderError::BagNotFound { path }) => {
        eprintln!("Bag not found: {}", path.display());
    },
    Err(ReaderError::UnsupportedVersion { version }) => {
        eprintln!("Unsupported bag version: {}", version);
    },
    Err(e) => {
        eprintln!("Other error: {}", e);
    }
}
```

## 📊 Supported ROS2 Message Types

This library supports **94+ ROS2 message types** across all major categories:

### Core Message Categories
- **📡 std_msgs** - Standard message types (String, Header, etc.)
- **📐 geometry_msgs** - Geometric primitives (Point, Pose, Transform, etc.)
- **🤖 sensor_msgs** - Sensor data (Image, PointCloud2, Imu, NavSatFix, etc.)
- **🗺️ nav_msgs** - Navigation messages (Odometry, Path, etc.)
- **🔧 diagnostic_msgs** - System diagnostics
- **⏰ builtin_interfaces** - Time and duration types

### Cross-Compatibility Guarantee

This Rust implementation provides **100% compatibility** with the Python rosbags library:

| Feature | Python rosbags | rosbags-rs |
|---------|----------------|------------|
| SQLite3 reading | ✅ | ✅ |
| MCAP reading | ✅ | ✅ |
| SQLite3 writing | ✅ | ✅ |
| MCAP writing | ✅ | ✅ |
| CDR deserialization | ✅ | ✅ |
| Message filtering | ✅ | ✅ |
| Compression support | ✅ | ✅ |
| Type safety | ❌ | ✅ |
| Memory safety | ❌ | ✅ |
| Performance | Good | **Excellent** |
| Cross-validation | N/A | **Byte-for-byte identical** |

## 🚀 Performance

- **Zero-copy message reading** where possible
- **Optimized SQL queries** for SQLite3 backend
- **SIMD-accelerated parsing** for MCAP backend (future work)
- **Lazy-loading of message data** - only read what you need
- **Minimal memory allocations** - focus on performance and efficiency
- **Bulk operations** - Batch reading and writing for maximum throughput

The library is designed for high-throughput applications where performance is critical. The `bag_filter` tool uses optimized raw copying by default, similar to `ros2 bag convert`, for maximum speed.

## 🧪 Testing

This library includes a comprehensive test suite that validates correctness against the Python `rosbags` library. All tests are self-contained and do not require an external ROS2 installation.

### Running Tests

```bash
cargo test -- --nocapture
```

The tests cover:
- **Unit tests** for individual modules
- **Integration tests** for reading complete bag files
- **Compatibility tests** to ensure byte-for-byte identical results with Python `rosbags`
- **Fuzz testing** to uncover edge cases and potential panics

### Test Data

The test bags are generated using the `generate_test_bags.py` script and are included in the repository.

## 🤝 Contributing

Contributions are welcome! Please feel free to submit a pull request or open an issue.

### Development Setup

1. Clone the repository
2. Install Rust: `curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh`
3. Build the project: `cargo build`
4. Run tests: `cargo test`

### Code Style

This project uses `rustfmt` for code formatting and `clippy` for linting. Please ensure your contributions are formatted and free of warnings.

```bash
cargo fmt
cargo clippy -- -D warnings
```

## 📜 License

This project is licensed under the Apache 2.0 License - see the [LICENSE](LICENSE) file for details.

