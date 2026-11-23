# 运动世界校园跑 1.7.1

- [Android 单APK下载](https://github.com/love-zhang-hua/sports-world-campus-run/releases/latest/download/SportsRoute-1.7.1.apk)：Android 8.0+ / ARM64，版本号1.7.1、versionCode 19，沿用1.6.2签名，可覆盖安装。
- [Windows 安装包下载](https://github.com/love-zhang-hua/sports-world-campus-run/releases/latest/download/SportsRoute-1.7.1-Setup.exe)：Windows 10/11 64位，本机安装向导；Windows发行物未做商业代码签名。
- [Android安装使用](ANDROID_GUIDE.md) · [Windows使用与模型说明](WINDOWS.md)。无需另装LocationSpoofer或测试包。

## 本版变化

修正旧版把两位跑者混合拟合后出现的反向速度—步频趋势。默认使用 Van Oeveren 等（2019）的群体参考关系，正确把每分钟完整步态周期转换成左右脚步数。6:00/km参考166.7步/分，4:00/km参考175.1步/分；步幅由实际速度推导。两组13条匿名摘要保留供核对，不据此声称个人生理预测。

Android与Windows同步更新，超出论文速度范围时显示边界估计提示。新增[论文与原始摘要审查](MODEL_EVIDENCE.md)，说明个体差异、单位换算、爬升和功率/热量的限制。路线高程继续采用SRTM30m，无随机补高、无强行匹配截图爬升。坡度研究不足以推导本人的通用步频修正系数。

## 使用与限制

先停止旧版回放并退出账号，再下载新版APK覆盖安装。打开后选路线和速度，检查轨迹；提交仍需单独确认。回放需定位权限和模拟位置设置。使用结束停止回放并退出账号。

论文模型是群体参考，不能预测个人生理表现或恢复逐秒传感器数据。DEM约30米分辨率不能可靠表示台阶、桥面或楼层。步频参数不会改写手机硬件计步器。Windows不能替代Android系统模拟定位。

Android地图仍为高德公开网页与账号点位；不是运动世界官方地图窗口。网络、网页变化和服务端要求可能影响使用，不保证提交结果。本次不提供iOS安装包，也未做真实账号登录或提交测试；具体本地验证见随附说明。

基于 [liing6666/Sports-simulation](https://github.com/liing6666/Sports-simulation) 0.5.1，原作者YanamiNeko，保留CC BY-NC 4.0署名及非商业许可。此许可不是OSI认可的开源许可证。第三方来源见THIRD_PARTY_NOTICES.md。
