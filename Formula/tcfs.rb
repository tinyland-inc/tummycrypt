# Homebrew formula for tcfs
# To use: brew tap tinyland-inc/tap && brew install tcfs
#
# This template is used by CI to generate the versioned formula.
# Placeholders: 0.9.2, 8501cd1c671862bd231c8ad2df5e5f58641eff9b3f0a9e7366a4f7313e9959b5, ,
#               c6bf64a012a48186c83438bc8ad9ec433fe638bb051f0dc239ac6f78d6dfd14a, 

class Tcfs < Formula
  desc "FOSS self-hosted odrive replacement — FUSE-based, SeaweedFS-backed file sync"
  homepage "https://github.com/tinyland-inc/tummycrypt"
  version "0.9.2"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.9.2/tcfs-0.9.2-macos-aarch64.tar.gz"
      sha256 "8501cd1c671862bd231c8ad2df5e5f58641eff9b3f0a9e7366a4f7313e9959b5"
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.9.2/tcfs-0.9.2-macos-x86_64.tar.gz"
      sha256 ""
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.9.2/tcfs-0.9.2-linux-aarch64.tar.gz"
      sha256 ""
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.9.2/tcfs-0.9.2-linux-x86_64.tar.gz"
      sha256 "c6bf64a012a48186c83438bc8ad9ec433fe638bb051f0dc239ac6f78d6dfd14a"
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
