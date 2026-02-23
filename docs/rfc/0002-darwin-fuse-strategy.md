# RFC 0002: Darwin FUSE Strategy — macFUSE Alternatives and Privilege Escalation

**Status**: Draft
**Author**: xoxd
**Date**: 2026-02-22
**Tracking**: Sprint 7
**Signed-off-by**: xoxd

---

## Abstract

This RFC addresses the macOS-specific challenges of mounting FUSE filesystems for
tcfs on Darwin. The current dependency on macFUSE introduces friction: it requires
a third-party kernel extension (or system extension on Apple Silicon), manual
approval in System Settings, and elevated privileges for mount operations. On
managed machines with ABR (Admin By Request), additional automation is needed.

We evaluate three paths forward:

1. **Status quo**: macFUSE with automated bootstrapping and extension approval
2. **Native reimplementation**: Pure Rust (or Zig) FUSE userspace via macOS NFS
   loopback or FSKit (macOS 15+)
3. **Hybrid**: macFUSE for existing macOS versions, FSKit for macOS 15+

## Motivation

During the Sprint 6 fleet deployment, tcfsd successfully connected to SeaweedFS
on all Darwin machines but failed to mount the FUSE filesystem:

| Host | macOS Version | Error | Root Cause |
|------|---------------|-------|------------|
| xoxd-bates | Sequoia 15.3 | `Read-only file system (os error 30)` | macFUSE system extension not approved |
| petting-zoo-mini | Sequoia 15.3 | `Permission denied (os error 13)` | macFUSE system extension not loaded |

The daemon's non-FUSE functionality works perfectly: S3 storage, gRPC socket,
metrics endpoint, device identity, and fleet sync all operate without FUSE. Only
the local mount point (which provides transparent file access via Finder/CLI)
requires FUSE.

### Why This Is Non-Trivial on macOS

1. **Sealed System Volume**: macOS 13+ enforces a read-only system volume. FUSE
   mounts must target the data volume (`/System/Volumes/Data`) or user-writable
   paths like `~/tcfs`.

2. **System Extension Approval**: macFUSE 4.x uses a system extension instead of
   a kernel extension. This requires:
   - User navigates to System Settings > Privacy & Security
   - Clicks "Allow" for the macFUSE extension
   - Reboots the machine
   - On Apple Silicon: may require reduced security in Recovery Mode

3. **Gatekeeper & Notarization**: macFUSE is notarized by its developer
   (Benjamin Fleischer), but custom FUSE implementations need their own Apple
   Developer ID and notarization workflow.

4. **Admin By Request (ABR)**: Managed machines in enterprise/education
   environments use ABR to gate sudo access. FUSE mount operations that require
   root (or the macFUSE helper) need ABR elevation, which is time-limited and
   requires user approval via a native dialog.

## Option 1: macFUSE with Automated Bootstrapping

### Approach

Keep macFUSE as the FUSE provider. Automate as much of the setup as possible via
the Nix home-manager module and launchd agents.

### Implementation

#### 1.1 Installation Detection & Auto-Install

```nix
# In tummycrypt.nix Darwin activation
home.activation.ensureMacFuse = lib.hm.dag.entryAfter [ "writeBoundary" ] ''
  if ! [ -d "/Library/Filesystems/macfuse.fs" ]; then
    echo "macFUSE not installed. Installing via Homebrew..."
    if command -v brew &>/dev/null; then
      $DRY_RUN_CMD brew install --cask macfuse
    else
      echo "ERROR: Homebrew not available. Install macFUSE manually:"
      echo "  brew install --cask macfuse"
      exit 1
    fi
  fi
'';
```

#### 1.2 System Extension Approval Automation

macOS does not provide a programmatic API to approve system extensions. However:

**Option A: MDM Profile (Recommended for managed fleets)**

Deploy a configuration profile that pre-approves the macFUSE system extension:

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "...">
<plist version="1.0">
<dict>
    <key>PayloadContent</key>
    <array>
        <dict>
            <key>PayloadType</key>
            <string>com.apple.system-extension-policy</string>
            <key>AllowedSystemExtensions</key>
            <dict>
                <key>9T634XKSRH</key>  <!-- macFUSE developer team ID -->
                <array>
                    <string>io.macfuse.filesystems.macfuse</string>
                </array>
            </dict>
        </dict>
    </array>
</dict>
</plist>
```

This requires an MDM server (Mosyle, Jamf, Fleet). For lab machines without MDM,
this is not viable unless we self-host a minimal MDM.

**Option B: osascript/AppleScript Guided Approval**

Cannot auto-approve, but can guide the user:

```bash
osascript -e 'display dialog "macFUSE needs approval.\n\nSystem Settings > Privacy & Security > Allow" \
  with title "tcfs Setup" buttons {"Open Settings", "Later"} default button 1'
if [ "$?" = "0" ]; then
  open "x-apple.systempreferences:com.apple.preference.security"
fi
```

**Option C: Reduced Security Mode (Apple Silicon)**

For headless machines (petting-zoo-mini), system extensions require booting into
Recovery Mode and running:

```bash
csrutil enable --without kext  # Or: bputil -d  (Reduced Security)
```

This is a one-time manual step per machine.

#### 1.3 ABR (Admin By Request) Integration

ABR gates sudo access behind a native approval dialog. When tcfs needs elevated
privileges (mount operations, extension loading), we need to detect and integrate
with ABR.

**Detection:**

```bash
# Check if ABR is installed
abr_installed() {
  [ -f "/Library/Application Support/AdminByRequest/AdminByRequest" ] || \
  [ -d "/Applications/Admin By Request.app" ]
}

# Check if ABR elevation is active (user has temporary admin)
abr_elevated() {
  # ABR adds user to admin group temporarily
  groups | grep -q admin
}
```

**Elevation Flow:**

```bash
if abr_installed && ! abr_elevated; then
  # Prompt user to request ABR elevation
  osascript -e 'display dialog "tcfs needs administrator access for FUSE mount.\n\n\
Please request Admin By Request elevation, then retry." \
    with title "tcfs - Admin Required" \
    buttons {"Request Admin", "Cancel"} default button 1'

  if [ "$?" = "0" ]; then
    # Open ABR elevation request
    open -a "Admin By Request"
    echo "Waiting for ABR elevation..."
    # Poll for elevation (max 5 minutes)
    for i in $(seq 1 30); do
      if abr_elevated; then
        echo "ABR elevation granted. Proceeding with mount."
        break
      fi
      sleep 10
    done
  fi
fi
```

**Launchd Integration:**

The tcfsd launchd agent should detect ABR and defer mount operations until
elevation is available:

```nix
# In daemon wrapper
if [ "$(uname)" = "Darwin" ] && abr_installed && ! abr_elevated; then
  echo "ABR detected, no elevation. Running in degraded mode (no FUSE mount)."
  exec ${tcfsdBin} --config "${configPath}" --no-mount
fi
```

### Pros

- Mature, well-tested (macFUSE has been around since 2006 as MacFuse/OSXFUSE)
- Supports all macOS versions back to 10.15
- Large user base (Docker Desktop, Cryptomator, VeraCrypt all use macFUSE)
- No additional code to maintain

### Cons

- Third-party dependency with unclear long-term maintenance
- System extension approval is manual and per-machine
- Apple Silicon requires reduced security mode for some configurations
- Notarization tied to macFUSE developer, not our keys
- ABR integration is polling-based, not event-driven
- Each macOS major release risks breaking macFUSE

## Option 2: Native FUSE Reimplementation in Rust

### Approach

Eliminate macFUSE entirely by implementing FUSE-like functionality using macOS
native APIs. Two sub-options exist:

#### 2A: NFS Loopback Mount

Mount a local NFS server that translates filesystem operations to tcfs operations.
No kernel extension needed.

**Architecture:**

```
Finder / CLI
    |
    v
NFS Client (built into macOS kernel)
    |
    v (localhost:2049)
tcfs-nfsd (Rust userspace NFS server)
    |
    v
tcfs storage layer (S3/SeaweedFS)
```

**Implementation:**

```rust
// Using nfs-server-rs or custom NFSv3/v4 implementation
use nfs_server::{NfsServer, FileSystem};

struct TcfsNfs {
    storage: Arc<StorageOperator>,
    cache: Arc<CacheManager>,
}

impl FileSystem for TcfsNfs {
    fn read(&self, path: &Path, offset: u64, size: u32) -> Result<Vec<u8>> {
        // Hydrate from SeaweedFS on demand
        self.storage.hydrate_and_read(path, offset, size)
    }

    fn write(&self, path: &Path, offset: u64, data: &[u8]) -> Result<u32> {
        // Write to local cache, queue sync
        self.cache.write_through(path, offset, data)
    }

    fn readdir(&self, path: &Path) -> Result<Vec<DirEntry>> {
        // Return .tc stubs + hydrated files
        self.storage.list_with_stubs(path)
    }
}
```

**Mount via launchd:**

```bash
# No macFUSE needed — just mount NFS
mkdir -p ~/tcfs
mount -t nfs -o tcp,resvport,locallocks localhost:/tcfs ~/tcfs
```

**Pros:**

- Zero third-party dependencies (NFS is built into macOS kernel)
- No system extension approval needed
- No reduced security mode needed
- Works on all macOS versions (NFS is ancient and stable)
- Can be notarized with our own Developer ID

**Cons:**

- NFS semantics differ from POSIX (stale caches, attribute caching)
- NFSv3 has 4GB file size limits on some configurations
- NFS loopback has overhead vs direct FUSE
- Must implement full NFS server (complex protocol)
- NFS port (2049) may require root or port >= 1024 workaround
- File change notifications (FSEvents) don't work over NFS

#### 2B: FSKit (macOS 15+ / Sequoia)

Apple introduced FSKit in macOS 15 as the official replacement for kernel-based
filesystems. It runs entirely in userspace with no kernel extension.

**Architecture:**

```
Finder / CLI
    |
    v
VFS Layer (macOS kernel)
    |
    v (XPC)
tcfs-fskit (FSKit Extension, Rust + Swift bridge)
    |
    v
tcfs storage layer (S3/SeaweedFS)
```

**Implementation:**

FSKit requires a System Extension bundle (.systemextension) distributed inside an
app bundle. The extension implements the `FSBlockDeviceFileSystem` or
`FSUnaryFileSystem` protocol.

```swift
// Swift bridge (FSKit requires Objective-C/Swift entry points)
import FSKit

@objc class TcfsFileSystem: FSUnaryFileSystem {
    let rustCore = TcfsRustBridge()

    override func mount(options: FSMountOptions) async throws -> FSVolume {
        try await rustCore.mount(options)
    }

    override func read(volume: FSVolume, node: FSNode,
                       offset: UInt64, length: UInt32) async throws -> Data {
        try await rustCore.read(node.path, offset, length)
    }
}
```

```rust
// Rust core (called via C FFI from Swift)
#[no_mangle]
pub extern "C" fn tcfs_read(path: *const c_char, offset: u64, len: u32,
                             buf: *mut u8) -> i32 {
    // ... hydration logic
}
```

**Code Signing & Notarization:**

FSKit extensions require:

1. **Apple Developer ID** ($99/year)
   - Needed for: Developer ID Application certificate, notarization
   - Entitlements: `com.apple.developer.fs-kit.user-space-driver`

2. **Provisioning Profile**
   - Must request `com.apple.developer.fs-kit.user-space-driver` entitlement
     from Apple (may require justification)

3. **Notarization**
   - All code must be notarized via `notarytool`
   - Includes the .systemextension bundle, the host app, and any helper tools

4. **Distribution**
   - Can be distributed outside App Store (Developer ID signed + notarized)
   - App bundle structure required:
     ```
     TummyCrypt.app/
       Contents/
         Library/
           SystemExtensions/
             dev.tinyland.tcfs-fskit.systemextension/
         MacOS/
           tcfs-helper  # Host app that manages the extension lifecycle
     ```

**Pros:**

- Apple-sanctioned API (future-proof)
- No kernel extension, no reduced security mode
- Full POSIX semantics (unlike NFS loopback)
- FSEvents / Spotlight integration possible
- Our own Developer ID (full control over signing)

**Cons:**

- macOS 15+ only (drops support for Ventura, Sonoma)
- FSKit API is new and sparsely documented (as of 2026)
- Requires Swift bridge layer (Rust cannot directly implement ObjC protocols)
- Apple Developer ID enrollment ($99/year) + entitlement approval
- .systemextension still requires user approval in System Settings (but lighter
  than kernel extensions)
- Test surface is small — few FSKit filesystems exist in the wild

#### 2C: Zig FUSE Implementation

Similar to 2A/2B but using Zig instead of Rust for the native layer.

**Rationale for Zig:**

- C ABI compatibility (no FFI overhead for calling macOS APIs)
- Compiles to tiny binaries (important for system extension size limits)
- `@cImport` directly imports macOS system headers
- Zig's allocator model maps well to filesystem buffer management

**Implementation would mirror 2A or 2B** but with Zig replacing Rust for the
platform-specific layer. The core tcfs logic (storage, crypto, sync) would remain
in Rust with a C FFI boundary.

**Tradeoff:** Introduces a second systems language into the codebase. Only
justified if the platform layer is small and well-bounded.

## Option 3: Hybrid Strategy (Recommended)

### Approach

Use macFUSE for macOS 13-14 (Ventura/Sonoma) and FSKit for macOS 15+ (Sequoia).
Runtime detection selects the backend.

### Implementation

```rust
enum FuseBackend {
    MacFuse,    // macOS 13-14
    FSKit,      // macOS 15+
    NfsLoopback, // Fallback (no FUSE/FSKit available)
    None,       // Degraded mode (push/pull only, no mount)
}

fn detect_backend() -> FuseBackend {
    let version = macos_version();
    if version >= (15, 0) && fskit_entitlement_granted() {
        FuseBackend::FSKit
    } else if macfuse_installed() && macfuse_extension_approved() {
        FuseBackend::MacFuse
    } else if can_bind_nfs_port() {
        FuseBackend::NfsLoopback
    } else {
        FuseBackend::None  // Graceful degradation
    }
}
```

### Graceful Degradation Levels

| Level | Mount | Push/Pull | Sync | Status |
|-------|-------|-----------|------|--------|
| Full (FSKit/macFUSE) | Yes | Yes | Yes | Transparent file access |
| NFS Loopback | Yes (limited) | Yes | Yes | Mount works, no FSEvents |
| Degraded (no mount) | No | Yes | Yes | CLI-only file access |

### ABR Integration (All Levels)

```rust
/// Detect ABR and adapt privilege escalation strategy
fn escalation_strategy() -> EscalationStrategy {
    if cfg!(target_os = "macos") {
        if abr_detected() {
            EscalationStrategy::ABR {
                // Use osascript to prompt for ABR elevation
                prompt: "tcfs needs temporary admin for FUSE mount",
                // Poll for elevation with exponential backoff
                poll_interval: Duration::from_secs(10),
                max_wait: Duration::from_secs(300),
            }
        } else {
            EscalationStrategy::Standard {
                // Use AuthorizationServices for sudo-like elevation
                right: "dev.tinyland.tcfs.mount",
            }
        }
    } else {
        EscalationStrategy::None // Linux: no elevation needed for FUSE3
    }
}
```

### Developer Key Strategy

Regardless of which FUSE backend ships, we need Apple code signing for macOS
distribution:

1. **Enroll in Apple Developer Program** ($99/year, `developer.apple.com`)
   - Organization: Tinyland Inc.
   - Type: Developer ID (for distribution outside App Store)

2. **Certificates needed:**
   - `Developer ID Application` — signs the tcfs binaries and app bundle
   - `Developer ID Installer` — signs .pkg installers

3. **Entitlements to request:**
   - `com.apple.developer.fs-kit.user-space-driver` (for FSKit path)
   - `com.apple.security.cs.allow-unsigned-executable-memory` (if needed for
     Rust runtime)

4. **CI/CD Integration:**
   ```yaml
   # GitHub Actions: sign and notarize
   - name: Sign tcfs binaries
     run: |
       codesign --sign "Developer ID Application: Tinyland Inc (TEAMID)" \
         --options runtime \
         --entitlements entitlements.plist \
         target/release/tcfs

   - name: Notarize
     run: |
       xcrun notarytool submit tcfs.zip \
         --apple-id "$APPLE_ID" \
         --team-id "$TEAM_ID" \
         --password "$APP_SPECIFIC_PASSWORD" \
         --wait
   ```

## Recommendation

**Short-term (v0.4.x):** Option 1 — macFUSE with ABR detection and guided setup.
Ship `--no-mount` degraded mode as the default on Darwin until FUSE is confirmed
working. This unblocks the fleet immediately.

**Medium-term (v0.5.x):** Option 2A — NFS loopback as a zero-dependency fallback.
This gives us a mount point on every Mac without any third-party software or
system extension approval.

**Long-term (v1.0):** Option 2B — FSKit native filesystem. This is the
Apple-sanctioned path and will be the most robust option as macOS evolves. Requires
Apple Developer enrollment and entitlement approval.

## Decision Matrix

| Criteria | macFUSE (Opt 1) | NFS Loopback (2A) | FSKit (2B) | Hybrid (Opt 3) |
|----------|-----------------|--------------------|-----------|----|
| Time to ship | 1 week | 3-4 weeks | 8-12 weeks | 4-6 weeks |
| macOS version support | 10.15+ | 10.15+ | 15+ only | All |
| Third-party deps | macFUSE | None | None | macFUSE (old macOS) |
| Kernel extension | Yes (sysext) | No | No | Varies |
| Apple Developer ID | No | No | Yes ($99/yr) | Yes |
| ABR compatible | With integration | Natively (no root for NFS?) | With sysext approval | Best coverage |
| POSIX compliance | Full | Partial (NFS caching) | Full | Varies |
| Spotlight/FSEvents | Yes | No | Yes | Varies |
| Maintenance burden | Low (upstream) | Medium (NFS server) | High (new API, Swift) | Highest |

## Open Questions

1. **FSKit entitlement timeline**: How long does Apple take to approve
   `com.apple.developer.fs-kit.user-space-driver`? Is it automatic or review-gated?

2. **NFS loopback root requirement**: Does `mount -t nfs localhost:/path` require
   root on modern macOS? If yes, this negates the zero-privilege advantage.

3. **ABR API**: Does Admin By Request expose any programmatic API (beyond polling
   group membership) for detecting elevation state?

4. **Zig vs Rust for platform layer**: Is the C ABI advantage of Zig worth adding
   a second language? Could use `objc2` Rust crate for ObjC bridging instead.

5. **Apple Developer enrollment**: Is Tinyland Inc already enrolled? If not, who
   initiates enrollment and manages the certificates?

---

Signed-off-by: xoxd
