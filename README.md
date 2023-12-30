<h1 align="center">Caracal</h1>

<p align="center">
    A file downloader written in
    <a href="https://www.rust-lang.org/" target="_blank">Rust Programming Language</a>.
</p>

<p align="center">
    <a href="https://github.com/xrelkd/caracal/releases"><img src="https://img.shields.io/github/v/release/xrelkd/caracal.svg"></a>
    <a href="https://github.com/xrelkd/caracal/actions?query=workflow%3ARust"><img src="https://github.com/xrelkd/caracal/workflows/Rust/badge.svg"></a>
    <a href="https://github.com/xrelkd/caracal/actions?query=workflow%3ARelease"><img src="https://github.com/xrelkd/caracal/workflows/Release/badge.svg"></a>
    <a href="https://github.com/xrelkd/caracal/blob/master/LICENSE"><img alt="GitHub License" src="https://img.shields.io/github/license/xrelkd/caracal"></a>
</p>

**[Installation](#installation) | [Usage](#usage) | [Configuration](#configuration)**

<details>
<summary>Table of contents</summary>

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [Configuration](#configuration)
- [License](#license)

</details>

## Features

- [x] Support downloading files from HTTP/HTTPs services
- [x] Support downloading files from SFTP services
- [x] Support downloading files from [MinIO](https://min.io/) services
- [x] Support parallel downloading to accelerate download speed
- [x] Support broken-point continuingly-transferring
- [x] Support daemonizing

## Installation

<details>
    <summary>Install the pre-built binaries</summary>

Pre-built binaries for Linux can be found on [the releases page](https://github.com/xrelkd/caracal/releases/), the latest release is available [here](https://github.com/xrelkd/caracal/releases/latest).

For example, to install `caracal` to `~/bin`:

```bash
# Create `~/bin`.
mkdir -p ~/bin

# Change directory to `~/bin`.
cd ~/bin

# Download and extract caracal to `~/bin/`.
# NOTE: replace the version with the version you want to install
export CARACAL_VERSION=v0.2.0

# NOTE: the architecture of your machine,
# Available values are `x86_64-unknown-linux-musl`, `x86_64-apple-darwin`, `aarch64-apple-darwin`.
export ARCH=x86_64-unknown-linux-musl
curl -s -L "https://github.com/xrelkd/caracal/releases/download/${CARACAL_VERSION}/caracal-${CARACAL_VERSION}-${ARCH}.tar.gz" | tar xzf -

# Add `~/bin` to the paths that your shell searches for executables
# this line should be added to your shells initialization file,
# e.g. `~/.bashrc` or `~/.zshrc`
export PATH="$PATH:$HOME/bin"

# Show version.
caracal version

# Show version.
caracal-daemon version
```

</details>

<details>
  <summary>Build from source</summary>

`caracal` requires the following tools and packages to build:

- `rustc`
- `cargo`
- `pkg-config`
- `libgit2`

With the above tools and packages already installed, you can simply run:

```bash
git clone --branch=main https://github.com/xrelkd/caracal.git
cd caracal

cargo install --path caracal
cargo install --path caracal-daemon
```

</details>

## Usage

### Standalone mode

Run `caracal` without `caracal-daemon`

```bash
# Show usage.
caracal help

# Download a file from HTTP server.
caracal https://www.rust-lang.org/

# Download multiple files from HTTP server.
caracal https://example.com/a.tar.gz https://example.com/b.zip

# Copy a file from local file system.
caracal /etc/os-release

# Download a file from SFTP server.
caracal sftp://my-ssh-server/etc/os-release

# Copy a file from MinIO server.
caracal minio://myminio/path/to/file

# Download multiple files from different services.
caracal \
    /etc/os-release \
    https://example.com/a.tar.gz \
    sftp://my-ssh-server/etc/os-release \
    minio://myminio/path/to/file

# Download multiple files from different services and put them in directory `/tmp/downloads`.
mkdir -p /tmp/downloads && \
    caracal -D /tmp/downloads \
        /etc/os-release \
        sftp://my-ssh-server/etc/os-release \
        minio://myminio/path/to/file \
        https://example.com/a.tar.gz

# Specify an alternative number of connections.
caracal -n 3 https://www.rust-lang.org/

# Set the connection timeout in second.
caracal -T 3 https://www.rust-lang.org/

```

### Daemon mode

Run `caracal-daemon` and use `caracal` to interact with `caracal-daemon`.

```bash
# Show usage.
caracal-daemon help

# Start daemon, caracal-daemon runs as a daemon.
# It provides gRPC endpoint, waiting for new commands.
caracal-daemon &

# Show usage of `add-uri`.
caracal help add-uri

# Use the subcommand `add-uri` to create new task.
# Add a new task for downloading a file from HTTP server.
caracal add-uri https://www.rust-lang.org/

# Show status of tasks.
caracal status

# Add a new task for downloading multiple files from HTTP server.
caracal add-uri https://example.com/a.tar.gz https://example.com/b.zip

# Add a new task for copying a file from local file system.
caracal add-uri /etc/os-release

# Add a new task for downloading a file from SFTP server.
caracal add-uri sftp://my-ssh-server/etc/os-release

# Add a new task for copying a file from MinIO server.
caracal add-uri minio://myminio/path/to/file

# Add a new task for downloading multiple files from different services.
caracal add-uri \
    /etc/os-release \
    https://example.com/a.tar.gz \
    sftp://my-ssh-server/etc/os-release \
    minio://myminio/path/to/file

# Download multiple files from different services and put them in directory `/tmp/downloads`.
mkdir -p /tmp/downloads && \
    caracal add-uri -D /tmp/downloads \
        /etc/os-release \
        sftp://my-ssh-server/etc/os-release \
        minio://myminio/path/to/file \
        https://example.com/a.tar.gz

# Pause tasks.
caracal pause 1 2 3

# Resume tasks.
caracal resume 1 2 3

# Pause all tasks.
caracal pause --all

# Resume all tasks.
caracal resume --all

# Remove tasks.
caracal remove 1 2 3
```

## Configuration

The configuration file of `caracal` is placed on `$XDG_CONFIG_HOME/caracal/caracal.toml`.

The configuration file of `caracal-daemon` is placed on `$XDG_CONFIG_HOME/caracal/caracal-daemon.toml`.

```bash
# Create directory to store configuration files.
mkdir -p $XDG_CONFIG_HOME/caracal/

# Generate default configuration and place it on `$XDG_CONFIG_HOME/caracal/caracal.toml`.
caracal default-config > $XDG_CONFIG_HOME/caracal/caracal.toml

# Generate default configuration and place it on `$XDG_CONFIG_HOME/caracal/caracal-daemon.toml`.
caracal-daemon default-config > $XDG_CONFIG_HOME/caracal/caracal-daemon.toml
```

<details>
<summary>Example of <b>$XDG_CONFIG_HOME/caracal/caracal.toml</b></summary>

```toml
# File paths to profiles, see profile file configuration
profile_files = ["/path/to/profile/file", "/path/to/profile/file2"]

[daemon]
# Endpoint of gRPC server
# Caracal connect to gRPC server via local socket with file path like "/path/to/caracal-daemon/grpc.sock"
# Caracal connect to gRPC server via HTTP with URI like "http://www.my.server.com/"
server_endpoint = "/path/to/caracal-daemon/grpc.sock"

[log]
# Emit log to systemd-journald
emit_journald = true
# Emit log to stdout
emit_stdout = false
# Emit log to stderr
emit_stderr = false
# Set the log level, available values are "ERROR", "WARN", "INFO", "DEBUG", "TRACE"
level = "INFO"

[downloader.http]
# The user-agent which will be passed to HTTP server
user_agent = "Caracal/0.2.0"
# The number of concurrent number of HTTP connection per task
concurrent_connections = 5
```

</details>

<details>
<summary>Example of <b>$XDG_CONFIG_HOME/caracal/caracal-daemon.toml</b></summary>

```toml
# File paths to profiles, see profile file configuration
profile_files = ["/path/to/profile/file", "/path/to/profile/file2"]

[log]
# Emit log to systemd-journald
emit_journald = true
# Emit log to stdout
emit_stdout = false
# Emit log to stderr
emit_stderr = false
# Set the log level, available values are "ERROR", "WARN", "INFO", "DEBUG", "TRACE"
level = "INFO"

[task_scheduler]
# The number of tasks to execute concurrently
concurrent_number = 10

[downloader.http]
# The user-agent which will be passed to HTTP server
user_agent = "Caracal/0.2.0"
# The number of concurrent number of HTTP connection per task
concurrent_connections = 5

[grpc]
# Provide gRPC via HTTP
enable_http = true
# Host address of gRPC, ignored while `enable_http` is `false`
host = "127.0.0.1"
# Port of gRPC server, ignored while `enable_http` is `false`
port = 37000
# Provide gRPC service via local socket (UNIX domain socket)
enable_local_socket = true
# Path of local socket
local_socket = "/path/to/caracal-daemon/grpc.sock"

[metrics]
# Enable Prometheus metrics
enable = true
# Host address of metrics
host = "127.0.0.1"
# Port of metrics
port = 37002
```

</details>

<details>
<summary>Example of <b>Profile</b> file</summary>

```toml
[[profiles]]
[profiles.MinIO]
# Name of profile
name         = "my-minio"
# Endpoint of MinIO server
endpoint_url = "https://my.minio.server.com"
# Access key of MinIO server
access_key   = "access_key"
# Secret key of MinIO server
secret_key   = "secret_key"

[[profiles]]
[profiles.MinIO]
name         = "my-minio2"
endpoint_url = "https://my.minio2.server.com"
access_key   = "access_key"
secret_key   = "secret_key"

[[profiles]]
[profiles.SSH]
# Name of profile
name          = "my-ssh-server"
# SSH host to connect
# It may be specified as either [user@]hostname or a URI of the form ssh://[user@]hostname[:port].
endpoint      = "my-ssh-server"
# Set the SSH user
user          = "user"
# Set the key file to use
identity_file = "/path/to/ssh/key"

[[profiles]]
[profiles.SSH]
name          = "my-ssh-server2"
endpoint      = "my-ssh-server2"
user          = "user"
identity_file = "/path/to/ssh/key2"
```

</details>

## License

Caracal is licensed under the GNU General Public License version 3. See [LICENSE](./LICENSE) for more information.
