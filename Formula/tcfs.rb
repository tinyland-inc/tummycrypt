# Homebrew formula for tcfs
# To use: brew tap tinyland-inc/tap && brew install tcfs
#
# This template is used by CI to generate the versioned formula.
# Placeholders: 0.2.5, cfa238162cd8e7b581cd2654276a5b0f6618cd917c9df4b4072687fb0249197d, 070f575fd0523f7c73d5f1bbdbbacf8d160fc4506f70d354f5001150e9708c0b,
#               1e39d8a086b8636b17d80b9e35ac9b25afdcac70ed98b522e84cf6196abdd668, 2f327f3847f9f91a0812b312f930b796d39dde0079a7d9f0a9eaccb687a192a3

class Tcfs < Formula
  desc "FOSS self-hosted odrive replacement — FUSE-based, SeaweedFS-backed file sync"
  homepage "https://github.com/tinyland-inc/tummycrypt"
  version "0.2.5"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.2.5/tcfs-0.2.5-macos-aarch64.tar.gz"
      sha256 "cfa238162cd8e7b581cd2654276a5b0f6618cd917c9df4b4072687fb0249197d"
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.2.5/tcfs-0.2.5-macos-x86_64.tar.gz"
      sha256 "070f575fd0523f7c73d5f1bbdbbacf8d160fc4506f70d354f5001150e9708c0b"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.2.5/tcfs-0.2.5-linux-aarch64.tar.gz"
      sha256 "2f327f3847f9f91a0812b312f930b796d39dde0079a7d9f0a9eaccb687a192a3"
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.2.5/tcfs-0.2.5-linux-x86_64.tar.gz"
      sha256 "1e39d8a086b8636b17d80b9e35ac9b25afdcac70ed98b522e84cf6196abdd668"
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
