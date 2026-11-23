# 运动世界校园跑

基于 [liing6666/Sports-simulation](https://github.com/liing6666/Sports-simulation) 0.5.1 的衍生项目，原作者 **YanamiNeko**。提供 Windows 桌面版、Android 路线助手和 SwiftUI iOS 源码，共用原项目 Rust 协议与轨迹核心。**本项目不是运动世界官方客户端。**

## Windows 电脑版 1.7.0

下载 [SportsRoute-1.7.0-Setup.exe](https://github.com/love-zhang-hua/sports-world-campus-run/releases/latest/download/SportsRoute-1.7.0-Setup.exe)，适用于 Windows 10/11 64位；按向导安装后从桌面打开。无需另外安装 Python、Rust 或 Android 模拟器。[安装、卸载和参数说明](docs/WINDOWS.md)。

本版合并两组匿名跑步样本，按配速适配参考步频、步幅与步数，并可从路线地形高程计算累计爬升。模型及误差范围见说明。Windows 版不能替代 Android 系统的模拟定位提供者；Android 1.7.0 同步配速参考和路线地形功能。

## Android：一个 APK 即可安装

**[直接下载 SportsRoute-1.7.0.apk](https://github.com/love-zhang-hua/sports-world-campus-run/releases/latest/download/SportsRoute-1.7.0.apk)** · **[查看完整安装与使用教程](docs/ANDROID_GUIDE.md)**

只安装 `SportsRoute-1.7.0.apk`，约 2.3 MiB，支持 Android 8.0+ / ARM64。登录、地图、路线核心和定位服务均已内置，无需另装 LocationSpoofer、原版程序、测试包或编程环境；不需要 Root 或地图 Key。安装后的桌面名称为 **运动路线助手**。

1. 手机下载 APK → 打开文件 → 系统要求时允许当前浏览器/文件管理器安装应用 → 安装。
2. 打开“运动路线助手” → 登录自己的账号 → 等待校区和点位 → “打开实时地图选路线” → “使用路线”。
3. 设置城市、速度、圈数和项目 → “检查轨迹” → 需要提交时点击“提交路线”并确认。
4. 如需虚拟定位回放，先允许精确定位，并在开发者选项中选择“运动路线助手”为模拟位置应用，再点击“开始”。
5. 使用结束先停止回放，再点击“退出登录”，等待服务端确认退出。

手机直接下载安装不需要 USB 调试或 USB 安装权限。仅使用登录与地图功能不需要模拟位置设置；源码 ZIP 和 iOS IPA 都不是 Android 安装所需文件。

## 其他下载与版本说明

- **Android 预编译 APK**：本仓库 Releases 中的 `SportsRoute-1.7.0.apk`，Android 8.0+、ARM64（arm64-v8a）。沿用既有签名，可覆盖同签名旧版。发布页附 SHA-256。
- **iOS**：保留 SwiftUI 源码和构建说明，本次未编译或运行，不提供 iOS 安装包。

## 核心功能

| 功能 | Android | iOS 初版 |
|---|---|---|
| 登录、手动验证码、服务端退出 | 已接入 | 共用 Rust 核心，未实测 |
| 校区边界、普通点、必经点 | 从账号接口读取 | 相同接口，未实测 |
| 地图选点、闭合路线、速度和圈数 | 支持 | 已实现，未实测 |
| 生成轨迹、确认后提交选定路线 | 共用上游核心 | 共用上游核心，未实测 |
| 系统模拟定位 | Android 开发者选项需指定本应用 | **不支持**；仅应用内地图回放 |
| 后台回放 | 前台定位服务 | 切到后台暂停 |

路线点之间按直线连接，不自动绕路。选路线时沿道路多选点。坐标与 GPX 约定 WGS84，高德画面使用 GCJ-02，进入/离开地图时转换。

## 地图来源

登录接口只返回校区边界和点位，**不会返回整张官方地图**。原官方客户端使用高德原生 SDK；本项目的“实时地图”打开高德公开手机网页，再叠加账号点位和自选路线，无需填写地图 Key。它是同供应商的公开地图页面，**不是官方 App 地图窗口，也不是正式授权的 SDK 集成**。

网页结构、网络或高德访问策略变化会使加载/选点失效。不要把“有点位”或“SDK 对象存在”当作地图加载成功。Android 主界面仍保留 OpenFreeMap 预览，点击“打开实时地图选路线”进入高德页面。

## 使用

1. 登录自己的账号，按提示手动完成验证码，等待校区和点位返回。
2. 选择校区，打开实时地图，按顺序轻点途经点，再选择“使用路线”。必经点用红色标记。
3. 设置城市、速度、圈数，先生成并检查轨迹；提交操作有单独确认。
4. Android 的“定位回放”与“提交路线”是两项独立操作。iOS 回放只移动本应用地图上的位置，不改变其他应用定位。
5. 使用结束选择“退出账号”。退出失败时保留本地会话供重试；不要用卸载代替服务端退出。

服务端的时间段、账号会话和验证要求仍可能阻止操作。未做成功提交保证。项目不提供学校或平台授权，请遵守相应规则。

## 构建

- [iOS 构建与签名](ios/README.md)
- [Android 构建](android-virtualrun/README.md)
- [发布说明与已知限制](docs/RELEASE.md)
- [第三方来源及许可](THIRD_PARTY_NOTICES.md)

目录：`src/` 为共享 Rust 核心，`android-virtualrun/` 为 Android 界面与定位服务，`ios/` 为 SwiftUI 客户端，`crates/route-planner/` 为原路线规划库。

## 致谢与许可

- **[liing6666/Sports-simulation](https://github.com/liing6666/Sports-simulation)**，原作者 **YanamiNeko**：登录协议、校区点位、轨迹生成、提交流程及路线算法。此项目是在其源码基础上的衍生实现，不只是概念参考。原 [LICENSE](LICENSE) 保留，原 README 见 [docs/UPSTREAM_README.md](docs/UPSTREAM_README.md)。
- [Coaixy/android_wh_physicalfitness_analyse](https://github.com/Coaixy/android_wh_physicalfitness_analyse)：用于核对地图组件和接口定义的研究资料，未将该仓库反编译文件打包到发行物。
- MapLibre GL JS、OpenFreeMap、OpenStreetMap、Noto Sans SC 和 winit：许可见第三方说明及各目录原始许可文件。

本项目源码依上游 **CC BY-NC 4.0（署名—非商业性使用）**公开，包括新增的 iOS 和 Android 代码；第三方组件遵守各自许可。须保留署名、原链接和修改说明；禁止商业使用。该非商业许可**不是 OSI 认可的开源许可证**。
