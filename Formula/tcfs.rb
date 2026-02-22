# Homebrew formula for tcfs
# To use: brew tap tinyland-inc/tap && brew install tcfs
#
# This template is used by CI to generate the versioned formula.
# Placeholders: 0.2.3, a7dffd0967708cb3ae44b8f557fef6b6a4b215399d283f7791dc013221a52eb4, 50fb82ca05d2b64abc75de0ad5c7acb8c2e9b8071a02a7148217778592fb43a4,
#               4d17ae04b600d7a5b09fdf4618c65a61d4c7f5ff6aa5fe804785a33da173a713, 1e2cc1e4d6a1af0704debd9d853b99b9760a48764db4bda10fb131632c85929e

class Tcfs < Formula
  desc "FOSS self-hosted odrive replacement — FUSE-based, SeaweedFS-backed file sync"
  homepage "https://github.com/tinyland-inc/tummycrypt"
  version "0.2.3"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.2.3/tcfs-0.2.3-macos-aarch64.tar.gz"
      sha256 "a7dffd0967708cb3ae44b8f557fef6b6a4b215399d283f7791dc013221a52eb4"
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.2.3/tcfs-0.2.3-macos-x86_64.tar.gz"
      sha256 "50fb82ca05d2b64abc75de0ad5c7acb8c2e9b8071a02a7148217778592fb43a4"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.2.3/tcfs-0.2.3-linux-aarch64.tar.gz"
      sha256 "1e2cc1e4d6a1af0704debd9d853b99b9760a48764db4bda10fb131632c85929e"
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.2.3/tcfs-0.2.3-linux-x86_64.tar.gz"
      sha256 "4d17ae04b600d7a5b09fdf4618c65a61d4c7f5ff6aa5fe804785a33da173a713"
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
