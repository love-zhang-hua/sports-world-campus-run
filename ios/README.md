# iOS 初版

SwiftUI + WKWebView + 共享 Rust 静态库，最低 iOS 16。功能源码已实现并通过 Mac/Xcode 编译；未做账号登录、真实提交、模拟器运行或手机测试。

## 在 Mac 上构建

安装 Xcode（含 iOS SDK）、Rust stable、XcodeGen。首次命令：

```bash
brew install xcodegen
rustup target add aarch64-apple-ios
cd ios
xcodegen generate
open CampusRun.xcodeproj
```

在 Xcode 的 Signing & Capabilities 选择自己的 Team；必要时改成自己的唯一 Bundle Identifier。选择连接的 iPhone 并 Build/Run。系统要求时在手机开启开发者模式。免费签名的有效期和安装限制由 Apple 决定。

工程构建阶段调用 `build-core.sh`，从仓库根目录编译 Rust `route-ios`，无默认 GUI 特性。原生头文件 `CampusRun/Bridge.h` 声明 `route_call` / `route_free`。返回 C 字符串由 Swift 读取后释放，调用串行执行。

仅编译、不运行测试：

```bash
xcodebuild -project CampusRun.xcodeproj -scheme CampusRun -configuration Release \
  -destination 'generic/platform=iOS' -derivedDataPath build \
  CODE_SIGNING_ALLOWED=NO CODE_SIGNING_REQUIRED=NO build
```

GitHub Actions 的 `Build iOS (no tests)` 工作流提供相同的未签名构建。Release 另提供由该构建封装的 `CampusRun-iOS-1.0.0-unsigned.ipa`，包内为 `Payload/CampusRun.app`；它未签名，必须用自己的 Apple 签名后才能安装。正式归档、签名、分发和出口合规配置由发行者完成；本仓库不包含证书或描述文件。

## 与 Android 的差别

- iOS 不提供 Android Mock Location 对等接口。本版仅在自己的地图内回放轨迹，不改变系统位置，不影响其他 App。
- 切后台立即暂停回放；回到前台手动继续。
- 账号、校区点位、轨迹预览与确认后的提交共用原 Rust 核心；接口/账号兼容性未经 iOS 实测。
- 密码只在界面内存中短暂保留，成功登录后清空。会话在 Application Support/CampusRun，启用系统文件保护并排除 iCloud 备份。
- 服务端退出成功才删除本地会话；退出失败保留会话以便重试。
- 地图从高德公开网页加载，桥接仅传递坐标。不是官方 SDK 接入；页面变化可能导致无法选点。
- 没有定位权限申请，因为本版不需要读取系统 GPS。首次默认中心用于看图；登录后的校区边界优先决定地图中心。
