# Android 运动路线助手

**手机用户：从本仓库 Releases 下载 `SportsRoute-1.7.0.apk` → [安装与使用教程](../docs/ANDROID_GUIDE.md)。**

安装包内已包含账号登录、地图界面、路线核心和定位服务，不需要另装 LocationSpoofer 或下面的开发工具。下载后直接打开 APK，按系统提示安装，桌面名称为“运动路线助手”。手机直接安装不需要 USB 调试或 USB 安装权限；只有虚拟定位回放需要设置模拟位置应用。

Release 提供正式版 `SportsRoute-1.7.0.apk`，Android 8.0+ / ARM64。沿用既有签名，私钥不入库。

登录/验证码/点位/轨迹核心由 Rust `route-app` 提供，Java 界面负责地图和本机模拟定位。定位回放前需在开发者选项中将“运动路线助手”设为模拟位置信息应用，并授予精确定位与通知权限。

主界面 OpenFreeMap 预览与单独的高德实时地图并存。实时地图是公开网页，非运动世界官方窗口。1.6.1 修复网络线程读取 WebView 设置导致地图失败的问题：主线程保存 User-Agent，网络请求复用该字符串。

## 开发者从源码构建（普通用户跳过）

需要 Python 3、JDK 17、Node.js、Android SDK Platform 34 / Build Tools 34.0.0。设置 `ANDROID_HOME` 与 `JAVA_HOME`，并将 JDK/Node 添加到 PATH。

`native/arm64-v8a/libsportssimulation.so` 是当前 APK 对应的预编译核心，可直接打包。修改 Rust 后先安装 NDK 及 Rust，再执行：

```bash
python scripts/build-android-core.py
python android-virtualrun/build.py --skip-tests
```

省略 `--skip-tests` 会执行已有 Java/JS 检查；1.7.0 发布已运行这些检查，并增加地形曲线和配速参考检查。构建脚本在临时目录输出中间文件，新机器会生成自己的本地测试签名。**不同签名不能覆盖原 APK**；原签名私钥未公开。需要先服务端退出后再卸载旧版，或由维护者使用原签名重签。

输出在 `android-virtualrun/build/`。`ANDROID_BUILD_TOOLS` 可覆盖工具版本，默认 34.0.0；`ANDROID_NDK_HOME` 可指定 NDK。

维护者可通过 `SPORTSROUTE_KEYSTORE` 指定已有私钥，通过 `SPORTSROUTE_KEY_ALIAS`、`SPORTSROUTE_STORE_PASSWORD`、`SPORTSROUTE_KEY_PASSWORD` 环境变量传入签名配置；私钥不进入 APK 或源码包。正式版关闭应用和 WebView 调试。

Windows 用户目录含中文时，部分 aapt2 版本无法读取临时目录，可设置 `SPORTSROUTE_BUILD_ROOT` 为可写的纯英文路径，例如 `C:/Android/build-temp`，再执行构建。
