# Homebrew formula for tcfs
# To use: brew tap tinyland-inc/tap && brew install tcfs
#
# This template is used by CI to generate the versioned formula.
# Placeholders: 0.3.0, 0cdf67aa48175d9fcc57f54790baa0399eb51a5234c282829f195db838ee921c, c89efedae8ca4662594661bd669a06617465511df4dc4aa4899e21da0339ac9e,
#               d106c4554947c0822db0654898b944099b9950ceaf22a7ad3c44759a4d2fe9d6, f1a5859ce7444aabacb8bc9723e043712cfb99b00eee64aebb4178f17848882a

class Tcfs < Formula
  desc "FOSS self-hosted odrive replacement — FUSE-based, SeaweedFS-backed file sync"
  homepage "https://github.com/tinyland-inc/tummycrypt"
  version "0.3.0"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.3.0/tcfs-0.3.0-macos-aarch64.tar.gz"
      sha256 "0cdf67aa48175d9fcc57f54790baa0399eb51a5234c282829f195db838ee921c"
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.3.0/tcfs-0.3.0-macos-x86_64.tar.gz"
      sha256 "c89efedae8ca4662594661bd669a06617465511df4dc4aa4899e21da0339ac9e"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.3.0/tcfs-0.3.0-linux-aarch64.tar.gz"
      sha256 "f1a5859ce7444aabacb8bc9723e043712cfb99b00eee64aebb4178f17848882a"
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.3.0/tcfs-0.3.0-linux-x86_64.tar.gz"
      sha256 "d106c4554947c0822db0654898b944099b9950ceaf22a7ad3c44759a4d2fe9d6"
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
