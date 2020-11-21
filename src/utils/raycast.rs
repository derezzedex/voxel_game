use ultraviolet::Vec3;

#[allow(dead_code)]
pub fn raycast<T>(mut origin: Vec3, mut end: Vec3, radius: f32, callback: T) -> Option<(Vec3, Vec3)>
where
    T: Fn(Vec3) -> bool,
{
    origin.x += 0.5;
    origin.y += 0.5;
    origin.z += 0.5;

    end *= radius;
    let end = origin + end;
    // end.x += 0.5;
    // end.y += 0.5;
    // end.z += 0.5;

    let floor_origin = origin.map(|point| point.floor());
    let (mut i, mut j, mut k) = (
        floor_origin.x as isize,
        floor_origin.y as isize,
        floor_origin.z as isize,
    );

    let floor_end = end.map(|point| point.floor());
    let (iend, jend, kend) = (
        floor_end.x as isize,
        floor_end.y as isize,
        floor_end.z as isize,
    );

    let di = if origin.x < end.x {
        1
    } else if origin.x > end.x {
        -1
    } else {
        0
    };

    let dj = if origin.y < end.y {
        1
    } else if origin.y > end.y {
        -1
    } else {
        0
    };

    let dk = if origin.z < end.z {
        1
    } else if origin.z > end.z {
        -1
    } else {
        0
    };

    let deltatx = 1. / (end.x - origin.x).abs();
    let deltaty = 1. / (end.y - origin.y).abs();
    let deltatz = 1. / (end.z - origin.z).abs();

    let minx = floor_origin.x;
    let maxx = minx + 1.;
    let mut tx = if origin.x > end.x {
        (origin.x - minx) * deltatx
    } else {
        (maxx - origin.x) * deltatx
    };

    let miny = floor_origin.y;
    let maxy = miny + 1.;
    let mut ty = if origin.y > end.y {
        (origin.y - miny) * deltaty
    } else {
        (maxy - origin.y) * deltaty
    };

    let minz = floor_origin.z;
    let maxz = minz + 1.;
    let mut tz = if origin.z > end.z {
        (origin.z - minz) * deltatz
    } else {
        (maxz - origin.z) * deltatz
    };
    let mut face = Vec3::zero();

    loop {
        let pos = Vec3::new(i as f32, j as f32, k as f32);

        if callback(pos) {
            return Some((pos, face));
        }

        if tx <= ty && tx <= tz {
            if i == iend {
                break;
            }
            tx += deltatx;
            i += di;

            // if di == 1 move pos x
            if di == 1 {
                face = Vec3::zero();
                face.x = 1.;
            } else if di == -1 {
                face = Vec3::zero();
                face.x = -1.;
            }
        // if di == -1 move neg x
        } else if ty <= tz {
            if j == jend {
                break;
            }
            ty += deltaty;
            j += dj;

            // if dj == 1 move pos y
            if dj == 1 {
                face = Vec3::zero();
                face.y = 1.;
            } else if dj == -1 {
                face = Vec3::zero();
                face.y = -1.;
            }
        // if dj == -1 move neg y
        } else {
            if k == kend {
                break;
            }
            tz += deltatz;
            k += dk;

            // if dj == 1 move pos y
            if dk == 1 {
                face = Vec3::zero();
                face.z = 1.;
            } else if dk == -1 {
                face = Vec3::zero();
                face.z = -1.;
            }
            // if dj == -1 move neg y
        }
    }

    None
}
