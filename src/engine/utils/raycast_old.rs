use cgmath::{Point3, Vector3};

fn intbound(s: f64, ds: f64) -> f64{
    // if ds < 0.{
    //     intbound(-s, -ds)
    // }else{
    //     let s = modulus(s, 1.);
    //     (1. - s) / ds
    // }

    if ds < 0. && s.round() == s{
        return 0.;
    }

    (if ds > 0. { (if s == 0. { 1.} else { s.ceil() }) - s } else { s - s.floor() }) / ds.abs()
    // s.fract() / ds.abs()
}

pub fn raycast<T>(origin: Point3<f64>, direction: Point3<f64>, limit: f64, callback: T) -> Option<(Point3<f64>, Vector3<f64>)>
    where T: Fn(Point3<f64>, Vector3<f64>) -> bool{
    let mut pos = origin.map(|p| p.floor());
    let step = direction.map(|p| if p > 0. { 1. } else if p < 0. { -1. } else { 0. });
    let mut max = Point3::new(intbound(origin.x, direction.x), intbound(origin.y, direction.y), intbound(origin.z, direction.z));
    let delta = Vector3::new(step.x / direction.x, step.y / direction.y, step.z / direction.z);
    let mut face = Vector3::new(0., 0., 0.);

    let radius = limit / (direction.x.powf(2.) + direction.y.powf(2.) + direction.z.powf(2.)).sqrt();

    loop{
        if callback(pos, face){
            return Some((pos, face));
        }

        if max.x < max.y{
            if max.x < max.z{
                pos.x += step.x;
                if max.x > radius { break }
                max.x += delta.x;
                face = Vector3::new(-step.x, 0., 0.);
            }else{
                pos.z += step.z;
                if max.z > radius { break }
                max.z += delta.z;
                face = Vector3::new(0., 0., -step.z);
            }
        }else{
            if max.y < max.z {
                pos.y += step.y;
                if max.y > radius { break }
                max.y += delta.y;
                face = Vector3::new(0., -step.y, 0.);
            }else{
                pos.z += step.z;
                if max.z > radius { break }
                max.z += delta.z;
                face = Vector3::new(0., 0., -step.z);
            }
        }
    }

    None
}
