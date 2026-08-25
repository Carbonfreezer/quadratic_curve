//! Contains the math for the quadratic interpolation polynomial.

pub type QuadPoint = [f32; 2];

/// How many support points to we want to deliver.
const SUPPORT_POINTS: usize = 100;

/// The stepsize for every point.
const STEP_SIZE: f32 = 2.0 / SUPPORT_POINTS as f32;

/// Minimum distance in x direction..
const MIN_GAP: f32 = 0.02;

/// A command to set an index.
#[derive(Debug, Clone)]
pub struct PointSettingCommand {
    pub index_to_set: usize,
    pub point: QuadPoint,
}

/// The logic that fits a quadratic curve to three control points.
pub struct QuadraticFitter {
    /// The three control points for the curve.
    base_points: [QuadPoint; 3],
    /// The three factors for the interpolation polynomial.
    factor: [f32; 3],
}

impl Default for QuadraticFitter {
    /// Pase parable
    fn default() -> Self {
        let mut res = Self {
            base_points: [[-1.0, 1.0], [0.0, 0.0], [1.0, 1.0]],
            factor: [0.0, 0.0, 0.0],
        };
        res.update_polynomial();
        res
    }
}

impl QuadraticFitter {
    /// Recomputes the three polynomial factors from the base points.
    fn update_polynomial(&mut self) {
        self.factor[0] = 1.0
            / ((self.base_points[0][0] - self.base_points[1][0])
                * (self.base_points[0][0] - self.base_points[2][0]));

        self.factor[1] = 1.0
            / ((self.base_points[1][0] - self.base_points[0][0])
                * (self.base_points[1][0] - self.base_points[2][0]));

        self.factor[2] = 1.0
            / ((self.base_points[2][0] - self.base_points[0][0])
                * (self.base_points[2][0] - self.base_points[1][0]));
    }

    /// The distance between two points squared.
    fn point_dist_sq(p0: QuadPoint, p1: QuadPoint) -> f32 {
        (p0[0] - p1[0]).powi(2) + (p0[1] - p1[1]).powi(2)
    }

    /// Gets the index of the closest point and returns it if the distance is not larger than the range.
    pub fn closest_point(&self, point: QuadPoint, max_range: f32) -> Option<usize> {
        let max_dist_sq = max_range * max_range;
        self.base_points
            .iter()
            .enumerate()
            .map(|(index, candidate)| (index, Self::point_dist_sq(*candidate, point)))
            .filter(|(_, dist_sq)| *dist_sq < max_dist_sq)
            .min_by(|a, b| a.1.total_cmp(&b.1))
            .map(|(index, _)| index)
    }

    /// Applies the movement command conditionally, makes sure that points stay in the -1.0, 1.0 range
    /// and do not overtake each other and do not get too close x position wise.
    pub fn apply_command(&mut self, command: PointSettingCommand) {
        let index = command.index_to_set;

        // We keep things sorted.
        let lower = match index {
            0 => -1.0,
            _ => self.base_points[index - 1][0] + MIN_GAP,
        };
        let upper = match index {
            2 => 1.0,
            _ => self.base_points[index + 1][0] - MIN_GAP,
        };

        let mut point = command.point;
        point[0] = point[0].clamp(lower, upper);
        point[1] = point[1].clamp(-1.0, 1.0);
        self.base_points[index] = point;
        self.update_polynomial();
    }

    /// The quadratic function that goes through the three control points.
    fn evaluate_function(&self, x: f32) -> f32 {
        let l0 = (x - self.base_points[1][0]) * (x - self.base_points[2][0]) * self.factor[0];
        let l1 = (x - self.base_points[0][0]) * (x - self.base_points[2][0]) * self.factor[1];
        let l2 = (x - self.base_points[0][0]) * (x - self.base_points[1][0]) * self.factor[2];

        l0 * self.base_points[0][1] + l1 * self.base_points[1][1] + l2 * self.base_points[2][1]
    }

    /// Asks for the base points we want to draw.
    pub fn get_base_points(&self) -> [QuadPoint; 3] {
        self.base_points
    }

    /// Gets the line with all the support points.
    pub fn get_line_points(&self) -> impl Iterator<Item = QuadPoint> + '_ {
        (0..=SUPPORT_POINTS)
            .map(|i| i as f32 * STEP_SIZE - 1.0)
            .map(|x| [x, self.evaluate_function(x)])
    }
}
