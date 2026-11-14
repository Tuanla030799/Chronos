import Foundation

enum ContentRepositoryError: Error, Equatable, CustomStringConvertible {
    case missingResource(String)
    case incompatibleFile(applicationID: Int64, schemaVersion: Int64)
    case malformedColumn(key: String, column: String)

    var description: String {
        switch self {
        case .missingResource(let name):
            "Thiếu \(name) trong bundle. Chạy `cargo run -p astro-data -- build --release` rồi build lại app."
        case .incompatibleFile(let applicationID, let schemaVersion):
            "File dữ liệu không đúng định dạng (application_id \(applicationID), schema \(schemaVersion))."
        case .malformedColumn(let key, let column):
            "Cột \(column) của \(key) không đọc được."
        }
    }
}

/// Reads `chronos_content.sqlite` from the app bundle. Keys always come from AstroCore
/// (`almanacDayKeys`, `tuviPalaceQuery`, …): the app never builds a key string itself.
final class ContentRepository: Sendable {
    static let applicationID: Int64 = 1_128_813_134
    static let schemaVersion: Int64 = 2

    let metadata: ContentMetadata
    private let database: SQLiteDatabase
    private let activityLabels: [String: String]
    private let entrySQL: String
    private let passageSQL: String

    /// Opens the files bundled with the app. Drafts appear only in a development build of the
    /// content, never in a release one.
    convenience init(bundle: Bundle = .main, includeDrafts: Bool = false) throws {
        guard let database = bundle.url(forResource: "chronos_content", withExtension: "sqlite") else {
            throw ContentRepositoryError.missingResource("chronos_content.sqlite")
        }
        guard let activities = bundle.url(forResource: "activities", withExtension: "json") else {
            throw ContentRepositoryError.missingResource("activities.json")
        }
        try self.init(databaseURL: database, activitiesURL: activities, includeDrafts: includeDrafts)
    }

    init(databaseURL: URL, activitiesURL: URL, includeDrafts: Bool = false) throws {
        let database = try SQLiteDatabase(readOnlyAt: databaseURL)
        let applicationID = try database.pragma("application_id")
        let schemaVersion = try database.pragma("user_version")
        guard applicationID == Self.applicationID, schemaVersion == Self.schemaVersion else {
            throw ContentRepositoryError.incompatibleFile(applicationID: applicationID, schemaVersion: schemaVersion)
        }
        let meta = Dictionary(
            try database.rows("SELECT key, value FROM meta") { ($0.text(0) ?? "", $0.text(1) ?? "") },
            uniquingKeysWith: { first, _ in first }
        )
        metadata = ContentMetadata(
            schemaVersion: schemaVersion,
            engineVersion: meta["engine_version"] ?? "",
            contentHash: meta["content_hash"] ?? ""
        )

        struct Activity: Decodable {
            let id: String
            let label: String
        }
        let activities = try JSONDecoder().decode([Activity].self, from: Data(contentsOf: activitiesURL))
        activityLabels = Dictionary(activities.map { ($0.id, $0.label) }, uniquingKeysWith: { first, _ in first })

        let draft = includeDrafts ? "" : " AND draft = 0"
        entrySQL = """
            SELECT key, collection, title, luck, summary, body, extra, citation, sensitivity
            FROM entry WHERE key = ?\(draft)
            """
        passageSQL = """
            SELECT p.id, p.stable_id, p.tier, p.section, p.verse, p.gates, p.condition_text,
                   p.body, p.citation, p.sensitivity, p.correction
            FROM passage_anchor a JOIN passage p ON p.id = a.passage_id
            WHERE a.anchor = ?\(draft.replacingOccurrences(of: "draft", with: "p.draft"))
            ORDER BY p.id
            """
        self.database = database
    }

    // MARK: Entries

    func entry(_ key: String) throws -> ContentEntry? {
        try database.rows(entrySQL, [key], map: Self.entry).first
    }

    /// Entries of `keys` in the same order. A key without text, a gap of the data, is skipped.
    func entries(_ keys: [String]) throws -> [ContentEntry] {
        try keys.compactMap(entry)
    }

    /// Vietnamese label of an activity id of `good_for` and `avoid`.
    func activityLabel(_ id: String) -> String {
        activityLabels[id] ?? id
    }

    func almanacDay(_ day: AlmanacDay) throws -> [ContentEntry] {
        try entries(almanacDayKeys(day: day))
    }

    func ageRestrictions(_ restrictions: AgeRestrictions) throws -> [ContentEntry] {
        try entries(ageRestrictionKeys(restrictions: restrictions))
    }

    func bazi(_ analysis: BaziAnalysis) throws -> [ContentEntry] {
        try entries(baziKeys(analysis: analysis))
    }

    func kua(_ kua: KuaInfo) throws -> ContentEntry? {
        try entry(kuaKey(kua: kua))
    }

    func eightStar(_ star: EightMansionsStar) throws -> ContentEntry? {
        try entry(eightStarKey(star: star))
    }

    func eightStarUse(_ star: EightMansionsStar, _ usage: EightUse) throws -> ContentEntry? {
        try entry(eightStarUseKey(star: star, usage: usage))
    }

    func kuaCompatibility(_ compatibility: KuaCompatibility) throws -> [ContentEntry] {
        try entries(kuaCompatibilityKeys(compatibility: compatibility))
    }

    /// A Nine Star, with the text of its phase in the period ruled by `period`.
    func nineStar(_ star: NineStar, period: NineStar) throws -> NineStarReading? {
        guard let entry = try entry(nineStarKey(star: star)) else { return nil }
        let phase = nineStarPhase(star: star, period: period)
        return NineStarReading(
            entry: entry,
            phase: phase,
            phaseName: nineStarPhaseName(value: phase),
            isAuspicious: nineStarPhaseIsAuspicious(value: phase)
        )
    }

    // MARK: Tử Vi

    func tuviPalace(chart: TuViChart, branch: Branch) throws -> PalaceReading {
        let query = tuviPalaceQuery(chart: chart, branch: branch)
        return PalaceReading(entries: try entries(query.keys), passages: try passages(for: query))
    }

    /// Section 9 of the spec: fetch by anchor, keep a passage when the facts of the lookup that
    /// found it open one of its gates, and count each passage once. Book order.
    func passages(for query: PalaceQuery) throws -> [Passage] {
        var chosen: [Int64: Passage] = [:]
        for lookup in query.lookups {
            for anchor in lookup.anchors {
                for (passage, gates) in try database.rows(passageSQL, [anchor], map: Self.passage)
                where chosen[passage.id] == nil {
                    if try gateMatches(gates: gates, facts: lookup.facts) {
                        chosen[passage.id] = passage
                    }
                }
            }
        }
        return chosen.values.sorted { $0.id < $1.id }
    }

    // MARK: Rows

    private static func entry(_ row: SQLiteRow) throws -> ContentEntry {
        let key = row.text(0) ?? ""
        let extra: EntryExtra
        if let json = row.text(6), !json.isEmpty {
            extra = try decode(EntryExtra.self, json, key: key, column: "extra")
        } else {
            extra = EntryExtra()
        }
        return ContentEntry(
            key: key,
            collection: row.text(1) ?? "",
            title: row.text(2) ?? "",
            luck: row.text(3).flatMap(ContentLuck.init(rawValue:)),
            summary: row.text(4) ?? "",
            body: try decode([String].self, row.text(5) ?? "[]", key: key, column: "body"),
            extra: extra,
            citation: row.text(7) ?? "",
            sensitivity: Sensitivity(rawValue: row.int(8))
        )
    }

    private static func passage(_ row: SQLiteRow) throws -> (Passage, String) {
        let stableID = row.text(1) ?? ""
        guard let tier = row.text(2).flatMap(PassageTier.init(rawValue:)) else {
            throw ContentRepositoryError.malformedColumn(key: stableID, column: "tier")
        }
        let passage = Passage(
            id: row.int(0),
            stableID: stableID,
            tier: tier,
            section: row.text(3).flatMap(PassageSection.init(rawValue:)),
            verse: row.text(4),
            conditionText: row.text(6),
            body: try decode([String].self, row.text(7) ?? "[]", key: stableID, column: "body"),
            citation: row.text(8) ?? "",
            sensitivity: Sensitivity(rawValue: row.int(9)),
            correction: row.text(10)
        )
        return (passage, row.text(5) ?? "[]")
    }

    private static func decode<T: Decodable>(_ type: T.Type, _ json: String, key: String, column: String) throws -> T {
        do {
            return try JSONDecoder().decode(type, from: Data(json.utf8))
        } catch {
            throw ContentRepositoryError.malformedColumn(key: key, column: column)
        }
    }
}
