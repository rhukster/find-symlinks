class FindSymlinks < Formula
  desc "Fast symlink finder"
  homepage "https://github.com/rhukster/find-symlinks"
  url "https://github.com/rhukster/find-symlinks/archive/refs/tags/v0.1.0.tar.gz"
  sha256 "794a1eb452a9ede1d47fd252ff80b7469eae3153b2ec07cb9a02e26f17764c05"
  head "https://github.com/rhukster/find-symlinks.git", branch: "main"

  depends_on "rust" => :build

  def install
    system "cargo", "install", *std_cargo_args
  end

  test do
    assert_match "find-symlinks", shell_output("#{bin}/find-symlinks --version")
  end
end
