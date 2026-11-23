import Foundation

enum Coordinates {
    static func outside(_ p: GeoPoint) -> Bool { p.lon < 72.004 || p.lon > 137.8347 || p.lat < 0.8293 || p.lat > 55.8271 }
    static func latitude(_ x: Double, _ y: Double) -> Double {
        var r = -100 + 2*x + 3*y + 0.2*y*y + 0.1*x*y + 0.2*sqrt(abs(x))
        r += (20*sin(6*x*Double.pi) + 20*sin(2*x*Double.pi))*2/3
        r += (20*sin(y*Double.pi) + 40*sin(y/3*Double.pi))*2/3
        r += (160*sin(y/12*Double.pi) + 320*sin(y*Double.pi/30))*2/3
        return r
    }
    static func longitude(_ x: Double, _ y: Double) -> Double {
        var r = 300 + x + 2*y + 0.1*x*x + 0.1*x*y + 0.1*sqrt(abs(x))
        r += (20*sin(6*x*Double.pi) + 20*sin(2*x*Double.pi))*2/3
        r += (20*sin(x*Double.pi) + 40*sin(x/3*Double.pi))*2/3
        r += (150*sin(x/12*Double.pi) + 300*sin(x/30*Double.pi))*2/3
        return r
    }
    static func gcj(_ p: GeoPoint) -> GeoPoint {
        if outside(p) { return p }
        let rad = p.lat*Double.pi/180
        let magic = 1 - 0.006693421622965943*pow(sin(rad), 2), root = sqrt(magic)
        let a = latitude(p.lon-105, p.lat-35)*180/((6378245*(1-0.006693421622965943))/(magic*root)*Double.pi)
        let o = longitude(p.lon-105, p.lat-35)*180/(6378245/root*cos(rad)*Double.pi)
        return GeoPoint(lat: p.lat+a, lon: p.lon+o)
    }
    static func wgs(_ p: GeoPoint) -> GeoPoint {
        if outside(p) { return p }; var result = p
        for _ in 0..<8 { let g = gcj(result); result.lat -= g.lat-p.lat; result.lon -= g.lon-p.lon }
        return result
    }
    static func mapPair(_ p: GeoPoint) -> [Double] { let g = gcj(p); return [g.lon, g.lat] }
}
