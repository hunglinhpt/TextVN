# SPDX-License-Identifier: GPL-3.0-or-later
# textvn.rb — Homebrew Cask formula for TextVN on macOS

cask "textvn" do
  version "0.2.9"
  # sha256 của TextVN-macos-universal-v<version>.zip — cập nhật MỖI bản phát hành
  # (lấy từ SHA256SUMS.txt của release; bước B7b trong release-process.md)
  sha256 "2af5a9c7db1be679cd6628ba5ef98569f123b1f572a1f53d336e3f06d892a71d"

  url "https://github.com/hunglinhpt/TextVN/releases/download/v#{version}/TextVN-macos-universal-v#{version}.zip"
  name "TextVN"
  desc "Vietnamese input method engine and menu bar control panel"
  homepage "https://github.com/hunglinhpt/TextVN"

  depends_on macos: ">= :ventura"

  app "TextVN.app"
  input_method "TextVN-IM.app"

  postflight do
    system_command "/usr/bin/touch",
                   args: ["#{ENV.fetch("HOME")}/Library/Input Methods"]
  end

  uninstall quit: [
              "vn.textvn.app",
              "vn.textvn.im",
            ],
            delete: [
              "~/Library/Input Methods/TextVN-IM.app",
              "~/Library/LaunchAgents/vn.textvn.app.plist",
            ]

  zap trash: [
    "~/Library/Application Support/TextVN",
    "~/Library/Logs/TextVN",
  ]
end
