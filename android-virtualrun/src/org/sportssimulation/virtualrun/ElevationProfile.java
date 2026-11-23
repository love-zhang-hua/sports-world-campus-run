package org.sportssimulation.virtualrun;
/** Metres along the complete run, including repeated laps. No random fallback. */
public final class ElevationProfile {
 private final double[] distance,height,ascent;
 public ElevationProfile(double[] d,double[] h){
  if(d==null||h==null||d.length<2||d.length!=h.length)throw new IllegalArgumentException("请先获取完整路线地形");
  distance=d.clone();height=h.clone();ascent=new double[d.length];
  for(int i=0;i<d.length;i++){
   if(!Double.isFinite(d[i])||!Double.isFinite(h[i])||h[i]<-500||h[i]>9000||d[i]<0||(i>0&&d[i]<=d[i-1]))throw new IllegalArgumentException("路线地形数据无效");
   if(i>0)ascent[i]=ascent[i-1]+Math.max(0,h[i]-h[i-1]);
  }
  if(Math.abs(d[0])>.01||d[d.length-1]<=0)throw new IllegalArgumentException("路线地形距离不完整");
 }
 public double length(){return distance[distance.length-1];}
 public double atFraction(double fraction){return interpolate(fraction,false);}
 public double ascentAtFraction(double fraction){return interpolate(fraction,true);}
 private double interpolate(double fraction,boolean gain){
  if(!Double.isFinite(fraction))throw new IllegalArgumentException("路线进度无效");
  double pos=Math.max(0,Math.min(1,fraction))*length();
  int k=java.util.Arrays.binarySearch(distance,pos);double[] values=gain?ascent:height;
  if(k>=0)return values[k];k=-k-1;double t=(pos-distance[k-1])/(distance[k]-distance[k-1]);
  return values[k-1]+(values[k]-values[k-1])*t;
 }
}
