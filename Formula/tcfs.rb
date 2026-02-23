# Homebrew formula for tcfs
# To use: brew tap tinyland-inc/tap && brew install tcfs
#
# This template is used by CI to generate the versioned formula.
# Placeholders: 0.5.0, e3424cc02e80594c3d8634b9c1a5121b3212a4f3d1dd417583754376c83d3ab7, 87ad554aa60376822e7b124df40bd7ec24457328a07fb74b4f523846dbaf54c9,
#               c6ffc8d37949a45e42f20ce1241bf3229f67515e40d39b0bcf83c7d4117cad7e, 5069fbc10d8181633471c858b6965e6015a657f57ac3108390ec094a1bd6acfd

class Tcfs < Formula
  desc "FOSS self-hosted odrive replacement — FUSE-based, SeaweedFS-backed file sync"
  homepage "https://github.com/tinyland-inc/tummycrypt"
  version "0.5.0"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.5.0/tcfs-0.5.0-macos-aarch64.tar.gz"
      sha256 "e3424cc02e80594c3d8634b9c1a5121b3212a4f3d1dd417583754376c83d3ab7"
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.5.0/tcfs-0.5.0-macos-x86_64.tar.gz"
      sha256 "87ad554aa60376822e7b124df40bd7ec24457328a07fb74b4f523846dbaf54c9"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.5.0/tcfs-0.5.0-linux-aarch64.tar.gz"
      sha256 "5069fbc10d8181633471c858b6965e6015a657f57ac3108390ec094a1bd6acfd"
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.5.0/tcfs-0.5.0-linux-x86_64.tar.gz"
      sha256 "c6ffc8d37949a45e42f20ce1241bf3229f67515e40d39b0bcf83c7d4117cad7e"
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
