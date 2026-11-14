import Foundation

/// Cát, Trung or Hung as written by the build into `entry.luck` and `extra.verdict`.
enum ContentLuck: String, Sendable, Hashable, Decodable {
    case cat = "Cat"
    case trung = "Trung"
    case hung = "Hung"

    var label: String {
        switch self {
        case .cat: "Cát"
        case .trung: "Bình"
        case .hung: "Hung"
        }
    }
}

/// The `sensitivity` bit mask of the build, in the order of `Sensitivity` in astro-data.
struct Sensitivity: OptionSet, Sendable, Hashable {
    let rawValue: Int64

    static let death = Sensitivity(rawValue: 1 << 0)
    static let illness = Sensitivity(rawValue: 1 << 1)
    static let legal = Sensitivity(rawValue: 1 << 2)
    static let sexuality = Sensitivity(rawValue: 1 << 3)
    static let familyLoss = Sensitivity(rawValue: 1 << 4)

    /// Decision D9: these stay behind one tap, with a note that the text is cultural reference.
    var isHiddenByDefault: Bool {
        !intersection([.death, .sexuality]).isEmpty
    }
}

struct ActivityRef: Sendable, Hashable, Decodable {
    let activity: String
    let note: String?
}

struct StarProfile: Sendable, Hashable, Decodable {
    let element: String?
    let labels: [String]
}

/// The `extra` column: the fields some collections add (section 3 of the spec).
struct EntryExtra: Sendable, Hashable, Decodable {
    var goodFor: [ActivityRef] = []
    var avoid: [ActivityRef] = []
    var verse: String?
    var image: String?
    var traits: [String] = []
    var fortune: [String] = []
    var verdict: ContentLuck?
    var auspiciousMeaning: String?
    var inauspiciousMeaning: String?
    var profile: StarProfile?

    init() {}

    init(from decoder: any Decoder) throws {
        let container = try decoder.container(keyedBy: CodingKeys.self)
        goodFor = try container.decodeIfPresent([ActivityRef].self, forKey: .goodFor) ?? []
        avoid = try container.decodeIfPresent([ActivityRef].self, forKey: .avoid) ?? []
        verse = try container.decodeIfPresent(String.self, forKey: .verse)
        image = try container.decodeIfPresent(String.self, forKey: .image)
        traits = try container.decodeIfPresent([String].self, forKey: .traits) ?? []
        fortune = try container.decodeIfPresent([String].self, forKey: .fortune) ?? []
        verdict = try container.decodeIfPresent(ContentLuck.self, forKey: .verdict)
        auspiciousMeaning = try container.decodeIfPresent(String.self, forKey: .auspiciousMeaning)
        inauspiciousMeaning = try container.decodeIfPresent(String.self, forKey: .inauspiciousMeaning)
        profile = try container.decodeIfPresent(StarProfile.self, forKey: .profile)
    }

    private enum CodingKeys: String, CodingKey {
        case goodFor = "good_for"
        case avoid, verse, image, traits, fortune, verdict, profile
        case auspiciousMeaning = "auspicious_meaning"
        case inauspiciousMeaning = "inauspicious_meaning"
    }
}

/// One row of `entry`: the text of a key such as `almanac.truc/Kien`.
struct ContentEntry: Identifiable, Sendable, Hashable {
    let key: String
    let collection: String
    let title: String
    let luck: ContentLuck?
    let summary: String
    let body: [String]
    let extra: EntryExtra
    let citation: String
    let sensitivity: Sensitivity

    var id: String { key }

    /// `extra.verdict` overrides the luck of the star where a use inverts it, as for a stove.
    var effectiveLuck: ContentLuck? { extra.verdict ?? luck }
}

enum PassageTier: String, Sendable, Hashable {
    case primary = "Primary"
    case more = "More"
}

enum PassageSection: String, Sendable, Hashable {
    case daiCuong = "DaiCuong"
    case namMenh = "NamMenh"
    case nuMenh = "NuMenh"
    case phuGiai = "PhuGiai"

    var title: String {
        switch self {
        case .daiCuong: "Đại cương"
        case .namMenh: "Nam mệnh"
        case .nuMenh: "Nữ mệnh"
        case .phuGiai: "Phú giải"
        }
    }
}

/// One conditional Tử Vi passage.
struct Passage: Identifiable, Sendable, Hashable {
    let id: Int64
    let stableID: String
    let tier: PassageTier
    let section: PassageSection?
    let verse: String?
    let conditionText: String?
    let body: [String]
    let citation: String
    let sensitivity: Sensitivity
    /// Where the text departs from the printed book, and why (D20 of the content spec).
    let correction: String?
}

struct PassageGroup: Identifiable, Sendable, Hashable {
    let section: PassageSection?
    let passages: [Passage]

    var id: String { section?.rawValue ?? "" }
}

/// Everything shown for one palace: entries of its keys, then the passages in book order.
struct PalaceReading: Sendable, Hashable {
    let entries: [ContentEntry]
    let passages: [Passage]

    var primary: [PassageGroup] { Self.groups(passages.filter { $0.tier == .primary }) }
    var more: [PassageGroup] { Self.groups(passages.filter { $0.tier == .more }) }

    /// Groups by section in order of first appearance, keeping book order inside each.
    private static func groups(_ passages: [Passage]) -> [PassageGroup] {
        var order: [PassageSection?] = []
        var bySection: [PassageSection?: [Passage]] = [:]
        for passage in passages {
            if bySection[passage.section] == nil {
                order.append(passage.section)
            }
            bySection[passage.section, default: []].append(passage)
        }
        return order.map { PassageGroup(section: $0, passages: bySection[$0] ?? []) }
    }
}

/// A Nine Star with the one of its two texts that applies in the current period.
struct NineStarReading: Sendable, Hashable {
    let entry: ContentEntry
    let phase: NineStarPhase
    let phaseName: String
    let isAuspicious: Bool

    var meaning: String? {
        isAuspicious ? entry.extra.auspiciousMeaning : entry.extra.inauspiciousMeaning
    }
}

struct ContentMetadata: Sendable, Hashable {
    let schemaVersion: Int64
    let engineVersion: String
    let contentHash: String
}
