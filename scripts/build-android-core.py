"""Build ARM64 Android shared core from source. Does not run tests or access accounts."""
from pathlib import Path
import os, platform, shutil, subprocess
root=Path(__file__).resolve().parents[1]
sdk=Path(os.environ.get('ANDROID_HOME', os.environ.get('ANDROID_SDK_ROOT', str(Path.home()/'Android/Sdk'))))
ndk_override=os.environ.get('ANDROID_NDK_HOME')
ndks=sorted((sdk/'ndk').glob('*'), key=lambda p:tuple(int(s) for s in p.name.split('.') if s.isdigit()))
ndk=Path(ndk_override) if ndk_override else (ndks[-1] if ndks else None)
if ndk is None: raise SystemExit('Install Android NDK or set ANDROID_NDK_HOME')
host={'Windows':'windows-x86_64','Darwin':'darwin-x86_64','Linux':'linux-x86_64'}[platform.system()]
bin_dir=ndk/'toolchains/llvm/prebuilt'/host/'bin'
cc=bin_dir/('aarch64-linux-android26-clang.cmd' if os.name=='nt' else 'aarch64-linux-android26-clang')
ar=bin_dir/('llvm-ar.exe' if os.name=='nt' else 'llvm-ar')
env=dict(os.environ,CC_aarch64_linux_android=str(cc),AR_aarch64_linux_android=str(ar),CARGO_TARGET_AARCH64_LINUX_ANDROID_LINKER=str(cc))
subprocess.run(['rustup','target','add','aarch64-linux-android'],check=True)
subprocess.run(['cargo','build','--locked','--lib','--release','--no-default-features','--features','route-app','--target','aarch64-linux-android'],cwd=root,env=env,check=True)
out=root/'android-virtualrun/native/arm64-v8a';out.mkdir(parents=True,exist_ok=True)
shutil.copy2(root/'target/aarch64-linux-android/release/libsportssimulation.so',out/'libsportssimulation.so')
print(out/'libsportssimulation.so')
