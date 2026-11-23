import Foundation
import UIKit

/// All native calls share a serial queue. No credentials are sent to either web view.
final class CoreClient {
    static let shared = CoreClient()
    private let queue = DispatchQueue(label: "campusrun.core", qos: .userInitiated)
    private init() {}

    func call(_ operation: String, _ payload: [String: Any]) async throws -> [String: Any] {
        let raw = try JSONSerialization.data(withJSONObject: payload)
        let input = String(decoding: raw, as: UTF8.self)
        return try await withCheckedThrowingContinuation { continuation in
            queue.async {
                do {
                    let manager = FileManager.default
                    var folder = try manager.url(for: .applicationSupportDirectory, in: .userDomainMask, appropriateFor: nil, create: true).appendingPathComponent("CampusRun", isDirectory: true)
                    try manager.createDirectory(at: folder, withIntermediateDirectories: true, attributes: [.protectionKey: FileProtectionType.completeUntilFirstUserAuthentication])
                    var values = URLResourceValues(); values.isExcludedFromBackup = true
                    try folder.setResourceValues(values)
                    let pointer = folder.path.withCString { path in
                        operation.withCString { op in input.withCString { route_call(path, op, $0) } }
                    }
                    guard let pointer else { throw CoreError.message("核心未返回结果") }
                    defer { route_free(pointer) }
                    let response = Data(String(cString: pointer).utf8)
                    guard let envelope = try JSONSerialization.jsonObject(with: response) as? [String: Any] else { throw CoreError.message("核心返回格式无效") }
                    guard envelope["ok"] as? Bool == true else { throw CoreError.message(envelope["error"] as? String ?? "操作失败") }
                    for file in (try? manager.contentsOfDirectory(at: folder, includingPropertiesForKeys: nil)) ?? [] {
                        try? manager.setAttributes([.protectionKey: FileProtectionType.completeUntilFirstUserAuthentication], ofItemAtPath: file.path)
                    }
                    continuation.resume(returning: envelope["data"] as? [String: Any] ?? [:])
                } catch { continuation.resume(throwing: error) }
            }
        }
    }
}
enum CoreError: LocalizedError {
    case message(String)
    var errorDescription: String? { if case .message(let value) = self { return value }; return nil }
}
