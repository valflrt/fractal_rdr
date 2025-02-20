use cumulative_histogram::{compute_histogram, cumulate_histogram, get_histogram_value};
use image::{Rgb, RgbImage};
use serde::{Deserialize, Serialize};

use crate::{mat::Mat2D, params::FrameParams, F};

pub fn color_raw_image(
    params: &FrameParams,
    coloring_mode: ColoringMode,
    custom_gradient: Option<&Vec<(f32, [u8; 3])>>,
    mut raw_image: Mat2D<F>,
) -> RgbImage {
    let &FrameParams {
        img_width,
        img_height,
        ..
    } = params;

    let mut output_image = RgbImage::new(img_width, img_height);

    let max_v = raw_image.vec.iter().copied().fold(0., F::max);
    let min_v = raw_image.vec.iter().copied().fold(max_v, F::min);

    match coloring_mode {
        ColoringMode::CumulativeHistogram { map } => {
            raw_image.vec.iter_mut().for_each(|v| *v /= max_v);
            let cumulative_histogram = cumulate_histogram(compute_histogram(&raw_image.vec));
            for j in 0..img_height as usize {
                for i in 0..img_width as usize {
                    let value = raw_image[(i, j)];

                    let t = map.apply(get_histogram_value(value, &cumulative_histogram));

                    output_image.put_pixel(i as u32, j as u32, color_mapping(t, custom_gradient));
                }
            }
        }
        ColoringMode::MaxNorm { max, map } => {
            let max = max.unwrap_or(max_v);

            for j in 0..img_height as usize {
                for i in 0..img_width as usize {
                    let value = raw_image[(i, j)];

                    let t = map.apply(value / max);

                    output_image.put_pixel(i as u32, j as u32, color_mapping(t, custom_gradient));
                }
            }
        }
        ColoringMode::MinMaxNorm { min, max, map } => {
            let min = min.unwrap_or(min_v);
            let max = max.unwrap_or(max_v);

            for j in 0..img_height as usize {
                for i in 0..img_width as usize {
                    let value = raw_image[(i, j)];

                    let t = map.apply((value - min) / (max - min));

                    output_image.put_pixel(i as u32, j as u32, color_mapping(t, custom_gradient));
                }
            }
        }
        ColoringMode::BlackAndWhite => {
            for j in 0..img_height as usize {
                for i in 0..img_width as usize {
                    let value = raw_image[(i, j)];
                    output_image.put_pixel(
                        i as u32,
                        j as u32,
                        if value >= 0.95 {
                            Rgb([0, 0, 0])
                        } else {
                            Rgb([255, 255, 255])
                        },
                    );
                }
            }
        }
    };

    output_image
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum ColoringMode {
    CumulativeHistogram {
        map: MapValue,
    },
    MaxNorm {
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<F>,
        map: MapValue,
    },
    MinMaxNorm {
        #[serde(skip_serializing_if = "Option::is_none")]
        min: Option<F>,
        #[serde(skip_serializing_if = "Option::is_none")]
        max: Option<F>,
        map: MapValue,
    },
    BlackAndWhite,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub enum MapValue {
    Linear,
    Squared,
    Powf(F),
}

impl MapValue {
    #[inline]
    pub fn apply(&self, t: F) -> F {
        match self {
            MapValue::Linear => t,
            MapValue::Squared => t * t,
            MapValue::Powf(p) => t.powf(*p),
        }
    }
}

const DEFAULT_GRADIENT: [(f32, [u8; 3]); 8] = [
    (0., [20, 8, 30]),
    (0.1, [160, 30, 200]),
    (0.25, [20, 160, 230]),
    (0.4, [60, 230, 80]),
    (0.55, [255, 230, 20]),
    (0.7, [255, 120, 20]),
    (0.85, [255, 40, 60]),
    (1., [20, 2, 10]),
];

pub fn color_mapping(t: F, custom_gradient: Option<&Vec<(f32, [u8; 3])>>) -> Rgb<u8> {
    fn map(t: F, gradient: &[(f32, [u8; 3])]) -> Rgb<u8> {
        let first = gradient[0];
        let last = gradient.last().unwrap();
        if t <= first.0 as F {
            Rgb(first.1)
        } else if t >= last.0 as F {
            Rgb(last.1)
        } else {
            for i in 0..gradient.len() {
                if gradient[i].0 as F <= t && t <= gradient[i + 1].0 as F {
                    let ratio = (t - gradient[i].0 as F) / (gradient[i + 1].0 - gradient[i].0) as F;
                    let [r1, g1, b1] = gradient[i].1;
                    let [r2, g2, b2] = gradient[i + 1].1;
                    let r = (r1 as F * (1. - ratio) + r2 as F * ratio).clamp(0., 255.) as u8;
                    let g = (g1 as F * (1. - ratio) + g2 as F * ratio).clamp(0., 255.) as u8;
                    let b = (b1 as F * (1. - ratio) + b2 as F * ratio).clamp(0., 255.) as u8;
                    return Rgb([r, g, b]);
                }
            }
            Rgb(last.1)
        }
    }

    if let Some(g) = custom_gradient {
        map(t, g)
    } else {
        map(t, DEFAULT_GRADIENT.as_ref())
    }
}

pub mod cumulative_histogram {
    use crate::F;

    const HISTOGRAM_SIZE: usize = 1000000;

    fn map_f_to_histogram_index(value: F) -> usize {
        ((value * (HISTOGRAM_SIZE - 1) as F) as usize).min(HISTOGRAM_SIZE - 1)
    }

    /// Compute an histogram from normalized values in range
    /// (0, 1).
    pub fn compute_histogram(pixel_values: &[F]) -> Vec<u32> {
        let mut histogram = vec![0; HISTOGRAM_SIZE];

        for &value in pixel_values.iter() {
            histogram[map_f_to_histogram_index(value)] += 1;
        }

        histogram
    }

    /// Computes the cumulative histogram associated with the
    /// histogram provided.
    pub fn cumulate_histogram(histogram: Vec<u32>) -> Vec<F> {
        let total = histogram.iter().sum::<u32>();
        let mut cumulative = vec![0.; HISTOGRAM_SIZE];
        let mut cumulative_sum = 0.;
        for (i, &count) in histogram.iter().enumerate() {
            cumulative_sum += count as F / total as F;
            cumulative[i] = cumulative_sum;
        }

        cumulative
    }

    /// Get the cumulative histogram value from a normalized value
    /// in range (0, 1).
    pub fn get_histogram_value(value: F, cumulative_histogram: &[F]) -> F {
        cumulative_histogram[map_f_to_histogram_index(value)]
    }
}
