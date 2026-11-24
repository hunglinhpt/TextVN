// SPDX-License-Identifier: GPL-3.0-or-later
//! Bảng chọn Emoji nhẹ phong cách macOS / iOS / MS Teams cho TextVN IMK (P12).
//!
//! - `NSPanel` nổi với `.nonactivatingPanel` + `NSVisualEffectView` (vật liệu popover chuẩn macOS)
//!   + Apple Color Emoji gốc — siêu nhẹ, KHÔNG cướp focus của ứng dụng đang gõ.
//! - Hỗ trợ 8 danh mục quen thuộc (Gần đây MRU + 7 nhóm), lọc trực tiếp VI/EN không dấu,
//!   điều hướng bằng phím mũi tên / Tab / Enter / Esc hoặc click chuột.
//! - G8: không ghi log từ khóa tìm kiếm hay emoji người dùng chọn.

import AppKit
import CoreBridge
import Foundation

public struct EmojiCategoryTab: Equatable {
    public let id: UInt8
    public let icon: String
    public let label: String
}

public enum EmojiPickerAction: Equatable {
    case redraw
    case reload(query: String, category: UInt8)
    case commit(String)
    case close
}

/// Trạng thái thuần của Emoji Picker (test được bằng `swift test` không cần WindowServer).
public final class EmojiPickerModel {
    public static let cols = 8
    public static let visibleRows = 5
    public static let tabs: [EmojiCategoryTab] = [
        EmojiCategoryTab(id: 0, icon: "🕒", label: "Gần đây"),
        EmojiCategoryTab(id: 1, icon: "😀", label: "Cảm xúc"),
        EmojiCategoryTab(id: 2, icon: "👋", label: "Cử chỉ"),
        EmojiCategoryTab(id: 3, icon: "❤️", label: "Biểu tượng"),
        EmojiCategoryTab(id: 4, icon: "🐶", label: "Thiên nhiên"),
        EmojiCategoryTab(id: 5, icon: "🍜", label: "Đồ ăn"),
        EmojiCategoryTab(id: 6, icon: "💻", label: "Công việc"),
        EmojiCategoryTab(id: 7, icon: "🎉", label: "Hoạt động"),
    ]

    public private(set) var visible = false
    public private(set) var query = ""
    public private(set) var category: UInt8 = 0
    public private(set) var items: [EmojiItem] = []
    public private(set) var selected = 0
    public private(set) var scrollRow = 0

    public init() {}

    public func open() {
        visible = true
        query = ""
        category = 0
        selected = 0
        scrollRow = 0
    }

    public func close() {
        visible = false
        query = ""
        scrollRow = 0
    }

    public func setItems(_ next: [EmojiItem]) {
        items = next
        if selected >= items.count { selected = 0 }
        scrollRow = 0
        ensureVisible()
    }

    public func switchCategory(_ cat: UInt8) {
        category = cat
        query = ""
        selected = 0
        scrollRow = 0
    }

    public func selectedItem() -> EmojiItem? {
        guard selected >= 0 && selected < items.count else { return nil }
        return items[selected]
    }

    public func scrollBy(_ deltaRows: Int) {
        let totalRows = (items.count + Self.cols - 1) / Self.cols
        let maxScroll = max(0, totalRows - Self.visibleRows)
        scrollRow = min(max(0, scrollRow + deltaRows), maxScroll)
    }

    private func ensureVisible() {
        let row = selected / Self.cols
        if row < scrollRow {
            scrollRow = row
        } else if row >= scrollRow + Self.visibleRows {
            scrollRow = row + 1 - Self.visibleRows
        }
    }

    /// Xử lý phím theo macOS hardware keyCode (Esc=53, Return=36, Tab=48, Backspace=51,
    /// Left=123, Right=124, Down=125, Up=126) và ký tự in được.
    public func handleKey(keyCode: UInt16, chars: String?, shift: Bool) -> EmojiPickerAction {
        switch keyCode {
        case 53: // Escape
            close()
            return .close
        case 36, 76: // Return / Keypad Enter
            if let item = selectedItem() {
                close()
                return .commit(item.glyph)
            }
            close()
            return .close
        case 48: // Tab
            let cur = Self.tabs.firstIndex { $0.id == category } ?? 0
            let next = shift ? (cur + Self.tabs.count - 1) % Self.tabs.count : (cur + 1) % Self.tabs.count
            switchCategory(Self.tabs[next].id)
            return .reload(query: query, category: category)
        case 51: // Backspace
            if !query.isEmpty {
                query.removeLast()
                selected = 0
                scrollRow = 0
                return .reload(query: query, category: category)
            }
            return .redraw
        case 123: // Left
            if selected > 0 { selected -= 1; ensureVisible() }
            return .redraw
        case 124: // Right
            if selected + 1 < items.count { selected += 1; ensureVisible() }
            return .redraw
        case 126: // Up
            if selected >= Self.cols { selected -= Self.cols; ensureVisible() }
            return .redraw
        case 125: // Down
            if !items.isEmpty {
                selected = min(selected + Self.cols, items.count - 1)
                ensureVisible()
            }
            return .redraw
        default:
            if let s = chars, let scalar = s.unicodeScalars.first, scalar.value >= 0x20, query.count < 32 {
                query.append(Character(scalar))
                selected = 0
                scrollRow = 0
                return .reload(query: query, category: category)
            }
            return .redraw
        }
    }
}

/// Cửa sổ nổi không cướp focus hiển thị lưới Apple Color Emoji.
public final class EmojiPickerPanel {
    private static let panelWidth: CGFloat = 332
    private static let panelHeight: CGFloat = 316

    public let model = EmojiPickerModel()
    public var onSelectGlyph: ((String) -> Void)?
    public var onSwitchCategory: ((UInt8) -> Void)?

    private var panel: NSPanel?
    private var canvas: EmojiCanvasView?

    public init() {}

    public var isVisible: Bool { model.visible }

    public func show(at anchor: NSPoint?, items: [EmojiItem]) {
        model.open()
        model.setItems(items)
        ensurePanel()
        guard let panel = panel, let canvas = canvas else { return }
        canvas.needsDisplay = true
        let origin = clampedOrigin(near: anchor ?? NSEvent.mouseLocation)
        panel.setFrameOrigin(origin)
        panel.orderFrontRegardless()
    }

    public func hide() {
        model.close()
        panel?.orderOut(nil)
    }

    public func updateItems(_ items: [EmojiItem]) {
        model.setItems(items)
        canvas?.needsDisplay = true
    }

    public func handleKey(keyCode: UInt16, chars: String?, shift: Bool) -> EmojiPickerAction {
        let action = model.handleKey(keyCode: keyCode, chars: chars, shift: shift)
        switch action {
        case .close, .commit:
            panel?.orderOut(nil)
        case .redraw, .reload:
            canvas?.needsDisplay = true
        }
        return action
    }

    private func ensurePanel() {
        if panel != nil { return }
        let frame = NSRect(x: 0, y: 0, width: Self.panelWidth, height: Self.panelHeight)
        let p = NSPanel(
            contentRect: frame,
            styleMask: [.borderless, .nonactivatingPanel],
            backing: .buffered,
            defer: false
        )
        p.level = .popUpMenu
        p.isOpaque = false
        p.backgroundColor = .clear
        p.hasShadow = true
        p.hidesOnDeactivate = false
        p.collectionBehavior = [.canJoinAllSpaces, .fullScreenAuxiliary]

        let fx = NSVisualEffectView(frame: frame)
        fx.material = .popover
        fx.blendingMode = .behindWindow
        fx.state = .active
        fx.wantsLayer = true
        fx.layer?.cornerRadius = 10
        fx.layer?.masksToBounds = true

        let view = EmojiCanvasView(frame: frame, model: model)
        view.onPickCell = { [weak self] idx in
            guard let self = self, idx >= 0 && idx < self.model.items.count else { return }
            let glyph = self.model.items[idx].glyph
            self.hide()
            self.onSelectGlyph?(glyph)
        }
        view.onPickTab = { [weak self] cat in
            guard let self = self else { return }
            self.model.switchCategory(cat)
            self.onSwitchCategory?(cat)
        }
        fx.addSubview(view)
        p.contentView = fx
        self.panel = p
        self.canvas = view
    }

    private func clampedOrigin(near pt: NSPoint) -> NSPoint {
        let screen = NSScreen.screens.first { NSMouseInRect(pt, $0.frame, false) } ?? NSScreen.main
        let vis = screen?.visibleFrame ?? NSRect(x: 0, y: 0, width: 1440, height: 900)
        var x = pt.x
        if x + Self.panelWidth > vis.maxX { x = vis.maxX - Self.panelWidth - 8 }
        if x < vis.minX { x = vis.minX + 8 }
        var y = pt.y - Self.panelHeight - 8
        if y < vis.minY { y = min(pt.y + 24, vis.maxY - Self.panelHeight - 8) }
        return NSPoint(x: x, y: y)
    }
}

private final class EmojiCanvasView: NSView {
    private let model: EmojiPickerModel
    var onPickCell: ((Int) -> Void)?
    var onPickTab: ((UInt8) -> Void)?

    private let padX: CGFloat = 10
    private let headerH: CGFloat = 38
    private let tabsH: CGFloat = 34
    private let footerH: CGFloat = 26
    private let cellW: CGFloat = 39
    private let cellH: CGFloat = 42

    override var isFlipped: Bool { true }

    init(frame: NSRect, model: EmojiPickerModel) {
        self.model = model
        super.init(frame: frame)
    }

    @available(*, unavailable)
    required init?(coder: NSCoder) { fatalError() }

    override func acceptsFirstMouse(for event: NSEvent?) -> Bool { true }

    override func scrollWheel(with event: NSEvent) {
        let dy = event.scrollingDeltaY
        if abs(dy) >= 1 {
            model.scrollBy(dy > 0 ? -1 : 1)
            needsDisplay = true
        }
    }

    override func mouseDown(with event: NSEvent) {
        let pt = convert(event.locationInWindow, from: nil)
        let tabY0 = headerH
        if pt.y >= tabY0 && pt.y < tabY0 + tabsH && pt.x >= padX && pt.x < bounds.width - padX {
            let tabW = (bounds.width - 2 * padX) / CGFloat(EmojiPickerModel.tabs.count)
            let idx = Int((pt.x - padX) / tabW)
            if idx >= 0 && idx < EmojiPickerModel.tabs.count {
                onPickTab?(EmojiPickerModel.tabs[idx].id)
            }
            return
        }
        let gridTop = headerH + tabsH + 4
        let gridBottom = gridTop + CGFloat(EmojiPickerModel.visibleRows) * cellH
        if pt.y >= gridTop && pt.y < gridBottom && pt.x >= padX && pt.x < padX + CGFloat(EmojiPickerModel.cols) * cellW {
            let col = Int((pt.x - padX) / cellW)
            let row = Int((pt.y - gridTop) / cellH)
            let idx = (model.scrollRow + row) * EmojiPickerModel.cols + col
            if idx >= 0 && idx < model.items.count {
                onPickCell?(idx)
            }
        }
    }

    override func draw(_ dirtyRect: NSRect) {
        // Thanh tìm kiếm.
        let searchRect = NSRect(x: padX, y: 8, width: bounds.width - 2 * padX, height: 26)
        let searchBg = NSBezierPath(roundedRect: searchRect, xRadius: 6, yRadius: 6)
        NSColor.controlBackgroundColor.withAlphaComponent(0.55).setFill()
        searchBg.fill()

        let searchText = model.query.isEmpty ? "🔍 Gõ tìm emoji (cười, tim, fire...)" : "🔍 \(model.query)"
        let searchAttrs: [NSAttributedString.Key: Any] = [
            .font: NSFont.systemFont(ofSize: 12.5),
            .foregroundColor: model.query.isEmpty ? NSColor.secondaryLabelColor : NSColor.labelColor,
        ]
        (searchText as NSString).draw(
            in: searchRect.insetBy(dx: 8, dy: 4),
            withAttributes: searchAttrs
        )

        // Dải 8 tab danh mục.
        let tabW = (bounds.width - 2 * padX) / CGFloat(EmojiPickerModel.tabs.count)
        let tabStyle = NSMutableParagraphStyle()
        tabStyle.alignment = .center
        let tabAttrs: [NSAttributedString.Key: Any] = [
            .font: NSFont.systemFont(ofSize: 15),
            .paragraphStyle: tabStyle,
        ]
        for (i, tab) in EmojiPickerModel.tabs.enumerated() {
            let tr = NSRect(x: padX + CGFloat(i) * tabW, y: headerH, width: tabW, height: tabsH)
            if model.query.isEmpty && model.category == tab.id {
                NSColor.controlAccentColor.withAlphaComponent(0.22).setFill()
                NSBezierPath(roundedRect: tr.insetBy(dx: 2, dy: 3), xRadius: 6, yRadius: 6).fill()
            }
            (tab.icon as NSString).draw(in: tr.offsetBy(dx: 0, dy: 6), withAttributes: tabAttrs)
        }

        // Lưới 8×5 Apple Color Emoji.
        let gridTop = headerH + tabsH + 4
        let emojiStyle = NSMutableParagraphStyle()
        emojiStyle.alignment = .center
        let emojiAttrs: [NSAttributedString.Key: Any] = [
            .font: NSFont.systemFont(ofSize: 22),
            .paragraphStyle: emojiStyle,
        ]
        let start = model.scrollRow * EmojiPickerModel.cols
        for row in 0..<EmojiPickerModel.visibleRows {
            for col in 0..<EmojiPickerModel.cols {
                let idx = start + row * EmojiPickerModel.cols + col
                guard idx < model.items.count else { break }
                let cell = NSRect(
                    x: padX + CGFloat(col) * cellW + 2,
                    y: gridTop + CGFloat(row) * cellH + 2,
                    width: cellW - 4,
                    height: cellH - 4
                )
                if idx == model.selected {
                    NSColor.controlAccentColor.withAlphaComponent(0.28).setFill()
                    NSBezierPath(roundedRect: cell, xRadius: 6, yRadius: 6).fill()
                }
                (model.items[idx].glyph as NSString).draw(
                    in: cell.offsetBy(dx: 0, dy: 5),
                    withAttributes: emojiAttrs
                )
            }
        }

        // Thanh chân: shortcode + phím tắt.
        let footerY = bounds.height - footerH
        let footerText: String
        if let item = model.selectedItem() {
            footerText = ":\(item.shortcode):  ·  Enter chèn · Tab đổi nhóm · Esc đóng"
        } else {
            footerText = "Không tìm thấy emoji  ·  Esc đóng"
        }
        let footerAttrs: [NSAttributedString.Key: Any] = [
            .font: NSFont.systemFont(ofSize: 11),
            .foregroundColor: NSColor.secondaryLabelColor,
        ]
        (footerText as NSString).draw(
            in: NSRect(x: padX, y: footerY + 5, width: bounds.width - 2 * padX, height: footerH - 4),
            withAttributes: footerAttrs
        )
    }
}
