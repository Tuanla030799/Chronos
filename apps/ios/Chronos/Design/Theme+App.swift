import SwiftUI

extension Theme {
    static func luck(_ luck: ContentLuck?) -> Color {
        switch luck {
        case .cat: gold
        case .hung: cinnabar
        case .trung, nil: inkSecondary
        }
    }
}

extension View {
    /// Paper behind a `List` or `Form`. Its rows set `.listRowBackground(Theme.surface)`
    /// themselves: that modifier only reaches a row from inside the list.
    func chronosBackground() -> some View {
        scrollContentBackground(.hidden)
            .background(Theme.paper)
    }
}
