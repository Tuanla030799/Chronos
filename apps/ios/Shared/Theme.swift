import SwiftUI
import UIKit

/// Palette of docs/IOS_APP_SPEC.md section 4: giấy Dó and mực Nho by day, Đêm Huyền Không by
/// night, Chu Sa and Kim as accents. Shared by the app and the widget.
enum Theme {
    static let paper = dynamic(light: 0xF4ECDC, dark: 0x0E1220)
    static let surface = dynamic(light: 0xFBF6EA, dark: 0x171C2E)
    static let ink = dynamic(light: 0x1F1A17, dark: 0xEDE6D6)
    static let inkSecondary = dynamic(light: 0x5B5148, dark: 0xA69F92)
    static let rule = dynamic(light: 0xD9CCB4, dark: 0x2A3047)
    static let cinnabar = dynamic(light: 0xB23A2B, dark: 0xE0604C)
    static let gold = dynamic(light: 0x85601E, dark: 0xD9AE5A)

    /// Ngũ Hành, each at least 4.5:1 on paper and surface in both appearances, also as text on its own 12% tint.
    static let kim = dynamic(light: 0x565E6B, dark: 0xB9C2CF)
    static let moc = dynamic(light: 0x2E6B3B, dark: 0x73C48A)
    static let thuy = dynamic(light: 0x1F5F8B, dark: 0x6FB1E6)
    static let hoa = dynamic(light: 0x9C4116, dark: 0xF28A5E)
    static let tho = dynamic(light: 0x7A5430, dark: 0xCFA97E)

    static func element(_ element: FiveElements) -> Color {
        switch element {
        case .kim: kim
        case .moc: moc
        case .thuy: thuy
        case .hoa: hoa
        case .tho: tho
        }
    }

    private static func dynamic(light: UInt32, dark: UInt32) -> Color {
        Color(uiColor: UIColor { traits in
            uiColor(traits.userInterfaceStyle == .dark ? dark : light)
        })
    }

    private static func uiColor(_ hex: UInt32) -> UIColor {
        UIColor(
            red: CGFloat((hex >> 16) & 0xFF) / 255,
            green: CGFloat((hex >> 8) & 0xFF) / 255,
            blue: CGFloat(hex & 0xFF) / 255,
            alpha: 1
        )
    }
}
