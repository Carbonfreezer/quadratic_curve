//! Contains the math for the quadratic interpolation polynomial.

pub type QuadPoint = [f32; 2];

/// How many support points to we want to deliver.
const SUPPORT_POINTS : usize = 100;

/// The stepsize for every point.
const STEP_SIZE : f32 = 2.0 / SUPPORT_POINTS as f32;


/// Minimum distance in x direction..
const MIN_GAP: f32 = 0.02;



/// A command to set an index.
#[derive(Debug, Clone)]
pub struct PointSettingCommand {
    pub index_to_set: usize,
    pub point: QuadPoint,
}

/// The quadratic fitter.
pub struct QuadraticFitter {
    base_points: [QuadPoint; 3],
}

impl QuadraticFitter {
    pub fn new() -> Self {
        Self {
            base_points: [[-1.0, 1.0], [0.0, 0.0], [1.0, 1.0]],
        }
    }

    /// The distance to the point.
    fn point_dist_sq(p0: QuadPoint, p1: QuadPoint) -> f32 {
        (p0[0] - p1[0]).powi(2) + (p0[1] - p1[1]).powi(2)
    }

    /// Gets the index of the closest point and generates the setting command.
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

    pub fn apply_command(&mut self, command: PointSettingCommand) {
        let index = command.index_to_set;


        // We keep things sorted.
        let lower = match index {
            0 => -1.0,
            _ => self.base_points[index - 1][0] + MIN_GAP,
        };
        let upper = match index {
            2 =>  1.0,
            _ => self.base_points[index + 1][0] - MIN_GAP,
        };

        let mut point = command.point;
        point[0] = point[0].clamp(lower, upper);
        self.base_points[index] = point;
    }

    /// Evaluates the functions.
    fn evaluate_function(&self, x: f32) -> f32 {
        let l0 = (x - self.base_points[1][0]) * (x - self.base_points[2][0])
            / ((self.base_points[0][0] - self.base_points[1][0])
                * (self.base_points[0][0] - self.base_points[2][0]));

        let l1 = (x - self.base_points[0][0]) * (x - self.base_points[2][0])
            / ((self.base_points[1][0] - self.base_points[0][0])
            * (self.base_points[1][0] - self.base_points[2][0]));

        let l2 = (x - self.base_points[0][0]) * (x - self.base_points[1][0])
            / ((self.base_points[2][0] - self.base_points[0][0])
            * (self.base_points[2][0] - self.base_points[1][0]));


        l0 * self.base_points[0][1] + l1 * self.base_points[1][1] + l2 * self.base_points[2][1]
    }

    /// Asks for the base points we want to draw.
    pub fn get_base_points(&self) -> [QuadPoint; 3] {
        self.base_points
    }

    /// Gets the line with all the support points.
    pub fn get_line_points(&self) -> impl Iterator<Item =QuadPoint> + '_ {
        (0..=SUPPORT_POINTS).map(|i| i as f32 * STEP_SIZE - 1.0).map(|x| [x, self.evaluate_function(x)])
    }
}
