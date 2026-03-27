# Homebrew formula for tcfs
# To use: brew tap tinyland-inc/tap && brew install tcfs
#
# This template is used by CI to generate the versioned formula.
# Placeholders: 0.10.0, 01e60e344f89e9b9747d83e69a11ad14c8e829e49309b4782b1fc8290f11e55a, e9fa781a698deb797d3163f673664113067eb22d0fc9599445dde620b205078c,
#               618a11fed6d1f8568e478deda8843e8f188356f295246f6733b366efdc4007eb, c9c745bf75cf146284a64fe6df8c0f8d81343ab0b2bfc5c1ba2e003caecd3f19

class Tcfs < Formula
  desc "FOSS self-hosted odrive replacement — FUSE-based, SeaweedFS-backed file sync"
  homepage "https://github.com/tinyland-inc/tummycrypt"
  version "0.10.0"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.10.0/tcfs-0.10.0-macos-aarch64.tar.gz"
      sha256 "01e60e344f89e9b9747d83e69a11ad14c8e829e49309b4782b1fc8290f11e55a"
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.10.0/tcfs-0.10.0-macos-x86_64.tar.gz"
      sha256 "e9fa781a698deb797d3163f673664113067eb22d0fc9599445dde620b205078c"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.10.0/tcfs-0.10.0-linux-aarch64.tar.gz"
      sha256 "c9c745bf75cf146284a64fe6df8c0f8d81343ab0b2bfc5c1ba2e003caecd3f19"
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.10.0/tcfs-0.10.0-linux-x86_64.tar.gz"
      sha256 "618a11fed6d1f8568e478deda8843e8f188356f295246f6733b366efdc4007eb"
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
