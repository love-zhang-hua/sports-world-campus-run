"use strict";
let accountMapData=null,mapProjection=null,mapCenter=[39.9042,116.4074],mapZoom=16,mapDrag=null;
let vectorMap=null,mapReady=false;let mapTileStats={loaded:0,failed:0,visible:0};
try{const v=JSON.parse(localStorage.getItem("mapViewport"));if(v&&Array.isArray(v.center)&&v.center.length===2&&v.center.every(Number.isFinite)&&Math.abs(v.center[0])<=85&&Math.abs(v.center[1])<=180&&v.zoom>=3&&v.zoom<=20){mapCenter=v.center;mapZoom=v.zoom;}}catch(e){}
function mapWorld(p,z=mapZoom){const n=256*2**z,a=Math.max(-85,Math.min(85,p[0]))*Math.PI/180;return[(p[1]+180)/360*n,(1-Math.asinh(Math.tan(a))/Math.PI)/2*n];}
function mapInverse(x,y,z=mapZoom){const n=256*2**z;return[Math.max(-85,Math.min(85,Math.atan(Math.sinh(Math.PI*(1-2*y/n)))*180/Math.PI)),((x/n*360)%360+360)%360-180];}
function campusRing(){return accountMapData?.campuses?.[accountMapData.selected_campus]?.ring||[];}
function fitAccountMap(){const dots=accountMapData?.checkpoints?.map(p=>[p.lat,p.lon])||[],all=[...campusRing(),...dots,...(typeof points==='undefined'?[]:points)];if(!all.length){drawAccountMap();return;}const lat=all.map(p=>p[0]),lon=all.map(p=>p[1]);mapCenter=[(Math.min(...lat)+Math.max(...lat))/2,(Math.min(...lon)+Math.max(...lon))/2];const canvas=document.getElementById('accountMap'),w=canvas.clientWidth||320,h=canvas.clientHeight||340;for(mapZoom=20;mapZoom>3;mapZoom--){const xs=all.map(p=>mapWorld(p)[0]),ys=all.map(p=>mapWorld(p)[1]);if(Math.max(...xs)-Math.min(...xs)<w-55&&Math.max(...ys)-Math.min(...ys)<h-55)break;}if(vectorMap)vectorMap.jumpTo({center:[mapCenter[1],mapCenter[0]],zoom:mapZoom});drawAccountMap();}
function renderAccountMap(data){accountMapData=data;const select=document.getElementById('campus'),list=document.getElementById('checkpointList');select.textContent='';list.textContent='';select.hidden=!data||!data.campuses.length;if(!data){document.getElementById('mapStatus').textContent='登录后显示你的校区与必经点。';drawAccountMap();return;}for(const c of data.campuses){const option=document.createElement('option');option.value=c.index;option.textContent=c.name;select.appendChild(option);}select.value=data.selected_campus;for(const p of data.checkpoints){const button=document.createElement('button');button.textContent=(p.required?'必经 · ':'普通 · ')+p.name;button.onclick=()=>addAccountPoint([p.lat,p.lon]);list.appendChild(button);}const must=data.checkpoints.filter(p=>p.required).length;document.getElementById('mapStatus').textContent=data.source+' · '+data.checkpoints.length+' 个点位 / '+must+' 个必经点'+(data.warnings.length?'\n'+data.warnings.join('\n'):'');fitAccountMap();}
function addAccountPoint(p){if(active){notice('请先停止定位回放');return;}if(points.length>=10000){notice('最多 10000 个路线点');return;}points.push(p);info();saveRoute();}

function mapFeatures(){
 const features=[],ring=campusRing(),route=typeof points==='undefined'?[]:points;
 const line=(ps,kind,close)=>{if(ps.length<2)return;const c=ps.map(p=>[p[1],p[0]]);if(close)c.push(c[0]);features.push({type:'Feature',properties:{kind},geometry:{type:'LineString',coordinates:c}});};
 line(ring,'campus',true);line(route,'route',document.getElementById('loop').checked);
 (accountMapData?.checkpoints||[]).forEach((p,i)=>features.push({type:'Feature',properties:{kind:p.required?'required':'checkpoint',label:(p.required?'?? ':'')+(i+1)},geometry:{type:'Point',coordinates:[p.lon,p.lat]}}));
 route.forEach(p=>features.push({type:'Feature',properties:{kind:'selected',label:''},geometry:{type:'Point',coordinates:[p[1],p[0]]}}));
 return {type:'FeatureCollection',features};
}
function drawAccountMap(){
 const container=document.getElementById('accountMap');if(!container)return;
 mapProjection={zoom:mapZoom};
 if(!vectorMap){
  if(typeof maplibregl==='undefined'){document.getElementById('tileStatus').textContent='????????????????';return;}
  vectorMap=new maplibregl.Map({container:'accountMap',style:'https://app.local/map-style.json',center:[mapCenter[1],mapCenter[0]],zoom:mapZoom,minZoom:3,maxZoom:20,attributionControl:false,dragRotate:false,pitchWithRotate:false,renderWorldCopies:false});
  vectorMap.touchZoomRotate.disableRotation();
  vectorMap.on('load',()=>{
   mapReady=true;
   vectorMap.addSource('account-route',{type:'geojson',data:mapFeatures()});
   vectorMap.addLayer({id:'route-lines',type:'line',source:'account-route',filter:['==',['geometry-type'],'LineString'],paint:{'line-color':['match',['get','kind'],'campus','#26975d','#126956'],'line-width':4}});
   vectorMap.addLayer({id:'route-points',type:'circle',source:'account-route',filter:['==',['geometry-type'],'Point'],paint:{'circle-color':['match',['get','kind'],'required','#eb493b','checkpoint','#ffb647','#126956'],'circle-radius':['match',['get','kind'],'selected',4,7],'circle-stroke-color':'#ffffff','circle-stroke-width':2}});
   vectorMap.addLayer({id:'checkpoint-labels',type:'symbol',source:'account-route',filter:['!=',['get','label'],''],layout:{'text-field':['get','label'],'text-font':['Noto Sans Regular'],'text-size':13,'text-offset':[0,-1.5],'text-allow-overlap':true},paint:{'text-color':'#203e34','text-halo-color':'#fff','text-halo-width':2}});
   drawAccountMap();
  });
  vectorMap.on('click',e=>addAccountPoint([e.lngLat.lat,e.lngLat.lng]));
  vectorMap.on('moveend',()=>{const c=vectorMap.getCenter();mapCenter=[c.lat,c.lng];mapZoom=vectorMap.getZoom();try{localStorage.setItem('mapViewport',JSON.stringify({center:mapCenter,zoom:mapZoom}));}catch(e){}});
  vectorMap.on('sourcedata',e=>{if(e.sourceId!=='account-route'&&e.isSourceLoaded)mapTileStats.loaded++;});
  vectorMap.on('idle',()=>{mapTileStats.visible=vectorMap.queryRenderedFeatures().length;document.getElementById('tileStatus').textContent='???? ? ?????'+(mapTileStats.failed?' ? ??????????':'');});
  vectorMap.on('error',()=>{mapTileStats.failed++;document.getElementById('tileStatus').textContent='??????????????????????';});
 }
 if(mapReady)vectorMap.getSource('account-route').setData(mapFeatures());
}
document.getElementById('mapZoomIn').onclick=()=>{if(vectorMap)vectorMap.zoomIn();};
document.getElementById('mapZoomOut').onclick=()=>{if(vectorMap)vectorMap.zoomOut();};
document.getElementById('mapFit').onclick=fitAccountMap;
window.addEventListener('resize',()=>{if(vectorMap)vectorMap.resize();});
