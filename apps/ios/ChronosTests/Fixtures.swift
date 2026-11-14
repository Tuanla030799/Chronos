import Foundation
@testable import Chronos

extension Profile {
    /// The person most tests read: a man born 15 May 1990 at 09:30 in Vietnam.
    static let sample = Profile(
        name: "Hồ sơ mẫu",
        gender: .male,
        birth: BirthDate(kind: .solar, year: 1990, month: 5, day: 15, hour: 9, minute: 30)
    )
}
