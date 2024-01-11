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

**[Installation](#installation) | [Usage](#usage) | [Configuration](#configuration)| [Container](#container)**

<details>
<summary>Table of contents</summary>

- [Features](#features)
- [Screenshots](#screenshots)
- [Installation](#installation)
- [Usage](#usage)
- [Configuration](#configuration)
- [Container](#container)
- [License](#license)

</details>

## Features

- [x] Support downloading files from HTTP/HTTPs services
- [x] Support downloading files from SFTP services
- [x] Support downloading files from [MinIO](https://min.io/) services
- [x] Support parallel downloading to accelerate download speed
- [x] Support broken-point continuingly-transferring
- [x] Support daemonizing
- [x] Provide terminal user interface (TUI)

## Screenshots

- Terminal user interface (TUI)

![screenshot tui](docs/_static/screenshot-tui.png)

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
export CARACAL_VERSION=v0.3.1

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

# Show version.
caracal-tui version
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
cargo install --path caracal-tui
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

### Terminal user interface (TUI)

- **NOTE**: Remember to start `caracal-daemon`. Terminal user interface does not provide standalone mode, it provides user interface for user to interact with `caracal-daemon`.

```bash
# Start `caracal-daemon` and put it in the background.
caracal-daemon &

# Show version.
caracal-tui version

# Start terminal user interface.
caracal-tui
```

## Configuration

The configuration file of `caracal` is placed on `$XDG_CONFIG_HOME/caracal/caracal.toml`.

The configuration file of `caracal-daemon` is placed on `$XDG_CONFIG_HOME/caracal/caracal-daemon.toml`.

The configuration file of `caracal-tui` is placed on `$XDG_CONFIG_HOME/caracal/caracal-tui.toml`.

```bash
# Create directory to store configuration files.
mkdir -p $XDG_CONFIG_HOME/caracal/

# Generate default configuration and place it on `$XDG_CONFIG_HOME/caracal/caracal.toml`.
caracal default-config > $XDG_CONFIG_HOME/caracal/caracal.toml

# Generate default configuration and place it on `$XDG_CONFIG_HOME/caracal/caracal-daemon.toml`.
caracal-daemon default-config > $XDG_CONFIG_HOME/caracal/caracal-daemon.toml

# Generate default configuration and place it on `$XDG_CONFIG_HOME/caracal/caracal-tui.toml`.
caracal-tui default-config > $XDG_CONFIG_HOME/caracal/caracal-tui.toml
```

<details>
<summary>Example of <b>$XDG_CONFIG_HOME/caracal/caracal.toml</b></summary>

**NOTE**: `~` in a file path will be resolved to `$HOME`.

```toml
# File paths to profiles, see profile file configuration
profile_files = ["/path/to/profile/file", "/path/to/profile/file2", "~/path/to/my/profile"]

[daemon]
# Endpoint of gRPC server
# Caracal connect to gRPC server via local socket with file path like "/path/to/caracal-daemon/grpc.sock"
# Caracal connect to gRPC server via HTTP with URI like "http://www.my.server.com/"
server_endpoint = "/path/to/caracal-daemon/grpc.sock"
# Access token, remove this line to disable authentication
access_token    = "my-access-token"
# File path of access token, remove this line to disable authentication
# `access_token_file_path` is preferred if both `access_token` and `access_token_file_path` are provided.
access_token_file_path = "/path/to/access-token"

[log]
# Emit log to systemd-journald
emit_journald = true
# Emit log to stdout
emit_stdout = false
# Emit log to stderr
emit_stderr = false
# Set the log level, available values are "ERROR", "WARN", "INFO", "DEBUG", "TRACE"
level = "INFO"

[downloader]
# Path of default output directory
default_output_directory = "/path/to/default/output/directory"

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
# Access token, remove this line to disable authentication
access_token    = "my-access-token"
# File path of access token, remove this line to disable authentication
# `access_token_file_path` is preferred if both `access_token` and `access_token_file_path` are provided.
access_token_file_path = "/path/to/access-token"

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
<summary>Example of <b>$XDG_CONFIG_HOME/caracal/caracal-tui.toml</b></summary>

**NOTE**: `~` in a file path will be resolved to `$HOME`.

```toml
[daemon]
# Endpoint of gRPC server
# Caracal connect to gRPC server via local socket with file path like "/path/to/caracal-daemon/grpc.sock"
# Caracal connect to gRPC server via HTTP with URI like "http://www.my.server.com/"
server_endpoint = "/path/to/caracal-daemon/grpc.sock"
# Access token, remove this line to disable authentication
access_token    = "my-access-token"
# File path of access token, remove this line to disable authentication
# `access_token_file_path` is preferred if both `access_token` and `access_token_file_path` are provided.
access_token_file_path = "/path/to/access-token"

[log]
# Emit log to systemd-journald
emit_journald = true
# Emit log to stdout
emit_stdout = false
# Emit log to stderr
emit_stderr = false
# Set the log level, available values are "ERROR", "WARN", "INFO", "DEBUG", "TRACE"
level = "INFO"
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

## Container

Container images are available on [GitHub Packages](https://github.com/xrelkd/caracal/pkgs/container/caracal).

- Run `caracal-daemon` with `Docker`

```bash
docker pull ghcr.io/xrelkd/caracal:latest
docker run -d ghcr.io/xrelkd/caracal:latest
```

<details>
<summary>Run with <b>Docker Compose</b></summary>

We use `Docker Compose` to configurate `caracal-daemon` service.

1. Create `docker-compose.yaml` and `caracal-daemon.toml` with the following contents.

- `docker-compose.yaml`

```yaml
services:
  caracal:
    image: ghcr.io/xrelkd/caracal:latest
    ports:
      - "127.0.0.1:37000:37000"
      - "127.0.0.1:37002:37002"
    volumes:
      - ${PWD}/caracal-daemon.toml:/etc/caracal/caracal-daemon.toml
      - downloads:/downloads
    entrypoint: ["caracal-daemon", "--config=/etc/caracal/caracal-daemon.toml"]

volumes:
  downloads:
```

- `caracal-daemon.toml`

```toml
profile_files = []

[log]
# systemd-journald is not available in container, disable it
emit_journald = false
# Emit log message to stdout.
emit_stdout   = true
emit_stderr   = false
level         = "INFO"

[task_scheduler]
concurrent_number = 10

[downloader]
# Set default output directory to `/downloads`.
default_output_directory = "/downloads"

[downloader.http]
user_agent             = "Caracal/0.2.0"
concurrent_connections = 5

[grpc]
enable_http         = true
# Disable local socket because we only interact with the daemon via HTTP.
enable_local_socket = false
host                = "0.0.0.0"
port                = 37000

[metrics]
enable = true
host   = "0.0.0.0"
port   = 37002
```

**NOTE**: In order to connect the `caracal-daemon` in container, `daemon.server_endpoint` in `caracal.toml` should be set as `http://127.0.0.1:37000`.

2. Run `docker compose up` to start the container.
3. Run `caracal add-uri https://www.rust-lang.org/` to create a new task, the downloaded file is placed on `/downloads` in the container.
4. Run `caracal status` to display the status of tasks.
5. Run `docker compose down` to stop the container.

</details>

## Kubernetes

<details>
<summary>Deploy on <b>Kubernetes</b></summary>

Save the following contents to `caracal.yaml` and execute `kubectl apply -f caracal.yaml` to deploy `caracal-daemon` on Kubernetes cluster:

```yaml
# https://kubernetes.io/docs/concepts/configuration/configmap/
kind: ConfigMap
apiVersion: v1
metadata:
  name: caracal

data:
  caracal-daemon.toml: |
    profile_files = []

    [log]
    emit_journald = false
    emit_stdout = true
    emit_stderr = false
    level = "INFO"

    [task_scheduler]
    concurrent_number = 10

    [downloader]
    default_output_directory = "/tmp"

    [downloader.http]
    user_agent = "Caracal/0.2.0"
    concurrent_connections = 5

    [grpc]
    enable_http = true
    enable_local_socket = false
    host = "0.0.0.0"
    port = 37000

    [metrics]
    enable = true
    host = "0.0.0.0"
    port = 37002

---
# https://kubernetes.io/docs/concepts/workloads/controllers/deployment/
apiVersion: apps/v1
kind: Deployment
metadata:
  name: caracal
  labels:
    app.kubernetes.io/name: caracal
spec:
  selector:
    matchLabels:
      app: caracal
  replicas: 1
  template:
    metadata:
      labels:
        app: caracal
    spec:
      restartPolicy: Always
      volumes:
        - name: config
          configMap:
            name: caracal
            items:
              - key: caracal-daemon.toml
                path: caracal-daemon.toml
      containers:
        - name: caracal
          image: ghcr.io/xrelkd/caracal:latest
          imagePullPolicy: IfNotPresent
          command:
            - "caracal-daemon"
            - "--config=/etc/caracal/caracal-daemon.toml"
          volumeMounts:
            - name: config
              mountPath: /etc/caracal/
          ports:
            - containerPort: 37000
              name: grpc
            - containerPort: 37002
              name: metrics
---
# https://kubernetes.io/docs/concepts/services-networking/service/
apiVersion: v1
kind: Service
metadata:
  name: caracal

spec:
  selector:
    app: caracal
  type: ClusterIP
  ports:
    - name: grpc
      protocol: TCP
      port: 37000
      targetPort: 37000
---
```

</details>

## License

Caracal is licensed under the GNU General Public License version 3. See [LICENSE](./LICENSE) for more information.
