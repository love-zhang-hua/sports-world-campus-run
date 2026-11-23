from pathlib import Path
import os,sys,subprocess,shutil,uuid,zipfile,json,hashlib,base64,tempfile,argparse,re
import xml.etree.ElementTree as ET
parser=argparse.ArgumentParser()
parser.add_argument("--skip-tests",action="store_true")
options=parser.parse_args()
root=Path(__file__).resolve().parent
sdk=Path(os.environ.get("ANDROID_HOME",os.environ.get("ANDROID_SDK_ROOT",str(Path.home()/"Android/Sdk"))))
bt=sdk/"build-tools"/os.environ.get("ANDROID_BUILD_TOOLS","34.0.0"); android=sdk/"platforms/android-34/android.jar"
jdk=Path(shutil.which("jar")).parent if shutil.which("jar") else None
if jdk is None:raise SystemExit("JDK with javac, jar and keytool is required")
os.environ["JAVA_HOME"]=str(jdk.parent)
scratch_root=os.environ.get("SPORTSROUTE_BUILD_ROOT")
if scratch_root:Path(scratch_root).mkdir(parents=True,exist_ok=True)
work=Path(tempfile.mkdtemp(prefix="virtualrun-build-",dir=scratch_root))
def tool(folder,name):
 suffix=(".bat" if name in ("d8","apksigner") else ".exe") if os.name=="nt" else ""
 return folder/(name+suffix)
delivery=root/"build";delivery.mkdir(exist_ok=True)
records=[]
if (delivery/"commands.json").exists():shutil.copy2(delivery/"commands.json",delivery/("previous-"+uuid.uuid4().hex[:8]+".json"))
def run(args,cwd=None):
 args=[str(a) for a in args];print("RUN",subprocess.list2cmdline(args),flush=True)
 r=subprocess.run(args,cwd=cwd,capture_output=True)
 stdout_bytes,stderr_bytes=r.stdout,r.stderr
 r.stdout=stdout_bytes.decode("utf8",errors="replace");r.stderr=stderr_bytes.decode("utf8",errors="replace")
 records.append(dict(command=args,cwd=str(cwd or Path.cwd()),stdout=r.stdout,stderr=r.stderr,stdout_base64=base64.b64encode(stdout_bytes).decode(),stderr_base64=base64.b64encode(stderr_bytes).decode(),exit_status=r.returncode))
 (delivery/"commands.json").write_text(json.dumps(records,ensure_ascii=False,indent=2),encoding="utf8")
 print(r.stdout,flush=True);print(r.stderr,file=sys.stderr,flush=True)
 if r.returncode:raise SystemExit(r.returncode)
 return r.stdout
shutil.copytree(root/"src",work/"src");shutil.copytree(root/"res",work/"res");shutil.copytree(root/"assets",work/"assets");shutil.copy2(root/"AndroidManifest.xml",work/"AndroidManifest.xml")
for n in ["classes","dex","tests"]:(work/n).mkdir()
run([tool(jdk,"javac"),"-J-Dfile.encoding=UTF-8","-encoding","UTF-8","-source","8","-target","8","-Xlint:-options","-bootclasspath",str(android)+os.pathsep+str(bt/"core-lambda-stubs.jar"),"-d",work/"classes",*sorted((work/"src").rglob("*.java"))])
if not options.skip_tests:
 run([tool(jdk,"javac"),"-J-Dfile.encoding=UTF-8","-encoding","UTF-8","-d",work/"tests",root/"src/org/sportssimulation/virtualrun/RouteEngine.java",root/"tests/RouteEngineTest.java"])
 run([tool(jdk,"java"),"-cp",work/"tests","org.sportssimulation.virtualrun.RouteEngineTest"])
 run([tool(jdk,"javac"),"-J-Dfile.encoding=UTF-8","-encoding","UTF-8","-d",work/"tests",root/"src/org/sportssimulation/virtualrun/CoordinateTransform.java",root/"tests/CoordinateTransformTest.java"])
 run([tool(jdk,"java"),"-cp",work/"tests","org.sportssimulation.virtualrun.CoordinateTransformTest"])
 run([tool(jdk,"javac"),"-J-Dfile.encoding=UTF-8","-encoding","UTF-8","-d",work/"tests",root/"src/org/sportssimulation/virtualrun/ElevationProfile.java",root/"tests/ElevationProfileTest.java"])
 run([tool(jdk,"java"),"-cp",work/"tests","org.sportssimulation.virtualrun.ElevationProfileTest"])
 run([shutil.which("node") or "node",root/"tests/ui_math_test.cjs"])
 run([shutil.which("node") or "node","--check",root/"assets/account-map.js"])
 run([shutil.which("node") or "node","--check",root/"assets/live-map.js"])
run([tool(jdk,"jar"),"cf",work/"classes.jar","-C",work/"classes","."])
run([tool(bt,"d8"),"--lib",android,"--min-api","26","--output",work/"dex",work/"classes.jar"])
run([tool(bt,"aapt2"),"compile","--dir",work/"res","-o",work/"resources.zip"])
run([tool(bt,"aapt2"),"link","-o",work/"unsigned.apk","-I",android,"--manifest",work/"AndroidManifest.xml","-A",work/"assets",work/"resources.zip"])
with zipfile.ZipFile(work/"unsigned.apk","a",compression=zipfile.ZIP_DEFLATED) as z:
 z.write(work/"dex/classes.dex","classes.dex")
 libs=list((root/"native").glob("*/libsportssimulation.so"))
 if not libs:raise SystemExit("Build original Rust core before packaging")
 for lib in libs:z.write(lib,"lib/"+lib.parent.name+"/libsportssimulation.so")
run([tool(bt,"zipalign"),"-f","4",work/"unsigned.apk",work/"aligned.apk"])
key_override=os.environ.get("SPORTSROUTE_KEYSTORE")
key=Path(key_override) if key_override else root/".signing/local-test.keystore"
if key_override and not key.is_file():raise SystemExit("Specified signing key not found")
key.parent.mkdir(parents=True,exist_ok=True)
alias=os.environ.get("SPORTSROUTE_KEY_ALIAS","androiddebugkey")
os.environ.setdefault("SPORTSROUTE_STORE_PASSWORD","android")
os.environ.setdefault("SPORTSROUTE_KEY_PASSWORD",os.environ["SPORTSROUTE_STORE_PASSWORD"])
if not key.exists():run([tool(jdk,"keytool"),"-genkeypair","-keystore",key,"-storepass:env","SPORTSROUTE_STORE_PASSWORD","-keypass:env","SPORTSROUTE_KEY_PASSWORD","-alias",alias,"-keyalg","RSA","-keysize","2048","-validity","10000","-dname","CN=VirtualRun Local Build,O=Local,C=CN"])
version=ET.parse(root/"AndroidManifest.xml").getroot().get("{http://schemas.android.com/apk/res/android}versionName")
if not version or not re.fullmatch(r"[0-9A-Za-z._-]+",version):raise SystemExit("Invalid manifest versionName")
apk=work/("SportsRoute-"+version+".apk")
run([tool(bt,"apksigner"),"sign","--ks",key,"--ks-key-alias",alias,"--ks-pass","env:SPORTSROUTE_STORE_PASSWORD","--key-pass","env:SPORTSROUTE_KEY_PASSWORD","--out",apk,work/"aligned.apk"])
run([tool(bt,"apksigner"),"verify","--verbose",apk]);run([tool(bt,"zipalign"),"-c","4",apk])
run([tool(bt,"aapt2"),"dump","badging",apk])
shutil.copy2(apk,delivery/apk.name)
print("APK",delivery/apk.name,"SHA256",hashlib.sha256(apk.read_bytes()).hexdigest())
print("BUILD_WORKSPACE",work)
