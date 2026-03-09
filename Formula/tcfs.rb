# Homebrew formula for tcfs
# To use: brew tap tinyland-inc/tap && brew install tcfs
#
# This template is used by CI to generate the versioned formula.
# Placeholders: 0.9.3, d13aeb88292a844c518b652c8a0c9c66319bba7f1741093570acf12b8b58bf50, b9b19921e4906d756d4a31144413a3cd0ea9df4a5aa3e15773c4ca1f4e7282c9,
#               4232fb577008b498d05e98457b63f0ff6343f7860831baae661989347b960a30, 46f828107a3604e696f8b6ccef1003f6254baae6eae6105a02e3a875083099bf

class Tcfs < Formula
  desc "FOSS self-hosted odrive replacement — FUSE-based, SeaweedFS-backed file sync"
  homepage "https://github.com/tinyland-inc/tummycrypt"
  version "0.9.3"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.9.3/tcfs-0.9.3-macos-aarch64.tar.gz"
      sha256 "d13aeb88292a844c518b652c8a0c9c66319bba7f1741093570acf12b8b58bf50"
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.9.3/tcfs-0.9.3-macos-x86_64.tar.gz"
      sha256 "b9b19921e4906d756d4a31144413a3cd0ea9df4a5aa3e15773c4ca1f4e7282c9"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.9.3/tcfs-0.9.3-linux-aarch64.tar.gz"
      sha256 "46f828107a3604e696f8b6ccef1003f6254baae6eae6105a02e3a875083099bf"
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.9.3/tcfs-0.9.3-linux-x86_64.tar.gz"
      sha256 "4232fb577008b498d05e98457b63f0ff6343f7860831baae661989347b960a30"
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
