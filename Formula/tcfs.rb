# Homebrew formula for tcfs
# To use: brew tap tinyland-inc/tap && brew install tcfs
#
# This template is used by CI to generate the versioned formula.
# Placeholders: 0.2.1, 1b718c043a9fd76b5e09c8585036e7bc9f191758a8616bae34ae852d823b8c86, 547c00ecca0b0828622264b87ba8d32dc170a9a82d1e57244b835ba79819da84,
#               cc727d707c806975be43b8b008bd8939a19dceda8ca3d91f0be4d5c7f5b8f4dc, 8511ab3be812e626621617cd99d6f68c2b605f001e54dd7ded60f2457f14f7aa

class Tcfs < Formula
  desc "FOSS self-hosted odrive replacement — FUSE-based, SeaweedFS-backed file sync"
  homepage "https://github.com/tinyland-inc/tummycrypt"
  version "0.2.1"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.2.1/tcfs-0.2.1-macos-aarch64.tar.gz"
      sha256 "1b718c043a9fd76b5e09c8585036e7bc9f191758a8616bae34ae852d823b8c86"
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.2.1/tcfs-0.2.1-macos-x86_64.tar.gz"
      sha256 "547c00ecca0b0828622264b87ba8d32dc170a9a82d1e57244b835ba79819da84"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.2.1/tcfs-0.2.1-linux-aarch64.tar.gz"
      sha256 "8511ab3be812e626621617cd99d6f68c2b605f001e54dd7ded60f2457f14f7aa"
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.2.1/tcfs-0.2.1-linux-x86_64.tar.gz"
      sha256 "cc727d707c806975be43b8b008bd8939a19dceda8ca3d91f0be4d5c7f5b8f4dc"
    end
  end

  def install
    bin.install "tcfs"
    bin.install "tcfsd"
    bin.install "tcfs-tui"
  end

  service do
    run [opt_bin/"tcfsd", "--config", etc/"tcfs/config.toml"]
    keep_alive true
    log_path var/"log/tcfsd.log"
    error_log_path var/"log/tcfsd.log"
  end

  test do
    assert_match "tcfs", shell_output("#{bin}/tcfs --version")
  end
end
