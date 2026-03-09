# Homebrew formula for tcfs
# To use: brew tap tinyland-inc/tap && brew install tcfs
#
# This template is used by CI to generate the versioned formula.
# Placeholders: 0.9.0, 69c9f18337b5eb784c4fcbf145b584eaa6d8e76d5b1e18b96e8c67e4ca71d397, ,
#               a07aabc1a7714437691d55b258cfbc9bc39e55bb676371cfe6105ea1102a132f, 

class Tcfs < Formula
  desc "FOSS self-hosted odrive replacement — FUSE-based, SeaweedFS-backed file sync"
  homepage "https://github.com/tinyland-inc/tummycrypt"
  version "0.9.0"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.9.0/tcfs-0.9.0-macos-aarch64.tar.gz"
      sha256 "69c9f18337b5eb784c4fcbf145b584eaa6d8e76d5b1e18b96e8c67e4ca71d397"
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.9.0/tcfs-0.9.0-macos-x86_64.tar.gz"
      sha256 ""
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.9.0/tcfs-0.9.0-linux-aarch64.tar.gz"
      sha256 ""
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.9.0/tcfs-0.9.0-linux-x86_64.tar.gz"
      sha256 "a07aabc1a7714437691d55b258cfbc9bc39e55bb676371cfe6105ea1102a132f"
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
