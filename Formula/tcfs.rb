# Homebrew formula for tcfs
# To use: brew tap tinyland-inc/tap && brew install tcfs
#
# This template is used by CI to generate the versioned formula.
# Placeholders: 0.2.4, 366e4c19ad38f78b0d5928a8165b88beda8f6e4564d15c123adc05e3046d63f2, 199bf3b72bf03465dae7d2b379b0a7158d8c6fa0ab4721c3c2a1cf2315f82cba,
#               708437c8f807cd487aa1ec669659ad0875a605da715a35a1df0c0325733aacbd, abd53c093f1d65fe7e6cfef99de09c9ad42462f258f866797b64cf50fce9845f

class Tcfs < Formula
  desc "FOSS self-hosted odrive replacement — FUSE-based, SeaweedFS-backed file sync"
  homepage "https://github.com/tinyland-inc/tummycrypt"
  version "0.2.4"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.2.4/tcfs-0.2.4-macos-aarch64.tar.gz"
      sha256 "366e4c19ad38f78b0d5928a8165b88beda8f6e4564d15c123adc05e3046d63f2"
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.2.4/tcfs-0.2.4-macos-x86_64.tar.gz"
      sha256 "199bf3b72bf03465dae7d2b379b0a7158d8c6fa0ab4721c3c2a1cf2315f82cba"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.2.4/tcfs-0.2.4-linux-aarch64.tar.gz"
      sha256 "abd53c093f1d65fe7e6cfef99de09c9ad42462f258f866797b64cf50fce9845f"
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.2.4/tcfs-0.2.4-linux-x86_64.tar.gz"
      sha256 "708437c8f807cd487aa1ec669659ad0875a605da715a35a1df0c0325733aacbd"
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
