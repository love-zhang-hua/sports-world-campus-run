package org.sportssimulation.virtualrun;
import android.app.*;import android.content.*;import android.location.*;import android.os.*;import java.util.*;import org.json.*;
public final class RunService extends Service {
 private final Handler handler=new Handler(Looper.getMainLooper());private LocationManager lm;private PowerManager.WakeLock wake;
 private final List<String> providers=new ArrayList<>();private RouteEngine.Playback playback;private ElevationProfile profile;private long last;
 public static volatile String state="空闲",error="";public static volatile double metres,latitude,longitude,altitude,ascent;public static volatile boolean running,paused,hasTerrain;
 public static final String START="start",PAUSE="pause",STOP="stop";
 @Override public IBinder onBind(Intent i){return null;}
 @Override public void onCreate(){super.onCreate();lm=(LocationManager)getSystemService(LOCATION_SERVICE);getSystemService(NotificationManager.class).createNotificationChannel(new NotificationChannel("run","路线回放",NotificationManager.IMPORTANCE_LOW));}
 private Notification notification(){
  PendingIntent open=PendingIntent.getActivity(this,0,new Intent(this,MainActivity.class),PendingIntent.FLAG_UPDATE_CURRENT|PendingIntent.FLAG_IMMUTABLE);
  PendingIntent stop=PendingIntent.getService(this,1,new Intent(this,RunService.class).setAction(STOP),PendingIntent.FLAG_UPDATE_CURRENT|PendingIntent.FLAG_IMMUTABLE);
  return new Notification.Builder(this,"run").setSmallIcon(android.R.drawable.ic_menu_mylocation).setContentTitle("路线跑步模拟器").setContentText(state+" · "+Math.round(metres)+" 米").setContentIntent(open).setOngoing(true).addAction(new Notification.Action.Builder(null,"停止定位",stop).build()).build();
 }
 @Override public int onStartCommand(Intent intent,int flags,int id){
  if(intent==null){stopSelf();return START_NOT_STICKY;}
  String action=intent.getAction();
  if(STOP.equals(action)){state="已停止，已释放模拟定位";stopSelf();return START_NOT_STICKY;}
  if(PAUSE.equals(action)){if(playback!=null){playback.pause(!playback.paused());paused=playback.paused();last=SystemClock.elapsedRealtime();state=paused?"已暂停（保持当前位置）":"正在回放";}return START_NOT_STICKY;}
  if(!START.equals(action)||running)return START_NOT_STICKY;
  try{
   state="正在准备定位";error="";metres=0;altitude=0;ascent=0;hasTerrain=false;startForeground(10,notification());
   String raw=getSharedPreferences("routes",0).getString("pending","");JSONArray points=new JSONObject(raw).getJSONArray("points");JSONObject config=new JSONObject(raw);
   List<RouteEngine.Point> pts=new ArrayList<>();for(int i=0;i<points.length();i++){JSONArray a=points.getJSONArray(i);pts.add(new RouteEngine.Point(a.getDouble(0),a.getDouble(1)));}
   boolean loop=config.optBoolean("loop",false);int laps=loop?config.getInt("laps"):1;
   playback=new RouteEngine.Playback(new RouteEngine(pts,loop),config.getDouble("speed"),laps);
   JSONArray heights=config.getJSONArray("elevation_profile");double[] distances=new double[heights.length()],elevations=new double[heights.length()];for(int i=0;i<heights.length();i++){distances[i]=heights.getJSONArray(i).getDouble(0);elevations[i]=heights.getJSONArray(i).getDouble(1);}profile=new ElevationProfile(distances,elevations);if(Math.abs(profile.length()-playback.target)>1)throw new IllegalArgumentException("地形数据与路线不匹配，请重新获取");hasTerrain=true;
   for(String provider:new String[]{LocationManager.GPS_PROVIDER,LocationManager.NETWORK_PROVIDER}){
    lm.addTestProvider(provider,false,false,false,false,true,true,true,Criteria.POWER_LOW,Criteria.ACCURACY_FINE);
    providers.add(provider);lm.setTestProviderEnabled(provider,true);
   }
   wake=((PowerManager)getSystemService(POWER_SERVICE)).newWakeLock(PowerManager.PARTIAL_WAKE_LOCK,"virtualrun:route");wake.acquire();
   running=true;paused=false;state="正在回放";last=SystemClock.elapsedRealtime();tick.run();
  }catch(Exception e){fail(e);}
  return START_NOT_STICKY;
 }
 private final Runnable tick=new Runnable(){public void run(){try{
  if(playback==null)return;long now=SystemClock.elapsedRealtime();playback.advance((now-last)/1000.0);last=now;
  RouteEngine.Point p=playback.point();latitude=p.lat;longitude=p.lon;metres=playback.travelled();
  double fraction=metres/playback.target;altitude=profile.atFraction(fraction);ascent=profile.ascentAtFraction(fraction);
  for(String name:providers){Location l=new Location(name);l.setLatitude(p.lat);l.setLongitude(p.lon);l.setAccuracy(5);l.setAltitude(altitude);l.setVerticalAccuracyMeters(30);l.setSpeed(playback.paused()||playback.done()?0:(float)playback.speed);l.setSpeedAccuracyMetersPerSecond(.2f);l.setBearing(playback.heading());l.setBearingAccuracyDegrees(5);l.setTime(System.currentTimeMillis());l.setElapsedRealtimeNanos(SystemClock.elapsedRealtimeNanos());lm.setTestProviderLocation(name,l);}
  if(playback.done()){state="回放完成，已释放模拟定位";stopSelf();return;}
  getSystemService(NotificationManager.class).notify(10,notification());handler.postDelayed(this,1000);
 }catch(Exception e){fail(e);}}};
 private void fail(Exception e){error=e.getClass().getSimpleName()+": "+String.valueOf(e.getMessage());state="定位失败：请检查模拟位置应用与定位权限";stopSelf();}
 @Override public void onDestroy(){handler.removeCallbacksAndMessages(null);for(String p:providers)try{lm.removeTestProvider(p);}catch(Exception ignored){}providers.clear();if(wake!=null&&wake.isHeld())wake.release();running=false;paused=false;playback=null;profile=null;stopForeground(STOP_FOREGROUND_REMOVE);super.onDestroy();}
}
