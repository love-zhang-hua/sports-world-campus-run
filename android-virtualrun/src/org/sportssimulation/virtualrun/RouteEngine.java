package org.sportssimulation.virtualrun;
import java.util.*;
/** Sports-simulation geometry port; see THIRD_PARTY_NOTICES.md. WGS84 throughout. */
public final class RouteEngine {
 public static final double R=6371008.8;
 public static final class Point {
  public final double lat,lon;
  public Point(double a,double o) {
   if(!Double.isFinite(a)||!Double.isFinite(o)||Math.abs(a)>85||Math.abs(o)>180) throw new IllegalArgumentException("坐标须为 WGS84，纬度 -85～85，经度 -180～180"); lat=a;lon=o;
  }
 }
 public final List<Point> points; public final double[] arcs; public final double length;
 public RouteEngine(List<Point> input,boolean closed) {
  if(input==null||input.size()<2||input.size()>10000) throw new IllegalArgumentException("路线需要 2～10000 个点");
  ArrayList<Point> clean=new ArrayList<>();
  for(Point q:input) if(clean.isEmpty()||distance(clean.get(clean.size()-1),q)>0.01) clean.add(q);
  if(clean.size()<2) throw new IllegalArgumentException("路线不能全部是同一个点");
  if(closed&&distance(clean.get(0),clean.get(clean.size()-1))>0.01)clean.add(clean.get(0));
  points=Collections.unmodifiableList(clean); arcs=new double[points.size()];
  for(int i=1;i<arcs.length;i++) {double d=distance(points.get(i-1),points.get(i));if(d>100000)throw new IllegalArgumentException("相邻点距离不能超过 100 公里");arcs[i]=arcs[i-1]+d;}
  length=arcs[arcs.length-1]; if(length<1||length>1000000)throw new IllegalArgumentException("路线总长须为 1 米～1000 公里");
 }
 public static double distance(Point a,Point b){double x=Math.toRadians(b.lat-a.lat),y=Math.toRadians(b.lon-a.lon);double h=Math.pow(Math.sin(x/2),2)+Math.cos(Math.toRadians(a.lat))*Math.cos(Math.toRadians(b.lat))*Math.pow(Math.sin(y/2),2);return 2*R*Math.asin(Math.sqrt(Math.min(1,h)));}
 public Point at(double metres){
  if(!Double.isFinite(metres))throw new IllegalArgumentException("距离无效");
  double s=Math.max(0,Math.min(length,metres));int k=Arrays.binarySearch(arcs,s);if(k>=0)return points.get(k);k=-k-1;
  Point a=points.get(k-1),b=points.get(k);double t=(s-arcs[k-1])/(arcs[k]-arcs[k-1]);
  double d=(b.lon-a.lon+540)%360-180;double lon=(a.lon+d*t+540)%360-180;
  return new Point(a.lat+(b.lat-a.lat)*t,lon);
 }
 public static float bearing(Point a,Point b){double la=Math.toRadians(a.lat),lb=Math.toRadians(b.lat),d=Math.toRadians(b.lon-a.lon);return (float)((Math.toDegrees(Math.atan2(Math.sin(d)*Math.cos(lb),Math.cos(la)*Math.sin(lb)-Math.sin(la)*Math.cos(lb)*Math.cos(d)))+360)%360);}
 /** Direct port of TrackSpec::dimensions and rounded_rectangle; spherical WGS84 projection replaces fixed BD09 scale. */
 public static List<Point> stadium(Point center,double lap,double degrees){
  if(!Double.isFinite(lap)||lap<100||lap>1000||!Double.isFinite(degrees))throw new IllegalArgumentException("跑道圈长须为 100～1000 米");
  double scale=lap/(2*84.39+2*Math.PI*36.80),s=84.39*scale,r=36.80*scale,angle=Math.toRadians(degrees);
  int ns=Math.max(8,(int)Math.ceil(s/.75)),na=Math.max(8,(int)Math.ceil(Math.PI*r/.75));List<Point> result=new ArrayList<>();
  for(int part=0;part<4;part++)for(int i=0;i<(part%2==0?ns:na);i++){
   double x,y;if(part==0){x=-s/2+s*i/ns;y=-r;}else if(part==2){x=s/2-s*i/ns;y=r;}else{double a=(part==1?-Math.PI/2:Math.PI/2)+Math.PI*i/na;x=(part==1?s/2:-s/2)+r*Math.cos(a);y=r*Math.sin(a);}
   double east=x*Math.cos(angle)-y*Math.sin(angle),north=x*Math.sin(angle)+y*Math.cos(angle);
   result.add(new Point(center.lat+Math.toDegrees(north/R),center.lon+Math.toDegrees(east/(R*Math.cos(Math.toRadians(center.lat))))));
  }
  result.add(result.get(0));return result;
 }
 public static final class Playback {
  public final RouteEngine route;public final double speed,target;private double travelled;private boolean paused;
  public Playback(RouteEngine r,double kmh,int laps){if(!Double.isFinite(kmh)||kmh<.5||kmh>30||laps<1||laps>100)throw new IllegalArgumentException("速度 0.5～30 km/h，圈数 1～100");route=r;speed=kmh/3.6;target=r.length*laps;}
  public void advance(double seconds){if(!Double.isFinite(seconds)||seconds<0)throw new IllegalArgumentException("时间无效");if(!paused)travelled=Math.min(target,travelled+seconds*speed);}
  public void pause(boolean p){paused=p;}public boolean paused(){return paused;}public boolean done(){return travelled>=target;}public double travelled(){return travelled;}
  public Point point(){return route.at(done()?route.length:travelled%route.length);}
  public float heading(){double s=done()?route.length:travelled%route.length;return bearing(route.at(Math.max(0,s-.5)),route.at(Math.min(route.length,s+.5)));}
 }
}
