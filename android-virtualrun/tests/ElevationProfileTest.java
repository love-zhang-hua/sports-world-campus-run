package org.sportssimulation.virtualrun;
public final class ElevationProfileTest {
 private static void eq(double a,double b){if(Math.abs(a-b)>1e-8)throw new AssertionError(a+" != "+b);}
 public static void main(String[] args){
  ElevationProfile p=new ElevationProfile(new double[]{0,100,200,300,400},new double[]{20,30,20,30,20});
  eq(p.atFraction(.125),25);eq(p.ascentAtFraction(.125),5);eq(p.ascentAtFraction(.5),10);eq(p.ascentAtFraction(1),20);eq(p.atFraction(-1),20);eq(p.atFraction(2),20);
  ElevationProfile flat=new ElevationProfile(new double[]{0,400},new double[]{0,0});eq(flat.ascentAtFraction(1),0);
  int rejected=0;try{new ElevationProfile(new double[]{0,0},new double[]{20,30});}catch(IllegalArgumentException e){rejected++;}
  try{new ElevationProfile(new double[]{0,1},new double[]{Double.NaN,30});}catch(IllegalArgumentException e){rejected++;}
  try{new ElevationProfile(new double[]{1,2},new double[]{20,30});}catch(IllegalArgumentException e){rejected++;}
  if(rejected!=3)throw new AssertionError("Invalid DEM accepted");
  System.out.println("PASS elevation interpolation, repeated-lap ascent, flat terrain, bounds and missing/invalid DEM rejection");
 }
}
