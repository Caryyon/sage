class Sage < Formula
  desc "Decentralized AI that runs on your machine"
  homepage "https://whatssage.ai"
  url "https://github.com/Caryyon/sage/archive/refs/tags/v#{version}.tar.gz"
  version "0.3.7"
  sha256 "PLACEHOLDER"
  license "MIT"

  depends_on "rust" => :build

  def install
    system "cargo", "build", "--release", "--bin", "sage-cli"
    bin.install "target/release/sage-cli" => "sage"
  end

  test do
    system "#{bin}/sage", "--version"
  end
end
