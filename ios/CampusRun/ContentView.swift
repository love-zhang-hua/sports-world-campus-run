import SwiftUI

struct ContentView: View {
    @ObservedObject var model: RouteModel
    @State private var showMap = false
    @State private var mapEditable = true
    @State private var confirmSubmit = false
    @State private var confirmClear = false
    @State private var accountExpanded = true
    var body: some View {
        NavigationStack {
            Form {
                Section {
                    DisclosureGroup(model.loggedIn ? "运动世界 · \(model.accountName)" : "运动世界账号", isExpanded: $accountExpanded) {
                        if model.loggedIn {
                            Button("退出账号", role: .destructive) { Task { await model.logout() } }.disabled(model.busy)
                        } else {
                            TextField("账号", text: $model.username).textContentType(.username).keyboardType(.phonePad).autocorrectionDisabled()
                            SecureField("密码", text: $model.password).textContentType(.password)
                            Button("登录") { Task { await model.login(); if model.loggedIn { accountExpanded = false } } }.disabled(model.busy)
                        }
                    }
                }
                Section("地图与路线") {
                    if !model.campuses.isEmpty {
                        Picker("校区", selection: $model.selectedCampus) {
                            ForEach(model.campuses) { campus in Text(campus.name).tag(campus.id) }
                        }.disabled(model.busy || model.playing)
                        .onChange(of: model.selectedCampus) { _ in Task { await model.refreshMap() } }
                    }
                    Button { mapEditable = !model.playing; showMap = true } label: {
                        Label(model.playing ? "查看实时地图回放" : "打开实时地图选路线", systemImage: "map.fill").font(.headline).padding(.vertical, 8)
                    }.disabled(model.busy)
                    HStack {
                        Text("\(model.route.points.count) 个途经点")
                        Spacer()
                        Text("\(model.checkpoints.count) 点 · \(model.checkpoints.filter(\.required).count) 必经")
                    }.font(.footnote).foregroundStyle(.secondary)
                    if model.loggedIn { Button("刷新校区点位") { Task { await model.refreshMap() } }.disabled(model.busy || model.playing) }
                    if !model.checkpoints.isEmpty {
                        DisclosureGroup("查看点位") {
                            ForEach(model.checkpoints) { p in
                                Label(p.name, systemImage: p.required ? "flag.fill" : "mappin.circle").foregroundStyle(p.required ? Color.red : Color.orange)
                            }
                        }
                    }
                    if !model.route.points.isEmpty {
                        HStack {
                            Button("撤销末点") { model.setPoints(Array(model.route.points.dropLast())) }
                            Spacer()
                            Button("清空", role: .destructive) { confirmClear = true }
                        }.disabled(model.playing || model.busy)
                    }
                }
                Section("跑步设置") {
                    TextField("城市，例如徐州市", text: $model.route.city)
                    Picker("项目", selection: $model.route.sport) { Text("校园跑").tag("campus"); Text("户外跑").tag("outdoor") }
                    HStack { Text("速度"); Spacer(); Text(String(format: "%.1f km/h", model.route.speed)).monospacedDigit() }
                    Slider(value: $model.route.speed, in: 3.6...20, step: 0.1)
                    Toggle("闭合路线", isOn: $model.route.closed)
                    if model.route.closed { Stepper("\(model.route.laps) 圈", value: $model.route.laps, in: 1...100) }
                    HStack { Text(String(format: "%.0f 米", model.distance)); Spacer(); Text(String(format: "约 %.1f 分钟", model.minutes)) }.font(.headline)
                }.disabled(model.playing || model.busy)
                Section("应用内回放") {
                    if model.playing || model.travelled > 0 {
                        ProgressView(value: model.travelled, total: max(1, model.distance))
                        Text(String(format: "%.0f / %.0f 米", model.travelled, model.distance)).monospacedDigit()
                    }
                    HStack {
                        if model.playing {
                            Button(model.paused ? "继续" : "暂停") { model.pause() }
                            Spacer(); Button("停止") { model.stop() }
                        } else { Button("开始回放") { model.play() }.disabled(!model.validRoute || model.busy) }
                    }
                    Text("iOS 只在本应用地图内模拟，不能修改其他应用的系统定位；切到后台会暂停。").font(.footnote).foregroundStyle(.secondary)
                }
                Section {
                    Button("生成并检查轨迹") { Task { await model.preview() } }.disabled(!model.validRoute || model.busy || model.playing)
                    Button("提交选定路线") { confirmSubmit = true }.disabled(!model.validRoute || !model.loggedIn || model.busy || model.playing || model.route.city.trimmingCharacters(in: .whitespaces).isEmpty)
                }
                Section {
                    if model.busy { ProgressView() }
                    Text(model.message).font(.footnote).textSelection(.enabled)
                    Text("iOS 初版 · 未做登录或真机测试").font(.caption).foregroundStyle(.secondary)
                }
            }
            .navigationTitle("运动世界校园跑")
            .sheet(isPresented: $showMap) { RouteMapScreen(model: model, editable: mapEditable) }
            .sheet(isPresented: $model.showCaptcha) {
                CaptchaView(complete: { proof in model.showCaptcha = false; Task { await model.login(captcha: proof) } }, cancel: { model.showCaptcha = false; model.message = "验证已取消，尚未登录" })
            }
            .confirmationDialog("提交选定路线到运动世界？", isPresented: $confirmSubmit, titleVisibility: .visible) {
                Button("确认提交") { Task { await model.submit() } }
                Button("取消", role: .cancel) {}
            } message: { Text(String(format: "%@ · %.0f 米 · %.1f 分钟。将调用现有核心提交，结果以服务端为准。", model.route.city, model.distance, model.minutes)) }
            .confirmationDialog("清空当前路线？", isPresented: $confirmClear, titleVisibility: .visible) {
                Button("清空", role: .destructive) { model.setPoints([]) }
            }
            .onDisappear { model.save() }
        }
    }
}
