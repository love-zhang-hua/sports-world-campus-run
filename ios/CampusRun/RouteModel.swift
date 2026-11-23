import Foundation
import Combine
import CoreLocation

struct GeoPoint: Codable, Equatable {
    var lat: Double
    var lon: Double
    var pair: [Double] { [lat, lon] }
    var coordinate: CLLocationCoordinate2D { .init(latitude: lat, longitude: lon) }
    var valid: Bool { lat.isFinite && lon.isFinite && abs(lat) <= 85 && abs(lon) <= 180 }
    func distance(to other: GeoPoint) -> Double {
        CLLocation(latitude: lat, longitude: lon).distance(from: CLLocation(latitude: other.lat, longitude: other.lon))
    }
}
struct Campus: Identifiable {
    let id: Int
    let name: String
    let ring: [GeoPoint]
}
struct Checkpoint: Identifiable {
    let id: Int
    let name: String
    let point: GeoPoint
    let required: Bool
}
struct SavedRoute: Codable {
    var points: [GeoPoint] = []
    var speed: Double = 8
    var laps: Int = 1
    var closed: Bool = true
    var city: String = ""
    var sport: String = "campus"
    var seed: Int = Int.random(in: 1...Int(Int32.max))
}

@MainActor final class RouteModel: ObservableObject {
    @Published var route = SavedRoute()
    @Published var busy = false
    @Published var loggedIn = false
    @Published var accountName = ""
    @Published var message = "先登录读取点位，或直接打开地图选择路线。"
    @Published var campuses: [Campus] = []
    @Published var checkpoints: [Checkpoint] = []
    @Published var selectedCampus = 0
    @Published var showCaptcha = false
    @Published var username = ""
    @Published var password = ""
    @Published var playing = false
    @Published var paused = false
    @Published var travelled: Double = 0
    @Published var cursor: GeoPoint?
    private var timer: Timer?
    private var lastTick: Date?
    private var started = false

    init() {
        if let data = UserDefaults.standard.data(forKey: "selectedRoute"), let saved = try? JSONDecoder().decode(SavedRoute.self, from: data), saved.points.count <= 10000, saved.points.allSatisfy(\.valid), (3.6...20).contains(saved.speed), (1...100).contains(saved.laps) { route = saved }
    }
    var ring: [GeoPoint] { campuses.first(where: { $0.id == selectedCampus })?.ring ?? [] }
    var legs: [GeoPoint] {
        guard route.closed, let first = route.points.first, route.points.last != first else { return route.points }
        return route.points + [first]
    }
    var lapLength: Double { zip(legs, legs.dropFirst()).reduce(0) { $0 + $1.0.distance(to: $1.1) } }
    var distance: Double { lapLength * Double(route.closed ? route.laps : 1) }
    var minutes: Double { distance / (route.speed / 3.6) / 60 }
    var validRoute: Bool { route.points.count >= 2 && route.points.count <= 10000 && route.points.allSatisfy(\.valid) && lapLength > 0 }
    var payload: [String: Any] { ["points": route.points.map(\.pair), "speed": route.speed, "laps": route.laps, "loop": route.closed, "city": route.city, "sport": route.sport, "seed": route.seed] }
    func save() { if let data = try? JSONEncoder().encode(route) { UserDefaults.standard.set(data, forKey: "selectedRoute") } }
    func setPoints(_ points: [GeoPoint]) {
        guard !playing, points.count <= 10000, points.allSatisfy(\.valid) else { return }
        route.points = points; route.seed = Int.random(in: 1...Int(Int32.max)); save()
    }
    func startSession() async {
        guard !started else { return }; started = true
        await perform("session")
    }
    func login(captcha: [String: Any]? = nil) async {
        guard !username.trimmingCharacters(in: .whitespaces).isEmpty, !password.isEmpty else { message = "请填写账号和密码"; return }
        var value: [String: Any] = ["username": username.trimmingCharacters(in: .whitespaces), "password": password]
        if let captcha { value["captcha"] = captcha }
        await perform("login", value)
    }
    func refreshMap() async { guard loggedIn else { return }; await perform("mapData", ["campus": selectedCampus]) }
    func logout() async { stop(); await perform("logout") }
    func preview() async { save(); await perform("preview", payload) }
    func submit() async { save(); await perform("submit", payload) }
    private func perform(_ op: String, _ payload: [String: Any] = [:]) async {
        guard !busy else { return }; busy = true
        message = ["login": "正在登录…", "logout": "正在请求服务端退出…", "mapData": "正在读取校区与必经点…", "submit": "正在提交，请勿重复操作…", "preview": "正在生成轨迹…"][op] ?? "正在读取会话…"
        do {
            let data = try await CoreClient.shared.call(op, payload)
            if let state = data["logged_in"] as? Bool {
                loggedIn = state; accountName = data["name"] as? String ?? "已登录"
                if state { password = "" } else { campuses = []; checkpoints = []; username = ""; password = "" }
            }
            if op == "mapData" {
                campuses = (data["campuses"] as? [[String: Any]] ?? []).compactMap { v in
                    guard let id = v["index"] as? Int else { return nil }
                    let ring = (v["ring"] as? [[Double]] ?? []).compactMap { q -> GeoPoint? in
                        guard q.count == 2 else { return nil }; let p = GeoPoint(lat: q[0], lon: q[1]); return p.valid ? p : nil
                    }
                    return Campus(id: id, name: v["name"] as? String ?? "校区", ring: ring)
                }
                selectedCampus = data["selected_campus"] as? Int ?? 0
                checkpoints = (data["checkpoints"] as? [[String: Any]] ?? []).enumerated().compactMap { i, v in
                    guard let lat = v["lat"] as? Double, let lon = v["lon"] as? Double else { return nil }
                    let point = GeoPoint(lat: lat, lon: lon); guard point.valid else { return nil }
                    return Checkpoint(id: i, name: v["name"] as? String ?? "点位", point: point, required: v["required"] as? Bool ?? false)
                }
                message = "已读取 \(checkpoints.count) 个点位，\(checkpoints.filter(\.required).count) 个必经点。" + (data["warnings"] as? [String] ?? []).joined(separator: "；")
            } else if op == "preview" {
                message = String(format: "轨迹已生成：%.0f 米，%.1f 分钟。尚未提交。", data["distance_m"] as? Double ?? 0, (data["duration_s"] as? Double ?? 0) / 60)
            } else {
                message = data["message"] as? String ?? (loggedIn ? "已登录" : "尚未登录")
                if op == "submit" { message += "\n" + String(describing: data["record_id"] ?? "未返回记录编号") }
            }
            busy = false
            if (op == "login" || op == "session") && loggedIn { await refreshMap() }
        } catch {
            busy = false; message = error.localizedDescription
            if op == "login" && message.contains("CAPTCHA_REQUIRED") { showCaptcha = true; message = "请手动完成登录验证" }
        }
    }
    func play() {
        guard validRoute, !playing else { return }
        save(); travelled = 0; cursor = route.points.first; paused = false; playing = true; lastTick = Date()
        timer = Timer.scheduledTimer(withTimeInterval: 0.2, repeats: true) { [weak self] _ in
            Task { @MainActor in self?.tick() }
        }
    }
    func pause() { paused.toggle(); lastTick = Date() }
    func stop() { timer?.invalidate(); timer = nil; playing = false; paused = false; lastTick = nil }
    func background() { if playing { paused = true; lastTick = nil } }
    private func tick() {
        let now = Date(); defer { lastTick = now }
        guard playing, !paused, let previous = lastTick else { return }
        travelled = min(distance, travelled + min(1, now.timeIntervalSince(previous)) * route.speed / 3.6)
        var remain = travelled >= distance ? lapLength : travelled.truncatingRemainder(dividingBy: lapLength)
        for (a, b) in zip(legs, legs.dropFirst()) {
            let d = a.distance(to: b)
            if remain <= d && d > 0 {
                let ratio = remain / d
                var delta = b.lon - a.lon
                if delta > 180 { delta -= 360 }; if delta < -180 { delta += 360 }
                var lon = a.lon + ratio * delta
                if lon > 180 { lon -= 360 }; if lon < -180 { lon += 360 }
                cursor = GeoPoint(lat: a.lat + ratio * (b.lat - a.lat), lon: lon); break
            }; remain -= d
        }
        if travelled >= distance { stop(); message = "应用内回放完成。未更改系统定位。" }
    }
}
