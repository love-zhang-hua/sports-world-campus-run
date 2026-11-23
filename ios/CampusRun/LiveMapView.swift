import SwiftUI
import WebKit

struct RouteMapScreen: View {
    @Environment(\.dismiss) private var dismiss
    @ObservedObject var model: RouteModel
    @State private var selection: [GeoPoint] = []
    @State private var ready = false
    @State private var status = "正在加载高德公开实时地图…"
    let editable: Bool
    var body: some View {
        NavigationStack {
            VStack(spacing: 0) {
                Text(status).font(.footnote).frame(maxWidth: .infinity, alignment: .leading).padding(12)
                LiveMapView(model: model, editable: editable, changed: { selection = $0; status = "已选 \($0.count) 个途经点" }, state: { ready = $0; status = $1 })
            }
            .navigationTitle(editable ? "选择路线" : "路线回放")
            .navigationBarTitleDisplayMode(.inline)
            .toolbar {
                ToolbarItem(placement: .cancellationAction) { Button("返回") { dismiss() } }
                ToolbarItem(placement: .confirmationAction) {
                    if editable { Button("使用路线") { model.setPoints(selection); dismiss() }.disabled(!ready || selection.count < 2) }
                }
            }
        }
    }
}
struct LiveMapView: UIViewRepresentable {
    @ObservedObject var model: RouteModel
    let editable: Bool
    let changed: ([GeoPoint]) -> Void
    let state: (Bool, String) -> Void
    func makeCoordinator() -> Coordinator { Coordinator(self) }
    func makeUIView(context: Context) -> WKWebView {
        let configuration = WKWebViewConfiguration()
        configuration.websiteDataStore = .nonPersistent()
        configuration.userContentController.add(context.coordinator, name: "routeCoordinates")
        let web = WKWebView(frame: .zero, configuration: configuration)
        web.navigationDelegate = context.coordinator
        web.allowsBackForwardNavigationGestures = false
        web.isOpaque = false
        context.coordinator.web = web
        context.coordinator.load()
        return web
    }
    func updateUIView(_ web: WKWebView, context: Context) {
        context.coordinator.parent = self
        if let point = model.cursor, let data = try? JSONSerialization.data(withJSONObject: Coordinates.mapPair(point)) {
            web.evaluateJavaScript("window.__routePlayback && window.__routePlayback(\(String(decoding: data, as: UTF8.self)))", completionHandler: nil)
        }
    }
    static func dismantleUIView(_ web: WKWebView, coordinator: Coordinator) {
        coordinator.task?.cancel(); web.stopLoading(); web.navigationDelegate = nil
        web.configuration.userContentController.removeScriptMessageHandler(forName: "routeCoordinates")
    }
    final class Coordinator: NSObject, WKNavigationDelegate, WKScriptMessageHandler {
        var parent: LiveMapView
        weak var web: WKWebView?
        var task: Task<Void, Never>?
        init(_ parent: LiveMapView) { self.parent = parent }
        @MainActor func load() {
            let ring = parent.model.ring, points = parent.model.route.points
            let center: GeoPoint
            if !ring.isEmpty { center = GeoPoint(lat: ring.map(\.lat).reduce(0,+)/Double(ring.count), lon: ring.map(\.lon).reduce(0,+)/Double(ring.count)) }
            else { center = points.first ?? GeoPoint(lat: 34.16661610535, lon: 117.11362908317) }
            let c = Coordinates.gcj(center)
            let url = URL(string: "https://m.amap.com/share/index/lnglat=\(c.lon),\(c.lat)&name=Campus&src=uriapi&callnative=0&innersrc=uriapi")!
            let config: [String: Any] = ["points": points.map(Coordinates.mapPair), "ring": ring.map(Coordinates.mapPair), "center": [c.lon,c.lat], "loop": parent.model.route.closed, "editable": parent.editable, "checkpoints": parent.model.checkpoints.map { ["position": Coordinates.mapPair($0.point), "required": $0.required, "name": $0.name] as [String: Any] }]
            task = Task { @MainActor [weak self] in
                guard let self else { return }
                do {
                    let ua = (try? await self.web?.evaluateJavaScript("navigator.userAgent")) as? String
                    var request = URLRequest(url: url); request.timeoutInterval = 25
                    request.setValue(ua ?? "Mozilla/5.0 (iPhone; CPU iPhone OS 16_0 like Mac OS X) AppleWebKit/605.1.15 Version/16.0 Mobile Safari/604.1", forHTTPHeaderField: "User-Agent")
                    let session = URLSession(configuration: .ephemeral)
                    defer { session.finishTasksAndInvalidate() }
                    let (data, response) = try await session.data(for: request)
                    guard !Task.isCancelled else { return }
                    guard let http = response as? HTTPURLResponse, http.statusCode == 200, response.url?.scheme == "https", response.url?.host == "m.amap.com", data.count <= 6_000_000, var html = String(data: data, encoding: .utf8) else { throw CoreError.message("实时地图页面加载失败") }
                    let expression = try NSRegularExpression(pattern: #"<script[^>]*src=["'][^"']*webapi[.]amap[.]com/maps[^>]*></script>"#, options: .caseInsensitive)
                    guard let match = expression.firstMatch(in: html, range: NSRange(html.startIndex..., in: html)), let range = Range(match.range, in: html), let resource = Bundle.main.url(forResource: "live-map", withExtension: "js") else { throw CoreError.message("公开地图页面已变化，暂不能选点") }
                    let json = String(decoding: try JSONSerialization.data(withJSONObject: config), as: UTF8.self).replacingOccurrences(of: "<", with: "\\u003c")
                    let bridge = "window.RouteCoordinates={ready:()=>webkit.messageHandlers.routeCoordinates.postMessage({type:'ready'}),update:s=>webkit.messageHandlers.routeCoordinates.postMessage({type:'update',value:s}),failed:()=>webkit.messageHandlers.routeCoordinates.postMessage({type:'failed'})};"
                    let script = bridge + "window.__routeConfig=" + json + ";" + (try String(contentsOf: resource, encoding: .utf8))
                    html.insert(contentsOf: "<script>" + script.replacingOccurrences(of: "</script", with: "<\\/script") + "</script>", at: range.upperBound)
                    self.web?.loadHTMLString(html, baseURL: url)
                } catch { if !Task.isCancelled { self.parent.state(false, error.localizedDescription) } }
            }
        }
        func userContentController(_ userContentController: WKUserContentController, didReceive message: WKScriptMessage) {
            guard message.frameInfo.isMainFrame, message.webView === web, let body = message.body as? [String: Any], let type = body["type"] as? String else { return }
            switch type {
            case "ready": parent.state(true, parent.editable ? "轻点选路线；双指缩放。底图由高德公开页面提供。" : "仅在本应用内回放，不修改系统定位。")
            case "failed": parent.state(false, "实时地图未完成初始化，请返回后重试。")
            case "update":
                guard let raw = body["value"] as? String, raw.utf8.count <= 1_500_000, let data = raw.data(using: .utf8), let array = try? JSONSerialization.jsonObject(with: data) as? [[Double]], array.count <= 10000 else { return }
                var points: [GeoPoint] = []
                for pair in array {
                    guard pair.count == 2 else { return }
                    let point = GeoPoint(lat: pair[1], lon: pair[0]); guard point.valid else { return }
                    points.append(Coordinates.wgs(point))
                }
                parent.changed(points)
            default: break
            }
        }
        func webView(_ webView: WKWebView, decidePolicyFor navigationAction: WKNavigationAction, decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
            guard let url = navigationAction.request.url else { decisionHandler(.cancel); return }
            let allowed = url.absoluteString == "about:blank" || (url.scheme == "https" && ["m.amap.com", "uri.amap.com", "ditu.amap.com"].contains(url.host ?? ""))
            decisionHandler(allowed ? .allow : .cancel)
        }
        func webView(_ webView: WKWebView, didFailProvisionalNavigation navigation: WKNavigation!, withError error: Error) { parent.state(false, error.localizedDescription) }
        func webView(_ webView: WKWebView, didFail navigation: WKNavigation!, withError error: Error) { parent.state(false, error.localizedDescription) }
    }
}
