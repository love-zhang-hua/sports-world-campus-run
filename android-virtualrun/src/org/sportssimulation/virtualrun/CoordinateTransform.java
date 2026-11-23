package org.sportssimulation.virtualrun;
/** GCJ02/WGS84 transform matching Sports-simulation src/track/geom.rs. */
public final class CoordinateTransform {
 private static boolean outside(double a,double o){return o<72.004||o>137.8347||a<.8293||a>55.8271;}
 private static double lat(double x,double y){double r=-100+2*x+3*y+.2*y*y+.1*x*y+.2*Math.sqrt(Math.abs(x));r+=(20*Math.sin(6*x*Math.PI)+20*Math.sin(2*x*Math.PI))*2/3;r+=(20*Math.sin(y*Math.PI)+40*Math.sin(y/3*Math.PI))*2/3;r+=(160*Math.sin(y/12*Math.PI)+320*Math.sin(y*Math.PI/30))*2/3;return r;}
 private static double lon(double x,double y){double r=300+x+2*y+.1*x*x+.1*x*y+.1*Math.sqrt(Math.abs(x));r+=(20*Math.sin(6*x*Math.PI)+20*Math.sin(2*x*Math.PI))*2/3;r+=(20*Math.sin(x*Math.PI)+40*Math.sin(x/3*Math.PI))*2/3;r+=(150*Math.sin(x/12*Math.PI)+300*Math.sin(x/30*Math.PI))*2/3;return r;}
 public static double[] toGcj(double a,double o){if(outside(a,o))return new double[]{a,o};double rad=Math.toRadians(a),magic=1-.006693421622965943*Math.pow(Math.sin(rad),2),root=Math.sqrt(magic);double da=lat(o-105,a-35)*180/((6378245*(1-.006693421622965943))/(magic*root)*Math.PI),doo=lon(o-105,a-35)*180/(6378245/root*Math.cos(rad)*Math.PI);return new double[]{a+da,o+doo};}
 public static double[] toWgs(double a,double o){if(outside(a,o))return new double[]{a,o};double la=a,lo=o;for(int i=0;i<8;i++){double[] g=toGcj(la,lo);la-=g[0]-a;lo-=g[1]-o;}return new double[]{la,lo};}
}
