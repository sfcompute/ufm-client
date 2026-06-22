# ufm-client

A Rust client library for NVIDIA's Unified Fabric Manager (UFM) REST API.

## Features

- **Type-safe API** — Strongly-typed domain models (`Pkey`, `Guid`, `Membership`, etc.)
- **Async/await** — Built on tokio and reqwest
- **Builder pattern** — Fluent API for complex operations
- **Pluggable auth** — Basic and token authentication
- **Automatic retries** — Configurable retry logic for transient failures

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
ufm-client = { git = "https://github.com/sfcompute/ufm-client.git" }
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
```

## Quick Start

```rust
use ufm_client::{UfmClient, Auth};
use ufm_client::types::{Pkey, Guid, Membership};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create client
    let client = UfmClient::builder()
        .base_url("https://ufm-server")
        .auth(Auth::basic("admin", "password"))
        .verify_ssl(false)  // For self-signed certificates
        .build()?;

    // Get UFM version
    let version = client.version().await?;
    println!("UFM: {}", version.ufm_release_version);

    // List partition keys
    let pkeys = client.pkeys().list().await?;
    
    // Create a partition
    let pkey: Pkey = "0x12".parse()?;
    client.pkeys()
        .create(&pkey)
        .ip_over_ib(true)
        .send()
        .await?;

    // Add GUIDs to partition
    let guid: Guid = "0002c903000e0b72".parse()?;
    client.pkeys()
        .add_guids(&pkey, &[guid])
        .membership(Membership::Full)
        .send()
        .await?;

    Ok(())
}
```

## API Overview

### Client Configuration

```rust
let client = UfmClient::builder()
    .base_url("https://ufm-server")
    .auth(Auth::basic("user", "pass"))  // or Auth::token("...")
    .timeout(Duration::from_secs(60))
    .max_retries(3)
    .verify_ssl(false)
    .build()?;
```

### Partition Keys (PKeys)

| Method | Description |
|--------|-------------|
| `pkeys().list()` | List all partition keys |
| `pkeys().get(&pkey)` | Get partition details |
| `pkeys().get_all()` | Get all partitions with full details |
| `pkeys().create(&pkey)` | Create a new partition (builder) |
| `pkeys().add_guids(&pkey, &guids)` | Add GUIDs to partition (builder) |
| `pkeys().remove_guids(&pkey, &guids)` | Remove GUIDs from partition |
| `pkeys().delete(&pkey)` | Delete a partition |
| `pkeys().set_qos(&pkey, &qos)` | Update QoS configuration |

### Virtual Ports (VPorts)

| Method | Description |
|--------|-------------|
| `vports().list()` | List all virtual ports |
| `vports().is_virtualization_enabled()` | Check virtualization status |
| `vports().by_system_guid(guid)` | Filter vports by system |

## Examples

Run the included examples:

```bash
# Set environment variables
export UFM_HOST=ufm-server
export UFM_USER=admin
export UFM_PASSWORD=secret

# Run examples
cargo run --example status
cargo run --example create_partition
cargo run --example add_guids_to_partition
cargo run --example view_partition
cargo run --example delete_partition
```

## Error Handling

All operations return `Result<T, ufm_client::Error>`. Error types include:

- `Error::Authentication` — Auth failures
- `Error::Api` — UFM API errors with status code
- `Error::NotFound` — Resource not found
- `Error::Timeout` — Request timeout
- `Error::RateLimited` — Rate limit exceeded (retryable)

```rust
use ufm_client::Error;

match client.pkeys().get(&pkey).await {
    Ok(details) => println!("{:?}", details),
    Err(Error::NotFound { .. }) => println!("PKey not found"),
    Err(e) if e.is_retryable() => println!("Transient error, retry later"),
    Err(e) => return Err(e.into()),
}
```

## Further Reading

- [UFM REST API Documentation](https://docs.nvidia.com/networking/display/ufmsdnapplianceumv625) — Official UFM API reference
- [API module docs](src/api/) — Detailed Rust API documentation
- [Examples](examples/) — Complete working examples

## License

Licensed under MIT OR Apache-2.0.

