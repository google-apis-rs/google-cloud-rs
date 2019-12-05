use crate::vision::api;

/// A bounding box, delimiting annotations on images.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BoundingBox {
    /// The X-axis coordinate of the top-left corner.
    pub x: i32,
    /// The Y-axis coordinate of the top-left corner.
    pub y: i32,
    /// The width of the bounding box.
    pub w: i32,
    /// The height of the bounding box.
    pub h: i32,
}

impl BoundingBox {
    /// Creates a new bounding box from:
    /// - x: the X-axis coordinate of the top-left corner.
    /// - y: the Y-axis coordinate of the top-left corner.
    /// - w: the width of the bounding box.
    /// - h: the height of the bounding box.
    pub fn new(x: i32, y: i32, w: i32, h: i32) -> BoundingBox {
        BoundingBox { x, y, w, h }
    }
}

impl From<api::BoundingPoly> for BoundingBox {
    fn from(poly: api::BoundingPoly) -> BoundingBox {
        let (lx, ly, gx, gy) = poly
            .vertices
            .into_iter()
            .fold(None, |acc, el| {
                let api::Vertex { x: vx, y: vy } = el;
                match acc {
                    None => Some((vx, vy, vx, vy)),
                    Some((x, y, w, h)) => Some((vx.min(x), vy.min(y), vx.max(w), vy.max(h))),
                }
            })
            .unwrap();
        BoundingBox::new(lx, ly, gx - lx, gy - ly)
    }
}
