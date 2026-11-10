import Foundation
import SQLite3

enum SQLiteError: Error, Equatable, CustomStringConvertible {
    case open(path: String, message: String)
    case prepare(sql: String, message: String)
    case step(message: String)

    var description: String {
        switch self {
        case .open(let path, let message): "Không mở được \(path): \(message)"
        case .prepare(let sql, let message): "Câu SQL lỗi (\(sql)): \(message)"
        case .step(let message): "Đọc SQLite lỗi: \(message)"
        }
    }
}

/// One row of a result, read by column index.
struct SQLiteRow {
    fileprivate let statement: OpaquePointer

    func text(_ column: Int32) -> String? {
        guard let pointer = sqlite3_column_text(statement, column) else { return nil }
        return String(cString: pointer)
    }

    func int(_ column: Int32) -> Int64 {
        sqlite3_column_int64(statement, column)
    }
}

/// A read-only connection to a file that never changes. Opened with `immutable=1` (section 2.5
/// of INTERPRETATION_DB_SPEC.md), so SQLite takes no locks. Prepared statements are cached and
/// every use holds the lock, which makes the connection safe to share.
final class SQLiteDatabase: @unchecked Sendable {
    private let handle: OpaquePointer
    private let lock = NSLock()
    private var statements: [String: OpaquePointer] = [:]

    init(readOnlyAt url: URL) throws {
        var handle: OpaquePointer?
        let uri = url.absoluteString + "?immutable=1"
        let flags = SQLITE_OPEN_READONLY | SQLITE_OPEN_URI | SQLITE_OPEN_FULLMUTEX
        let status = sqlite3_open_v2(uri, &handle, flags, nil)
        guard status == SQLITE_OK, let handle else {
            let message = handle.map { String(cString: sqlite3_errmsg($0)) }
                ?? String(cString: sqlite3_errstr(status))
            sqlite3_close_v2(handle)
            throw SQLiteError.open(path: url.path, message: message)
        }
        self.handle = handle
    }

    deinit {
        for statement in statements.values {
            sqlite3_finalize(statement)
        }
        sqlite3_close_v2(handle)
    }

    /// Runs `sql` with text `arguments` bound in order and maps every row.
    func rows<T>(_ sql: String, _ arguments: [String] = [], map: (SQLiteRow) throws -> T) throws -> [T] {
        lock.lock()
        defer { lock.unlock() }
        let statement = try prepared(sql)
        defer {
            sqlite3_reset(statement)
            sqlite3_clear_bindings(statement)
        }
        for (index, argument) in arguments.enumerated() {
            sqlite3_bind_text(statement, Int32(index + 1), argument, -1, SQLITE_TRANSIENT)
        }
        var result: [T] = []
        while true {
            switch sqlite3_step(statement) {
            case SQLITE_ROW:
                result.append(try map(SQLiteRow(statement: statement)))
            case SQLITE_DONE:
                return result
            default:
                throw SQLiteError.step(message: String(cString: sqlite3_errmsg(handle)))
            }
        }
    }

    /// The value of a pragma that returns one integer, such as `user_version`.
    func pragma(_ name: String) throws -> Int64 {
        try rows("PRAGMA \(name)") { $0.int(0) }.first ?? 0
    }

    private func prepared(_ sql: String) throws -> OpaquePointer {
        if let statement = statements[sql] {
            return statement
        }
        var statement: OpaquePointer?
        guard sqlite3_prepare_v3(handle, sql, -1, UInt32(SQLITE_PREPARE_PERSISTENT), &statement, nil) == SQLITE_OK,
              let statement
        else {
            throw SQLiteError.prepare(sql: sql, message: String(cString: sqlite3_errmsg(handle)))
        }
        statements[sql] = statement
        return statement
    }
}

/// SQLite copies a bound value given this destructor; the C macro is not imported into Swift.
private let SQLITE_TRANSIENT = unsafeBitCast(-1, to: sqlite3_destructor_type.self)
