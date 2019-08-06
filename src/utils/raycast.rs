use cgmath::{Point3, Vector3, InnerSpace, Zero};

pub fn modulus(value: f64, modulus: f64) -> f64{
    return (value % modulus + modulus) % modulus;
}

pub fn intbound(mut s: f64, ds: f64) -> f64{
    if ds < 0.0{
        return intbound(-s, -ds);
    }else{
        s = modulus(s, 1.0);
        return (1.0 - s) / ds;
    }
}

pub fn signum(num: f64) -> f64{
    if num > 0.0{
        return 1.0;
    }else if num < 0.0{
        return -1.0;
    }else{
        return 0.0;
    }
}

pub fn bounding(step: f64, min: f64, max: f64, v: f64) -> bool{
    if step > 0.0{
        return v < max;
    }else{
        return v >= min;
    }
}

pub enum RayCastError{
    ZeroDirection,
    NoCollision
}

pub const MAX_SEARCH: Vector3<f64> = Vector3::new(8., 8., 8.);
pub const MIN_SEARCH: Vector3<f64> = Vector3::new(-8., -8., -8.);

pub fn raycast<T>(origin: Point3<f64>, direction: Vector3<f64>, mut radius: f64, callback: T) -> Result<(Point3<f64>, Vector3<f64>), RayCastError>
    where T: Fn(Point3<f64>, Vector3<f64>) -> bool{

    let mut origin = origin.map(|point| point.floor());
    let mut step = Vector3::new(signum(direction.x), signum(direction.y), signum(direction.z));
    let mut max = Vector3::new(intbound(origin.x, direction.x), intbound(origin.y, direction.y), intbound(origin.z, direction.z));
    let mut delta = step.clone();
    delta.x /= direction.x;
    delta.y /= direction.y;
    delta.z /= direction.z;

    let mut face = Vector3::zero();

    if direction == Vector3::zero() {
        return Err(RayCastError::ZeroDirection)
    }

    radius /= (direction.x.powi(2) + direction.y.powi(2) + direction.z.powi(2)).sqrt();

    while ( bounding(step.x, MIN_SEARCH.x, MAX_SEARCH.x, origin.x) && bounding(step.y, MIN_SEARCH.y, MAX_SEARCH.y, origin.y) && bounding(step.z, MIN_SEARCH.z, MAX_SEARCH.z, origin.z)){

        if !(origin.x < MIN_SEARCH.x || origin.y < MIN_SEARCH.y || origin.z < MIN_SEARCH.z
          || origin.x >= MAX_SEARCH.x || origin.y >= MAX_SEARCH.y || origin.z >= MAX_SEARCH.z){
              if callback(origin, face){
                  return Ok((origin, face));
              }
          }

          if max.x < max.y{
              if max.x < max.z{
                  if max.x > radius{ break; }

                  origin.x += step.x;
                  max.x += delta.x;

                  face.x = -step.x;
                  face.y = 0.0;
                  face.z = 0.0;
              } else{
                  if max.z > radius { break; }

                  origin.z += step.z;
                  max.z += delta.z;
                  face.x = 0.0;
                  face.y = 0.0;
                  face.z = -step.z;
              }
          }else {
              if max.y < max.z{
                  if max.y > radius { break; }
                  origin.y += step.y;
                  max.y += delta.y;
                  face.x = 0.0;
                  face.y = -step.y;
                  face.z = 0.0;
              }else{
                  if max.z > radius { break; }
                  origin.z += step.z;
                  face.x = 0.0;
                  face.y = 0.0;
                  face.z = -step.z;
              }
          }

    }

    Err(RayCastError::NoCollision)
    // Ok(())
}
