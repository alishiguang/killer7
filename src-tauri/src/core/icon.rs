//! 程序化生成菜单栏图标。
//!
//! Swift 版用的是 SF Symbol 的 `bird`；Rust 版不引入二进制资源，直接用像素运算
//! 画一只风格化的小鸟，输出 macOS 模板图（纯黑 + Alpha，由系统自动适配深浅色）。

/// 生成图标像素数据。
///
/// 返回 `(rgba 字节, 宽, 高)`，尺寸为 `size`。
pub fn bird_rgba(size: u32) -> (Vec<u8>, u32, u32) {
    let mut pixels = vec![0u8; (size * size * 4) as usize];
    let s = size as f32;

    // 各部件在归一化坐标系下的位置。
    let body = Ellipse::new(0.50, 0.64, 0.32, 0.23);
    let head = Ellipse::new(0.77, 0.36, 0.19, 0.19);
    let wing = Ellipse::new(0.46, 0.62, 0.19, 0.11);
    let beak = [
        Point::new(0.93, 0.34),
        Point::new(1.06, 0.38),
        Point::new(0.93, 0.45),
    ];
    let tail = [
        Point::new(0.18, 0.55),
        Point::new(0.03, 0.34),
        Point::new(0.25, 0.42),
    ];

    for y in 0..size {
        for x in 0..size {
            let fx = (x as f32 + 0.5) / s;
            let fy = (y as f32 + 0.5) / s;

            let mut covered = false;
            for ellipse in [&body, &head, &wing] {
                if ellipse.contains(fx, fy) {
                    covered = true;
                    break;
                }
            }
            if !covered {
                covered = point_in_triangle(fx, fy, &beak) || point_in_triangle(fx, fy, &tail);
            }

            if covered {
                let offset = ((y * size + x) * 4) as usize;
                pixels[offset] = 0;
                pixels[offset + 1] = 0;
                pixels[offset + 2] = 0;
                pixels[offset + 3] = 255;
            }
        }
    }

    (pixels, size, size)
}

/// 椭圆。
#[derive(Clone, Copy)]
struct Ellipse {
    cx: f32,
    cy: f32,
    rx: f32,
    ry: f32,
}

impl Ellipse {
    fn new(cx: f32, cy: f32, rx: f32, ry: f32) -> Self {
        Self { cx, cy, rx, ry }
    }

    fn contains(&self, x: f32, y: f32) -> bool {
        let dx = (x - self.cx) / self.rx;
        let dy = (y - self.cy) / self.ry;
        dx * dx + dy * dy <= 1.0
    }
}

/// 二维点。
#[derive(Clone, Copy)]
struct Point {
    x: f32,
    y: f32,
}

impl Point {
    fn new(x: f32, y: f32) -> Self {
        Self { x, y }
    }
}

/// 判断点是否落在三角形内（重心坐标法）。
fn point_in_triangle(p: f32, q: f32, tri: &[Point; 3]) -> bool {
    let (a, b, c) = (tri[0], tri[1], tri[2]);
    let d = (b.y - c.y) * (a.x - c.x) + (c.x - b.x) * (a.y - c.y);
    if d.abs() < f32::EPSILON {
        return false;
    }
    let a1 = ((b.y - c.y) * (p - c.x) + (c.x - b.x) * (q - c.y)) / d;
    let b1 = ((c.y - a.y) * (p - c.x) + (a.x - c.x) * (q - c.y)) / d;
    let c1 = 1.0 - a1 - b1;
    a1 >= 0.0 && b1 >= 0.0 && c1 >= 0.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn produces_non_empty_icon() {
        let (pixels, w, h) = bird_rgba(22);
        assert_eq!(pixels.len(), (w * h * 4) as usize);
        assert!(pixels.contains(&255), "图标不应为空");
    }

    #[test]
    fn triangle_hit_test() {
        let tri = [
            Point::new(0.0, 0.0),
            Point::new(1.0, 0.0),
            Point::new(0.0, 1.0),
        ];
        assert!(point_in_triangle(0.2, 0.2, &tri));
        assert!(!point_in_triangle(0.9, 0.9, &tri));
    }
}
