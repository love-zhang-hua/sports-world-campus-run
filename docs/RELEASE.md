# 运动世界校园跑 1.7.0

- [Android 单APK下载](https://github.com/love-zhang-hua/sports-world-campus-run/releases/latest/download/SportsRoute-1.7.0.apk)：Android 8.0+ / ARM64，版本号1.7.0、versionCode 18，沿用1.6.2签名，可覆盖安装。
- [Windows 安装包下载](https://github.com/love-zhang-hua/sports-world-campus-run/releases/latest/download/SportsRoute-1.7.0-Setup.exe)：Windows 10/11 64位，本机安装向导；Windows发行物未做商业代码签名。
- [Android安装使用](ANDROID_GUIDE.md) · [Windows使用与模型说明](WINDOWS.md)。无需另装LocationSpoofer或测试包。

## 本版变化

合并两组、13条匿名运动摘要，按配速适配参考步频，并根据速度推导步幅，保持速度、步幅和步频一致。Android预览显示轨迹平均步频、步幅、步数与累计爬升。

预览、回放、提交流程查询实际路线的SRTM30m地形。Android定位回放不再把海拔固定为0；使用同一份高程曲线插值，暂停不增加爬升，绕圈按行程累计。采样坐标不含账号信息；不需要地图Key；请求失败明确报错，不随机补高度。修复短路线在终点取整时重复采样导致地形失败的问题。

## 使用与限制

先停止旧版回放并退出账号，再下载新版APK覆盖安装。打开后选路线和速度，检查轨迹；提交仍需单独确认。回放需定位权限和模拟位置设置。使用结束停止回放并退出账号。

样本是描述性参考，不能预测个人生理表现或恢复逐秒传感器数据。DEM约30米分辨率不能可靠表示台阶、桥面或楼层。步频参数不会改写手机硬件计步器。Windows不能替代Android系统模拟定位。

Android地图仍为高德公开网页与账号点位；不是运动世界官方地图窗口。网络、网页变化和服务端要求可能影响使用，不保证提交结果。本次不提供iOS安装包，也未做真实账号登录或提交测试；具体本地验证见随附说明。

基于 [liing6666/Sports-simulation](https://github.com/liing6666/Sports-simulation) 0.5.1，原作者YanamiNeko，保留CC BY-NC 4.0署名及非商业许可。此许可不是OSI认可的开源许可证。第三方来源见THIRD_PARTY_NOTICES.md。
