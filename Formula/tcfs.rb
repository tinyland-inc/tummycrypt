# Homebrew formula for tcfs
# To use: brew tap tinyland-inc/tap && brew install tcfs
#
# This template is used by CI to generate the versioned formula.
# Placeholders: 0.2.2, dae6065d0688cd008cedd138dab55139574d3758ee0a600789194f76db315356, 0890e761f29b7f9c42dfffe70e80c2646ea77c698d5a31a158c85a47dbe2903b,
#               30fbf76673297b6aa5d37866f82e0c54c22589b52c86b044c2668ccad34a894f, c5a6e8edeec4fd72d28099f807ec77633e637fc6b53b1fb4df1e16efd6ade56b

class Tcfs < Formula
  desc "FOSS self-hosted odrive replacement — FUSE-based, SeaweedFS-backed file sync"
  homepage "https://github.com/tinyland-inc/tummycrypt"
  version "0.2.2"
  license any_of: ["MIT", "Apache-2.0"]

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.2.2/tcfs-0.2.2-macos-aarch64.tar.gz"
      sha256 "dae6065d0688cd008cedd138dab55139574d3758ee0a600789194f76db315356"
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.2.2/tcfs-0.2.2-macos-x86_64.tar.gz"
      sha256 "0890e761f29b7f9c42dfffe70e80c2646ea77c698d5a31a158c85a47dbe2903b"
    end
  end

  on_linux do
    if Hardware::CPU.arm?
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.2.2/tcfs-0.2.2-linux-aarch64.tar.gz"
      sha256 "c5a6e8edeec4fd72d28099f807ec77633e637fc6b53b1fb4df1e16efd6ade56b"
    else
      url "https://github.com/tinyland-inc/tummycrypt/releases/download/v0.2.2/tcfs-0.2.2-linux-x86_64.tar.gz"
      sha256 "30fbf76673297b6aa5d37866f82e0c54c22589b52c86b044c2668ccad34a894f"
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
