package org.sportssimulation.virtualrun;
import java.util.*;
public final class RouteEngineTest {
 static int checks=0;static void check(boolean x,String s){checks++;if(!x)throw new AssertionError(s);}
 static void rejects(Runnable r){boolean bad=false;try{r.run();}catch(IllegalArgumentException e){bad=true;}check(bad,"invalid input was accepted");}
 public static void main(String[] args){
  for(double lat:new double[]{0,30,60})for(double lap:new double[]{100,200,300,400,1000})for(double rotation:new double[]{0,45,90}){
   RouteEngine r=new RouteEngine(RouteEngine.stadium(new RouteEngine.Point(lat,116),lap,rotation),true);check(Math.abs(r.length-lap)<.15,"stadium metric length");check(RouteEngine.distance(r.at(0),r.at(r.length))<.01,"closed seam");
  }
  RouteEngine r=new RouteEngine(Arrays.asList(new RouteEngine.Point(0,0),new RouteEngine.Point(0,.001)),false);check(Math.abs(r.length-111.19508)<.01,"geodesic distance");
  check(Math.abs(r.at(r.length/2).lon-.0005)<1e-9,"interpolation");check(r.at(-10).lon==0,"start clamp");check(Math.abs(r.at(1e6).lon-.001)<1e-9,"end clamp");
  RouteEngine.Playback p=new RouteEngine.Playback(r,7.2,1);p.advance(10);check(p.travelled()==20,"speed units");p.pause(true);p.advance(123);check(p.travelled()==20,"pause movement");p.pause(false);p.advance(5);check(p.travelled()==30,"resume movement");p.advance(1e6);check(p.done()&&p.travelled()==r.length,"finish clamp");check(Math.abs(p.point().lon-.001)<1e-9,"finish endpoint");
  RouteEngine ring=new RouteEngine(RouteEngine.stadium(new RouteEngine.Point(39,116),400,0),true);RouteEngine.Playback lp=new RouteEngine.Playback(ring,3.6,2);lp.advance(ring.length);check(!lp.done()&&RouteEngine.distance(lp.point(),ring.at(0))<.001,"lap boundary");lp.advance(ring.length);check(lp.done(),"multi lap completion");
  RouteEngine date=new RouteEngine(Arrays.asList(new RouteEngine.Point(0,179.999),new RouteEngine.Point(0,-179.999)),false);check(Math.abs(Math.abs(date.at(date.length/2).lon)-180)<1e-9,"dateline");
  rejects(()->new RouteEngine.Point(Double.NaN,0));rejects(()->new RouteEngine.Point(91,0));rejects(()->new RouteEngine(Arrays.asList(new RouteEngine.Point(0,0),new RouteEngine.Point(0,0)),false));rejects(()->new RouteEngine.Playback(r,0,1));rejects(()->new RouteEngine.Playback(r,8,0));rejects(()->RouteEngine.stadium(new RouteEngine.Point(0,0),99,0));rejects(()->p.advance(-1));rejects(()->p.advance(Double.NaN));
  System.out.println("PASS "+checks+" assertions: metric tracks, interpolation, pause/resume, completion, loop seam, dateline, invalid inputs");
 }
}