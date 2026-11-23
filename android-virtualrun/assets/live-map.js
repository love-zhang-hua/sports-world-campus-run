"use strict";
(function(){
 const cfg=window.__routeConfig||{},route=(cfg.points||[]).map(p=>p.slice());let captured=null,overlays=[],initial=false;
 window.__routeMapState={captured:false,ready:false,routeCount:route.length,checkpointCount:(cfg.checkpoints||[]).length,error:null};
 function emit(){window.__routeMapState.routeCount=route.length;RouteCoordinates.update(JSON.stringify(route));}
 function draw(){if(!captured)return;if(overlays.length)captured.remove(overlays);overlays=[];
  if(route.length>1){const path=route.map(p=>p.slice());if(cfg.loop)path.push(path[0]);overlays.push(new AMap.Polyline({path,strokeColor:'#126956',strokeWeight:5,zIndex:2100}));}
  route.forEach((p,i)=>{const n=document.createElement('div');n.textContent=String(i+1);n.style.cssText='background:#126956;color:white;border:2px solid white;border-radius:50%;width:24px;height:24px;text-align:center;line-height:20px;font:14px sans-serif;';overlays.push(new AMap.Marker({position:p,content:n,anchor:'center',zIndex:2200}));});
  (cfg.checkpoints||[]).forEach(p=>{overlays.push(new AMap.CircleMarker({center:p.position,radius:8,fillColor:p.required?'#eb493b':'#ffb647',fillOpacity:1,strokeColor:'#fff',strokeWeight:2,zIndex:2000}));});
  if((cfg.ring||[]).length>2)overlays.push(new AMap.Polyline({path:[...cfg.ring,cfg.ring[0]],strokeColor:'#26975d',strokeWeight:3,zIndex:1900}));
  if(overlays.length)captured.add(overlays);emit();
 }
 function attach(m){if(captured)return;captured=m;window.__routeLiveMap=m;window.__routeMapState.captured=true;
  function complete(){if(initial)return;initial=true;window.__routeMapState.ready=true;draw();if(overlays.length)m.setFitView(overlays,false,[40,40,40,40],18);else m.setZoomAndCenter(17,cfg.center);m.on('click',e=>{if(route.length>=10000)return;route.push([e.lnglat.getLng(),e.lnglat.getLat()]);draw();});RouteCoordinates.ready();}
  m.on('complete',complete);setTimeout(complete,1800);
 }
 try{if(!window.AMap||typeof AMap.Map!=='function')throw Error('Map SDK unavailable');const original=AMap.Map;AMap.Map=new Proxy(original,{construct(target,args,newTarget){const map=Reflect.construct(target,args,newTarget);attach(map);return map;}});
  window.__routeUndo=()=>{route.pop();draw();};
  setTimeout(()=>{if(!captured){window.__routeMapState.error='No live map instance';RouteCoordinates.failed();}},30000);
 }catch(e){window.__routeMapState.error=e.message;RouteCoordinates.failed();}
})();
