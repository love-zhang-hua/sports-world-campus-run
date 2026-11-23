const fs=require('node:fs'),vm=require('node:vm'),assert=require('node:assert/strict'),path=require('node:path');
const paint=new Proxy({},{get:()=>()=>{}});function el(){return {value:'',checked:false,style:{},textContent:'',disabled:false,hidden:false,open:false,width:600,height:400,clientWidth:320,clientHeight:340,addEventListener:()=>{},setPointerCapture:()=>{},children:[],getContext:()=>paint,appendChild(x){this.children.push(x);}};}
const html=fs.readFileSync(path.join(__dirname,'../assets/index.html'),'utf8'),elements=new Map();for(const match of html.matchAll(/id="([^"]+)"/g)){const x=el();x.value=({speed:'8',laps:'1',sport:'campus'})[match[1]]||'';elements.set(match[1],x);}
const activity=fs.readFileSync(path.join(__dirname,'../src/org/sportssimulation/virtualrun/MainActivity.java'),'utf8');for(const match of html.matchAll(/<script src="([^"]+)"/g))assert.ok(activity.includes('"/'+match[1]+'".equals(path)'), 'WebView must serve '+match[1]);
const calls=[],state={busy:false,revision:0,operation:'session',result:JSON.stringify({ok:true,data:{logged_in:false}})};
const Android={mapLocation:()=>'{}',saved:()=>'',core:(...x)=>calls.push(['core',...x]),coreState:()=>JSON.stringify(state),status:()=>'{"running":false}',save:(...x)=>calls.push(['save',...x]),clearSavedRoute:()=>calls.push(['clear'])};
const box={window:{devicePixelRatio:1,addEventListener:()=>{}},Image:class{},document:{getElementById:id=>elements.get(id)||null,createElement:()=>el()},Android,setInterval:()=>1,console};vm.createContext(box);for(const name of ['account-map.js','app.js'])vm.runInContext(fs.readFileSync(path.join(__dirname,'../assets/'+name),'utf8'),box);
let n=0;function test(code){assert.ok(vm.runInContext(code,box),code);n++;}
test('!document.getElementById("amapKey")&&!document.getElementById("companionOpen")&&!document.getElementById("import")&&!document.getElementById("stadium")');
test('$("refreshMap").disabled');test('(()=>{$("refreshMap").onclick();return $("notice").textContent.includes("登录")})()');assert.equal(calls.filter(c=>c[1]==='mapData').length,0);
test('(()=>{loadPoints([[39.9042,116.4074],[39.9052,116.4074]]);return points.length===2&&routeLength()>110&&cfg().laps===1})()');
test('(()=>{$("loop").checked=true;$("laps").value="2";info();return routeLength()>220&&cfg().laps===2})()');
test('(()=>{$("speed").value="0";try{cfg();return false}catch(e){return true}})()');test('(()=>{$("speed").value="8";$("laps").value="1.5";try{cfg();return false}catch(e){return true}})()');test('(()=>{$("loop").checked=false;return cfg().laps===1})()');test('(()=>{const p=points;active=true;loadPoints([[0,0],[0,1]]);active=false;return p===points})()');
test('(()=>{$("username").value="fixture";$("password").value="fixture-password";$("login").onclick();return $("loginResult").textContent.includes("正在登录")&&$("password").value==="fixture-password"})()');
state.revision++;state.operation='login';state.result=JSON.stringify({ok:false,error:'Fixture error'});test('(()=>{pollCore();return $("loginResult").textContent==="Fixture error"})()');
state.revision++;state.result=JSON.stringify({ok:true,data:{logged_in:true,name:'Fixture'}});test('(()=>{pollCore();return $("loginFields").hidden&&!$("logout").hidden&&!$("submitRoute").disabled})()');assert.equal(calls.filter(c=>c[1]==='mapData').length,1);
state.revision++;state.operation='mapData';state.result=JSON.stringify({ok:true,data:{campuses:[{index:0,name:'Fixture campus',ring:[[39,116],[39.01,116],[39.01,116.01]]}],selected_campus:0,checkpoints:[{name:'Fixture point',lat:39.005,lon:116.005}],warnings:[],source:'TEST FIXTURE'}});test('(()=>{pollCore();return accountMapData.checkpoints.length===1&&mapProjection!==null&&$("mapStatus").textContent.includes("TEST FIXTURE")})()');test('(()=>{const n=points.length;$("checkpointList").children[0].onclick();return points.length===n+1})()');
test('formatResult("preview",{distance_m:1000,duration_s:300}).includes("尚未提交")');state.revision++;state.operation='logout';state.result=JSON.stringify({ok:true,data:{logged_in:false,server_logout:false,message:'NO_LOCAL_SESSION'}});test('(()=>{pollCore();return !accountLogged&&!$("logout").hidden&&$("logoutResult").textContent==="NO_LOCAL_SESSION"&&accountMapData===null})()');
for(const p of [[39.9,116.4],[0,0],[-30,150],[65,-20]])test(`(()=>{const p=${JSON.stringify(p)},q=mapInverse(...mapWorld(p));return Math.abs(q[0]-p[0])<1e-7&&Math.abs(q[1]-p[1])<1e-7})()`);
console.log('PASS '+n+' UI/state assertions; fixture map explicitly synthetic; no account/network/submission calls');

assert.ok(Math.abs(vm.runInContext('referenceGait(10).cadence',box)-166.72)<1e-9);
for(const kmh of [3.6,8,10,12,15,20]){
 const g=vm.runInContext('referenceGait('+kmh+')',box);
 assert.ok(Math.abs(g.cadence/60*g.stride_cm/100-kmh/3.6)<1e-9);
}
assert.ok(vm.runInContext('formatResult("preview",{distance_m:2000,duration_s:720,steps:2220,cadence_spm:185,stride_cm:90,ascent_m:12,terrain_source:"DEM"}).includes("12.0")',box));
assert.ok(vm.runInContext('referenceGait(15).cadence>referenceGait(10).cadence',box));
assert.ok(Math.abs(vm.runInContext('referenceGait(15).cadence',box)-175.07)<1e-9);
assert.equal(vm.runInContext('referenceGait(3.6).supported',box),false);
assert.equal(vm.runInContext('referenceGait(20).supported',box),false);
assert.equal(vm.runInContext('referenceGait(8).supported',box),true);
assert.equal(vm.runInContext('referenceGait(NaN)',box),null);
assert.ok(vm.runInContext('(()=>{$("speed").value="20";info();return $("gaitReference").textContent.includes("超出研究速度范围")})()',box));
console.log('PASS published factor-of-two conversion, monotone speed response, unsupported-range warning, gait identity and DEM summary');
