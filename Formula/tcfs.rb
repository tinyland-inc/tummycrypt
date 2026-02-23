# Homebrew formula for tcfs
# To use: brew tap tinyland-inc/tap && brew install tcfs
#
# This template is used by CI to generate the versioned formula.
# Placeholders: 0.4.0, ec22a219c501edba588acdde0c35a7b8ef1857f89d91257bd9d2ea9173b99d1d, 638e735097034f7e2bc376268deb6021195198cec6c272930751f4bb6286553d,
#               2b0de3928c5379f3382afd53000385ddc33f4359e381b42fbe3f5e9e3b18a392, 80773fe049d376f934a75f6e6160350511f933d03db87d3424eacd214c33e983

class Tcfs < Formula
  desc "FOSS self-hosted odrive replacement — FUSE-based, SeaweedFS-backed file sync"
  homepage "https://github.com/tinyland-inc/tummycrypt"
  version "0.4.0"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.4.0/tcfs-0.4.0-macos-aarch64.tar.gz"
      sha256 "ec22a219c501edba588acdde0c35a7b8ef1857f89d91257bd9d2ea9173b99d1d"
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.4.0/tcfs-0.4.0-macos-x86_64.tar.gz"
      sha256 "638e735097034f7e2bc376268deb6021195198cec6c272930751f4bb6286553d"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.4.0/tcfs-0.4.0-linux-aarch64.tar.gz"
      sha256 "80773fe049d376f934a75f6e6160350511f933d03db87d3424eacd214c33e983"
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.4.0/tcfs-0.4.0-linux-x86_64.tar.gz"
      sha256 "2b0de3928c5379f3382afd53000385ddc33f4359e381b42fbe3f5e9e3b18a392"
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
