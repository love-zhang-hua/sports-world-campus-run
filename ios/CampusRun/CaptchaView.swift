import SwiftUI
import WebKit

struct CaptchaView: UIViewRepresentable {
    let complete: ([String: Any]) -> Void
    let cancel: () -> Void
    func makeCoordinator() -> Coordinator { Coordinator(self) }
    func makeUIView(context: Context) -> WKWebView {
        let config = WKWebViewConfiguration(); config.websiteDataStore = .nonPersistent()
        config.userContentController.add(context.coordinator, name: "humanCaptcha")
        let shim = "window.HumanCaptcha={cancel:()=>webkit.messageHandlers.humanCaptcha.postMessage({type:'cancel'}),complete:value=>webkit.messageHandlers.humanCaptcha.postMessage({type:'complete',value})};"
        config.userContentController.addUserScript(WKUserScript(source: shim, injectionTime: .atDocumentStart, forMainFrameOnly: true))
        let web = WKWebView(frame: .zero, configuration: config); web.navigationDelegate = context.coordinator
        if let resource = Bundle.main.url(forResource: "captcha", withExtension: "html"), let html = try? String(contentsOf: resource, encoding: .utf8) {
            web.loadHTMLString(html, baseURL: URL(string: "https://captcha.local/"))
        }
        return web
    }
    func updateUIView(_ uiView: WKWebView, context: Context) {}
    static func dismantleUIView(_ web: WKWebView, coordinator: Coordinator) {
        web.stopLoading(); web.navigationDelegate = nil; web.configuration.userContentController.removeScriptMessageHandler(forName: "humanCaptcha")
    }
    final class Coordinator: NSObject, WKScriptMessageHandler, WKNavigationDelegate {
        let parent: CaptchaView
        init(_ parent: CaptchaView) { self.parent = parent }
        func userContentController(_ controller: WKUserContentController, didReceive message: WKScriptMessage) {
            guard message.frameInfo.isMainFrame, let body = message.body as? [String: Any] else { return }
            if body["type"] as? String == "cancel" { parent.cancel(); return }
            guard body["type"] as? String == "complete", let raw = body["value"] as? String, raw.utf8.count < 100000, let data = raw.data(using: .utf8), let proof = try? JSONSerialization.jsonObject(with: data) as? [String: Any] else { return }
            guard ["lot_number", "captcha_output", "pass_token", "gen_time"].allSatisfy({ key in
                guard let value = proof[key] as? String else { return false }; return !value.isEmpty && value.count <= 16384
            }) else { return }
            parent.complete(proof)
        }
        func webView(_ webView: WKWebView, decidePolicyFor action: WKNavigationAction, decisionHandler: @escaping (WKNavigationActionPolicy) -> Void) {
            if action.targetFrame?.isMainFrame == true {
                decisionHandler(action.request.url?.absoluteString == "about:blank" || action.request.url?.host == "captcha.local" ? .allow : .cancel)
            } else { decisionHandler(action.request.url?.scheme == "https" ? .allow : .cancel) }
        }
    }
}
