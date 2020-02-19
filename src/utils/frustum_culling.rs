use std::mem;
use cgmath::{BaseFloat, Matrix4, Ortho, Perspective, PerspectiveFov, Vector3, Point3, prelude::*};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct FrustumCuller<S> {
    nx_x: S,
    nx_y: S,
    nx_z: S,
    nx_w: S,
    px_x: S,
    px_y: S,
    px_z: S,
    px_w: S,
    ny_x: S,
    ny_y: S,
    ny_z: S,
    ny_w: S,
    py_x: S,
    py_y: S,
    py_z: S,
    py_w: S,
    nz_x: S,
    nz_y: S,
    nz_z: S,
    nz_w: S,
    pz_x: S,
    pz_y: S,
    pz_z: S,
    pz_w: S,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct BoundingBox<S> {
    /// min point
    pub min: Point3<S>,
    /// max point
    pub max: Point3<S>,
}

#[repr(C)]
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct Sphere<S> {
    /// min point
    pub center: Vector3<S>,
    /// max point
    pub radius: S,
}

impl<S: BaseFloat> Sphere<S> {
    #[inline]
    pub fn from_params(center: Vector3<S>, radius: S) -> Self {
        Self { center, radius }
    }

    #[inline]
    pub fn new() -> Self {
        Self {
            center: Vector3::zero(),
            radius: S::zero(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Copy, Clone)]
pub enum Intersection {
    /// fully inside the frustum
    Inside,
    /// Partially inside the frustum
    Partial,
    /// Fully outside the frustum
    Outside,
}

impl<S: BaseFloat> BoundingBox<S> {
    #[inline]
    pub fn from_params(min: Point3<S>, max: Point3<S>) -> Self {
        Self { min, max }
    }

    #[inline]
    pub fn new() -> Self {
        Self::from_params(Point3::new(S::zero(), S::zero(), S::zero()), Point3::new(S::zero(), S::zero(), S::zero()))
    }
}

impl<S: BaseFloat> FrustumCuller<S> {
    /// Creates an identity frustum culler. This is equivalent to calling the `from_matrix` method
    /// passing an identity matrix.
    pub fn new() -> Self {
        Self::from_matrix(Matrix4::identity())
    }

    /// Creates a frustum culler from a given perspective frustum configuration.
    #[inline]
    pub fn from_perspective(perspective: Perspective<S>) -> Self {
        Self::from_matrix(perspective.into())
    }

    /// Creates a frustum culler from a given `PerspectiveFov` configuration.
    #[inline]
    pub fn from_perspective_fov(perspective: PerspectiveFov<S>) -> Self {
        Self::from_matrix(perspective.into())
    }

    #[inline]
    pub fn from_ortho(ortho: Ortho<S>) -> Self {
        Self::from_matrix(ortho.into())
    }

    /// Creates a `FrustumCuller` from an arbitrary matrix, from which the frustum planes are
    /// computed.
    pub fn from_matrix(m: Matrix4<S>) -> Self {
        let mut culler: Self = unsafe { mem::zeroed() };

        culler.nx_x = m.x.w + m.x.x;
        culler.nx_y = m.y.w + m.y.x;
        culler.nx_z = m.z.w + m.z.x;
        culler.nx_w = m.w.w + m.w.x;
        //if (allow_test_spheres) {
        let invl = (culler.nx_x * culler.nx_x + culler.nx_y * culler.nx_y
            + culler.nx_z * culler.nx_z)
            .sqrt()
            .recip();
        culler.nx_x *= invl;
        culler.nx_y *= invl;
        culler.nx_z *= invl;
        culler.nx_w *= invl;
        //}
        culler.px_x = m.x.w - m.x.x;
        culler.px_y = m.y.w - m.y.x;
        culler.px_z = m.z.w - m.z.x;
        culler.px_w = m.w.w - m.w.x;
        //if (allow_test_spheres) {
        let invl = (culler.px_x * culler.px_x + culler.px_y * culler.px_y
            + culler.px_z * culler.px_z)
            .sqrt()
            .recip();
        culler.px_x *= invl;
        culler.px_y *= invl;
        culler.px_z *= invl;
        culler.px_w *= invl;
        //}
        culler.ny_x = m.x.w + m.x.y;
        culler.ny_y = m.y.w + m.y.y;
        culler.ny_z = m.z.w + m.z.y;
        culler.ny_w = m.w.w + m.w.y;
        //if (allow_test_spheres) {
        let invl = (culler.ny_x * culler.ny_x + culler.ny_y * culler.ny_y
            + culler.ny_z * culler.ny_z)
            .sqrt()
            .recip();
        culler.ny_x *= invl;
        culler.ny_y *= invl;
        culler.ny_z *= invl;
        culler.ny_w *= invl;
        //}
        culler.py_x = m.x.w - m.x.y;
        culler.py_y = m.y.w - m.y.y;
        culler.py_z = m.z.w - m.z.y;
        culler.py_w = m.w.w - m.w.y;
        //if (allow_test_spheres) {
        let invl = (culler.py_x * culler.py_x + culler.py_y * culler.py_y
            + culler.py_z * culler.py_z)
            .sqrt()
            .recip();
        culler.py_x *= invl;
        culler.py_y *= invl;
        culler.py_z *= invl;
        culler.py_w *= invl;
        //}
        culler.nz_x = m.x.w + m.x.z;
        culler.nz_y = m.y.w + m.y.z;
        culler.nz_z = m.z.w + m.z.z;
        culler.nz_w = m.w.w + m.w.z;
        //if (allow_test_spheres) {
        let invl = (culler.nz_x * culler.nz_x + culler.nz_y * culler.nz_y
            + culler.nz_z * culler.nz_z)
            .sqrt()
            .recip();
        culler.nz_x *= invl;
        culler.nz_y *= invl;
        culler.nz_z *= invl;
        culler.nz_w *= invl;
        //}
        culler.pz_x = m.x.w - m.x.z;
        culler.pz_y = m.y.w - m.y.z;
        culler.pz_z = m.z.w - m.z.z;
        culler.pz_w = m.w.w - m.w.z;
        //if (allow_test_spheres) {
        let invl = (culler.pz_x * culler.pz_x + culler.pz_y * culler.pz_y
            + culler.pz_z * culler.pz_z)
            .sqrt()
            .recip();
        culler.pz_x *= invl;
        culler.pz_y *= invl;
        culler.pz_z *= invl;
        culler.pz_w *= invl;
        //}

        culler
    }

    /// Test wether a 3D point lies inside of the frustum
    pub fn test_point(&self, point: Vector3<S>) -> Intersection {
        if self.nx_x * point.x + self.nx_y * point.y + self.nx_z * point.z + self.nx_w >= S::zero()
            && self.px_x * point.x + self.px_y * point.y + self.px_z * point.z + self.px_w
                >= S::zero()
            && self.ny_x * point.x + self.ny_y * point.y + self.ny_z * point.z + self.ny_w
                >= S::zero()
            && self.py_x * point.x + self.py_y * point.y + self.py_z * point.z + self.py_w
                >= S::zero()
            && self.nz_x * point.x + self.nz_y * point.y + self.nz_z * point.z + self.nz_w
                >= S::zero()
            && self.pz_x * point.x + self.pz_y * point.y + self.pz_z * point.z + self.pz_w
                >= S::zero()
        {
            Intersection::Inside
        } else {
            Intersection::Outside
        }
    }

    /// Returns the result of testing the intersection of the frustum with a sphere, defined by a
    /// center point (`center`) and a radius (`radius`).
    ///
    /// This method will distinguish between a partial intersection and a total intersection.
    pub fn test_sphere<T>(&self, sphere: T) -> Intersection
    where
        T: Into<Sphere<S>>,
    {
        let sphere = sphere.into();

        let mut inside = true;
        let mut dist;
        dist = self.nx_x * sphere.center.x + self.nx_y * sphere.center.y
            + self.nx_z * sphere.center.z + self.nx_w;
        if dist >= -sphere.radius {
            inside &= dist >= sphere.radius;
            dist = self.px_x * sphere.center.x + self.px_y * sphere.center.y
                + self.px_z * sphere.center.z + self.px_w;
            if dist >= -sphere.radius {
                inside &= dist >= sphere.radius;
                dist = self.ny_x * sphere.center.x + self.ny_y * sphere.center.y
                    + self.ny_z * sphere.center.z + self.ny_w;
                if dist >= -sphere.radius {
                    inside &= dist >= sphere.radius;
                    dist = self.py_x * sphere.center.x + self.py_y * sphere.center.y
                        + self.py_z * sphere.center.z + self.py_w;
                    if dist >= -sphere.radius {
                        inside &= dist >= sphere.radius;
                        dist = self.nz_x * sphere.center.x + self.nz_y * sphere.center.y
                            + self.nz_z * sphere.center.z
                            + self.nz_w;
                        if dist >= -sphere.radius {
                            inside &= dist >= sphere.radius;
                            dist = self.pz_x * sphere.center.x + self.pz_y * sphere.center.y
                                + self.pz_z * sphere.center.z
                                + self.pz_w;
                            if dist >= -sphere.radius {
                                inside &= dist >= sphere.radius;
                                return if inside {
                                    Intersection::Inside
                                } else {
                                    Intersection::Partial
                                };
                            }
                        }
                    }
                }
            }
        }

        Intersection::Outside
    }

    /// Tests wether a given axis aligned bounding box intersects with the Frustum. There is a
    /// distinction between partial intersection and full intersection, which is given by the
    /// values of the `Intersection` enum.
    pub fn test_bounding_box<T>(&self, aab: T) -> Intersection
    where
        T: Into<BoundingBox<S>>,
    {
        let aab = aab.into();
        let mut inside = true;
        if self.nx_x * if self.nx_x < S::zero() {
            aab.min.x
        } else {
            aab.max.x
        } + self.nx_y * if self.nx_y < S::zero() {
            aab.min.y
        } else {
            aab.max.y
        } + self.nx_z * if self.nx_z < S::zero() {
            aab.min.z
        } else {
            aab.max.z
        } >= -self.nx_w
        {
            inside &= self.nx_x * if self.nx_x < S::zero() {
                aab.max.x
            } else {
                aab.min.x
            } + self.nx_y * if self.nx_y < S::zero() {
                aab.max.y
            } else {
                aab.min.y
            } + self.nx_z * if self.nx_z < S::zero() {
                aab.max.z
            } else {
                aab.min.z
            } >= -self.nx_w;
            if self.px_x * if self.px_x < S::zero() {
                aab.min.x
            } else {
                aab.max.x
            } + self.px_y * if self.px_y < S::zero() {
                aab.min.y
            } else {
                aab.max.y
            } + self.px_z * if self.px_z < S::zero() {
                aab.min.z
            } else {
                aab.max.z
            } >= -self.px_w
            {
                inside &= self.px_x * if self.px_x < S::zero() {
                    aab.max.x
                } else {
                    aab.min.x
                } + self.px_y * if self.px_y < S::zero() {
                    aab.max.y
                } else {
                    aab.min.y
                } + self.px_z * if self.px_z < S::zero() {
                    aab.max.z
                } else {
                    aab.min.z
                } >= -self.px_w;
                if self.ny_x * if self.ny_x < S::zero() {
                    aab.min.x
                } else {
                    aab.max.x
                } + self.ny_y * if self.ny_y < S::zero() {
                    aab.min.y
                } else {
                    aab.max.y
                } + self.ny_z * if self.ny_z < S::zero() {
                    aab.min.z
                } else {
                    aab.max.z
                } >= -self.ny_w
                {
                    inside &= self.ny_x * if self.ny_x < S::zero() {
                        aab.max.x
                    } else {
                        aab.min.x
                    } + self.ny_y * if self.ny_y < S::zero() {
                        aab.max.y
                    } else {
                        aab.min.y
                    } + self.ny_z * if self.ny_z < S::zero() {
                        aab.max.z
                    } else {
                        aab.min.z
                    } >= -self.ny_w;
                    if self.py_x * if self.py_x < S::zero() {
                        aab.min.x
                    } else {
                        aab.max.x
                    } + self.py_y * if self.py_y < S::zero() {
                        aab.min.y
                    } else {
                        aab.max.y
                    } + self.py_z * if self.py_z < S::zero() {
                        aab.min.z
                    } else {
                        aab.max.z
                    } >= -self.py_w
                    {
                        inside &= self.py_x * if self.py_x < S::zero() {
                            aab.max.x
                        } else {
                            aab.min.x
                        } + self.py_y * if self.py_y < S::zero() {
                            aab.max.y
                        } else {
                            aab.min.y
                        } + self.py_z * if self.py_z < S::zero() {
                            aab.max.z
                        } else {
                            aab.min.z
                        } >= -self.py_w;
                        if self.nz_x * if self.nz_x < S::zero() {
                            aab.min.x
                        } else {
                            aab.max.x
                        } + self.nz_y * if self.nz_y < S::zero() {
                            aab.min.y
                        } else {
                            aab.max.y
                        } + self.nz_z * if self.nz_z < S::zero() {
                            aab.min.z
                        } else {
                            aab.max.z
                        } >= -self.nz_w
                        {
                            inside &= self.nz_x * if self.nz_x < S::zero() {
                                aab.max.x
                            } else {
                                aab.min.x
                            }
                                + self.nz_y * if self.nz_y < S::zero() {
                                    aab.max.y
                                } else {
                                    aab.min.y
                                }
                                + self.nz_z * if self.nz_z < S::zero() {
                                    aab.max.z
                                } else {
                                    aab.min.z
                                } >= -self.nz_w;
                            if self.pz_x * if self.pz_x < S::zero() {
                                aab.min.x
                            } else {
                                aab.max.x
                            }
                                + self.pz_y * if self.pz_y < S::zero() {
                                    aab.min.y
                                } else {
                                    aab.max.y
                                }
                                + self.pz_z * if self.pz_z < S::zero() {
                                    aab.min.z
                                } else {
                                    aab.max.z
                                } >= -self.pz_w
                            {
                                inside &= self.pz_x * if self.pz_x < S::zero() {
                                    aab.max.x
                                } else {
                                    aab.min.x
                                }
                                    + self.pz_y * if self.pz_y < S::zero() {
                                        aab.max.y
                                    } else {
                                        aab.min.y
                                    }
                                    + self.pz_z * if self.pz_z < S::zero() {
                                        aab.max.z
                                    } else {
                                        aab.min.z
                                    } >= -self.pz_w;
                                return if inside {
                                    Intersection::Inside
                                } else {
                                    Intersection::Partial
                                };
                            }
                        }
                    }
                }
            }
        }

        Intersection::Outside
    }
}

// impl<S> From<(Vector3<S>, Vector3<S>)> for BoundingBox<S> {
//     #[inline]
//     fn from((min, max): (Vector3<S>, Vector3<S>)) -> Self {
//         Self { min, max }
//     }
// }
//
// impl<S> From<(Vector3<S>, S)> for Sphere<S> {
//     #[inline]
//     fn from((center, radius): (Vector3<S>, S)) -> Self {
//         Self { center, radius }
//     }
// }
