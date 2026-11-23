package org.sportssimulation.virtualrun;
import android.content.Context;import java.util.concurrent.*;import org.json.*;
public final class CoreBridge {
 private static final ExecutorService worker=Executors.newSingleThreadExecutor();
 private static boolean loaded;private static String loadError="";public static volatile boolean busy;public static volatile String operation="",result="";public static volatile long revision;
 static{try{System.loadLibrary("sportssimulation");loaded=true;}catch(Throwable e){loadError=e.toString();}}
 public static native String call(String path,String op,String input);
 public static synchronized boolean request(Context c,String op,String input){
  if(busy)return false;if(!java.util.Arrays.asList("session","login","logout","preview","submit","mapData","playback").contains(op))return false;
  final String path=c.getApplicationContext().getFilesDir().getAbsolutePath();busy=true;operation=op;result="";
  worker.execute(()->{String answer;try{
   answer=loaded?call(path,"playback".equals(op)?"preview":op,input):new JSONObject().put("ok",false).put("error","原程序核心加载失败："+loadError).toString();
   if("playback".equals(op)){
    JSONObject response=new JSONObject(answer);
    if(response.optBoolean("ok")){
     JSONObject config=new JSONObject(input);config.put("elevation_profile",response.getJSONObject("data").getJSONArray("elevation_profile"));
     if(!c.getSharedPreferences("routes",0).edit().putString("pending",config.toString()).commit())throw new IllegalStateException("路线保存失败");
     c.startForegroundService(new android.content.Intent(c,RunService.class).setAction(RunService.START));
    }
   }
  }catch(Throwable e){try{answer=new JSONObject().put("ok",false).put("error","核心处理失败："+String.valueOf(e.getMessage())).toString();}catch(Exception ignored){answer="{\"ok\":false,\"error\":\"核心处理失败\"}";}}result=answer;revision++;busy=false;});return true;
 }
 public static String state(){try{return new JSONObject().put("busy",busy).put("operation",operation).put("result",result).put("revision",revision).put("loaded",loaded).toString();}catch(Exception e){return "{}";}}
}
