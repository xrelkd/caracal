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

**[Installation](#installation) | [Usage](#usage)**

<details>
<summary>Table of contents</summary>

- [Features](#features)
- [Installation](#installation)
- [Usage](#usage)
- [License](#license)

</details>

## Features

- [x] Support HTTP/HTTPs
- [x] Support SFTP
- [x] Support MinIO

## Installation

<details>
    <summary>Install the pre-built binaries</summary>

Pre-built binaries for Linux can be found on [the releases page](https://github.com/xrelkd/caracal/releases/), the latest release is available [here](https://github.com/xrelkd/caracal/releases/latest).

For example, to install `caracal` to `~/bin`:

```bash
# create ~/bin
mkdir -p ~/bin

# change directory to ~/bin
cd ~/bin

# download and extract caracal to ~/bin/
# NOTE: replace the version with the version you want to install
export CARACAL_VERSION=v0.1.0

# NOTE: the architecture of your machine,
# available values are `x86_64-unknown-linux-musl`, `x86_64-apple-darwin`, `aarch64-apple-darwin`
export ARCH=x86_64-unknown-linux-musl
curl -s -L "https://github.com/xrelkd/caracal/releases/download/${CARACAL_VERSION}/caracal-${CARACAL_VERSION}-${ARCH}.tar.gz" | tar xzf -

# add `~/bin` to the paths that your shell searches for executables
# this line should be added to your shells initialization file,
# e.g. `~/.bashrc` or `~/.zshrc`
export PATH="$PATH:$HOME/bin"

# show version info
caracal version
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
```

</details>

## Usage

```bash
# show usage
caracal help

# download a file from HTTP server
caracal https://www.rust-lang.org/

# download multiple files from HTTP server
caracal https://example.com/a.tar.gz https://example.com/b.zip

# copy a file from local file system
caracal /etc/os-release

# download a file from SFTP server
caracal sftp://my-ssh-server/etc/os-release

# copy a file from MinIO server
caracal minio://myminio/path/to/file

# download multiple files from different services
caracal \
    /etc/os-release \
    https://example.com/a.tar.gz \
    sftp://my-ssh-server/etc/os-release \
    minio://myminio/path/to/file

# download multiple files from different services and put them in directory /tmp/downloads
mkdir -p /tmp/downloads && \
    caracal -D /tmp/downloads \
        /etc/os-release \
        sftp://my-ssh-server/etc/os-release \
        minio://myminio/path/to/file \
        https://example.com/a.tar.gz
```

## License

Caracal is licensed under the GNU General Public License version 3. See [LICENSE](./LICENSE) for more information.
