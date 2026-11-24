#!/usr/bin/env bash
set -euo pipefail

if [ "$#" -ne 1 ]; then
  echo "usage: $0 <tag>" >&2
  exit 1
fi

tag="$1"
version="${tag#v}"
repo="donkomura/rjq"

asset_sha256() {
  local name="$1"
  local digest
  digest=$(gh release view "$tag" --repo "$repo" --json assets \
    --jq ".assets[] | select(.name == \"$name\") | .digest")
  if [ -z "$digest" ]; then
    echo "asset $name not found in release $tag" >&2
    exit 1
  fi
  echo "${digest#sha256:}"
}

macos_arm=$(asset_sha256 "rjq-aarch64-apple-darwin.tar.gz")
macos_intel=$(asset_sha256 "rjq-x86_64-apple-darwin.tar.gz")
linux_arm=$(asset_sha256 "rjq-aarch64-unknown-linux-musl.tar.gz")
linux_intel=$(asset_sha256 "rjq-x86_64-unknown-linux-musl.tar.gz")

base_url="https://github.com/$repo/releases/download/$tag"

cat <<EOF
class Rjq < Formula
  desc "Interactive CLI tool for JSON querying"
  homepage "https://github.com/$repo"
  license "MIT"

  on_macos do
    on_arm do
      url "$base_url/rjq-aarch64-apple-darwin.tar.gz"
      sha256 "$macos_arm"
    end
    on_intel do
      url "$base_url/rjq-x86_64-apple-darwin.tar.gz"
      sha256 "$macos_intel"
    end
  end

  on_linux do
    on_arm do
      url "$base_url/rjq-aarch64-unknown-linux-musl.tar.gz"
      sha256 "$linux_arm"
    end
    on_intel do
      url "$base_url/rjq-x86_64-unknown-linux-musl.tar.gz"
      sha256 "$linux_intel"
    end
  end

  def install
    bin.install "rjq"
  end

  test do
    assert_match version.to_s, shell_output("#{bin}/rjq --version")
  end
end
EOF
