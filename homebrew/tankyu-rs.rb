class TankyuRs < Formula
  desc "Research intelligence graph — Rust port"
  homepage "https://github.com/cmbays/tankyu-rs"
  version "0.1.0" # updated by release automation

  on_macos do
    if Hardware::CPU.arm?
      url "https://github.com/cmbays/tankyu-rs/releases/download/v#{version}/tankyu-aarch64-apple-darwin"
      sha256 "" # updated by release automation
    else
      url "https://github.com/cmbays/tankyu-rs/releases/download/v#{version}/tankyu-x86_64-apple-darwin"
      sha256 "" # updated by release automation
    end
  end

  on_linux do
    url "https://github.com/cmbays/tankyu-rs/releases/download/v#{version}/tankyu-x86_64-unknown-linux-gnu"
    sha256 "" # updated by release automation
  end

  def install
    if OS.mac? && Hardware::CPU.arm?
      bin.install "tankyu-aarch64-apple-darwin" => "tankyu-rs"
    elsif OS.mac?
      bin.install "tankyu-x86_64-apple-darwin" => "tankyu-rs"
    else
      bin.install "tankyu-x86_64-unknown-linux-gnu" => "tankyu-rs"
    end
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/tankyu-rs --version")
  end
end
