# Homebrew formula for tcfs
# To use: brew tap tinyland-inc/tap && brew install tcfs
#
# This template is used by CI to generate the versioned formula.
# Placeholders: 0.9.1, a88f640872eb30a7a480fc9402b3d8c851a70672472b0f3017b8d8b3e51b5ff8, ,
#               4dd3b8ae75ddb69947922c821e03635a151cd274731233b35509e02e0b87b621, 

class Tcfs < Formula
  desc "FOSS self-hosted odrive replacement — FUSE-based, SeaweedFS-backed file sync"
  homepage "https://github.com/tinyland-inc/tummycrypt"
  version "0.9.1"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.9.1/tcfs-0.9.1-macos-aarch64.tar.gz"
      sha256 "a88f640872eb30a7a480fc9402b3d8c851a70672472b0f3017b8d8b3e51b5ff8"
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.9.1/tcfs-0.9.1-macos-x86_64.tar.gz"
      sha256 ""
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.9.1/tcfs-0.9.1-linux-aarch64.tar.gz"
      sha256 ""
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.9.1/tcfs-0.9.1-linux-x86_64.tar.gz"
      sha256 "4dd3b8ae75ddb69947922c821e03635a151cd274731233b35509e02e0b87b621"
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
