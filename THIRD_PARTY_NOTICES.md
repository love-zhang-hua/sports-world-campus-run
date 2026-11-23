# 来源、版权与修改说明

## Sports-simulation 0.5.1

- 来源：https://github.com/liing6666/Sports-simulation
- Copyright (c) 2026 YanamiNeko。
- 许可证：CC BY-NC 4.0，原文见根目录 LICENSE；法律条款 https://creativecommons.org/licenses/by-nc/4.0/legalcode.zh 。
- 保留原 Rust 核心及路线规划库。修改包括：选定路线适配器、账号校区/点位读取、服务端退出、Android JNI/iOS C ABI、Android 前台模拟定位服务、地图 WebView、SwiftUI 客户端、构建和发布文档。
- 新增代码也按 CC BY-NC 4.0 提供。其他第三方组件仍按其独立许可证分发。
- Windows/Android 1.7.0 修改：本机安装/卸载、独立数据目录、匿名配速—步频参考模型、由速度反算步幅、路线DEM高程与爬升、关闭上游自动更新。Android预览与回放共用地形曲线，新增步频、步幅、步数与爬升显示。

## 地图及字体

- MapLibre GL JS 5.12.0：BSD-3-Clause，完整许可及内嵌依赖声明保留在 android-virtualrun/assets/maplibre-LICENSE.txt。版本以随附文件头为准。
- OpenFreeMap：https://openfreemap.org ，用于 Android 主界面底图；数据 © OpenStreetMap contributors，ODbL，https://www.openstreetmap.org/copyright 。
- 高德公开手机网页：https://m.amap.com 。页面和运行时地图数据归相应权利人所有；未打包离线高德底图或从官方 APK 复制地图 Key。此处不授予地图数据或 SDK 的使用权。
- Noto Sans SC：SIL Open Font License 1.1，见 assets/fonts/OFL-1.1.txt 与 SOURCE.md。
- winit 0.30.13：见 vendor/winit-0.30.13 中原许可；保留原 Android 生命周期修复。

## 分析资料与未分发附件

- Windows 地形查询：Open Topo Data，https://www.opentopodata.org/api/ ，使用其 SRTM30m 数据集，坐标为WGS84、单位为米。地图数据及公共服务的使用规则独立于本项目源码许可证；不打包或转授整个高程数据库。

- Coaixy/android_wh_physicalfitness_analyse：仅参考其高德 MapView、点位和围栏接口说明，未分发反编译源码或官方 APK。
- LocationSpoofer v3.1.6：曾分析用户提供 APK 的公开控制入口。未复制其 DEX/native 库，未将其 APK 纳入源码或 Release；当前路线回放由本项目实现。
- 登录验证码在隔离网页中由用户手动完成，服务脚本运行时从提供方加载。账号密码和 Rust 会话不注入地图或验证码页面。

仓库不包含个人登录会话、账号密码、手机截图、签名私钥或 Apple 描述文件。
