# tcfs — TummyCrypt Filesystem

**FOSS self-hosted odrive replacement**

tcfs is a FUSE-based file sync daemon backed by [SeaweedFS](https://github.com/seaweedfs/seaweedfs) with end-to-end [age](https://age-encryption.org) encryption, content-defined chunking, and on-demand hydration via `.tc` stub files.

## Quick Install

### Linux / macOS

```bash
curl -fsSL https://github.com/tummycrypt/tummycrypt/releases/latest/download/install.sh | sh
```

### Homebrew

```bash
brew install tummycrypt/tap/tcfs
```

### Debian / Ubuntu

```bash
# Download the .deb from the latest release
sudo dpkg -i tcfs-*.deb
```

### RPM (Fedora / RHEL / Rocky)

```bash
sudo rpm -i tcfsd-*.rpm
```

### Nix

```bash
nix profile install github:tummycrypt/tummycrypt
```

### Container (K8s worker mode)

```bash
podman pull ghcr.io/tummycrypt/tcfsd:latest
```

## How It Works

1. **Push**: Files are split into content-defined chunks (FastCDC), compressed (zstd), encrypted (age), and uploaded to SeaweedFS via S3.
2. **Pull**: Manifests describe the chunk layout. Chunks are fetched, verified (BLAKE3), decrypted, decompressed, and reassembled.
3. **Mount**: FUSE driver presents remote files as local. Files appear as `.tc` stubs until opened — then they're hydrated on demand.
4. **Unsync**: Convert hydrated files back to stubs, reclaiming disk space while keeping the remote copy.

## Architecture

```
tcfs (CLI)  ───┐
tcfs-tui    ───┤  gRPC / Unix socket
               ├──── tcfsd (daemon) ──── FUSE mount
               │        │
               │        ├── tcfs-secrets (age/SOPS/KeePassXC)
               │        ├── tcfs-chunks  (FastCDC + zstd + BLAKE3)
               │        ├── tcfs-storage (OpenDAL → S3/SeaweedFS)
               │        └── tcfs-sync    (state cache + NATS JetStream)
               │
K8s workers ───┘  (tcfsd --mode=worker, scaled by KEDA)
```

## Binaries

| Binary | Purpose |
|--------|---------|
| `tcfs` | CLI: push, pull, sync-status, mount, unmount, unsync |
| `tcfsd` | Daemon: gRPC socket, FUSE mounts, Prometheus metrics, systemd notify |
| `tcfs-tui` | Terminal UI for interactive file management |

## Documentation

- [Architecture](ARCHITECTURE.md) — detailed system design
- [Protocol](PROTOCOL.md) — wire format, chunk layout, manifest schema
- [Security](SECURITY.md) — threat model, encryption details
- [Contributing](CONTRIBUTING.md) — development setup, PR workflow
- [Benchmarks](BENCHMARKS.md) — performance characteristics

## Platform Support

| Platform | Status | Notes |
|----------|--------|-------|
| Linux x86_64 | Full | FUSE mount, CLI, daemon, TUI |
| Linux aarch64 | Full | FUSE mount, CLI, daemon, TUI |
| macOS (Apple Silicon) | CLI only | FUSE via FUSE-T planned |
| macOS (Intel) | CLI only | FUSE via FUSE-T planned |
| Windows | Planned | Cloud Files API (CFAPI) for native Explorer integration |
| NixOS | Full | Flake + NixOS module + Home Manager module |

## License

Dual-licensed under MIT and Apache 2.0.
