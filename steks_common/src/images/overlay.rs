use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree, TreeParsing};

use crate::images::prelude::*;

pub struct OverlayChooser {
    pub options: Vec<Overlay>,
}

pub const DEFAULT_SCALE_MULTIPLIER: f32 = 1.1;
impl OverlayChooser {
    pub fn no_overlay() -> Self {
        Self { options: vec![] }
    }

    pub fn choose_scale_and_overlay(&self, h_scale: f32, v_scale: f32) -> (f32, Option<&Overlay>) {
        //println!("h: {h_scale} v: {v_scale}");
        if self.options.is_empty() {
            return (h_scale.max(v_scale) * DEFAULT_SCALE_MULTIPLIER, None);
        }

        let mut result = (f32::MAX, None);

        for ov in self.options.iter() {
            let scale = match ov.ratio {
                Ratio::WiderThanTall(r) => (r * h_scale).max(v_scale),
                Ratio::TallerThanWide(r) => (r * v_scale).max(h_scale),
            };

            //println!("Scale: {scale}");

            let scale = scale * DEFAULT_SCALE_MULTIPLIER;

            if scale < result.0 {
                result = (scale, Some(ov))
            }
        }

        return result;
    }
}

#[derive(Debug, Clone)]
pub struct Overlay {
    pub h_placement: HorizontalPlacement,
    pub v_placement: VerticalPlacement,

    pub ratio: Ratio,

    pub bytes: &'static [u8],
}

#[derive(Debug, Clone, Copy)]
pub enum Ratio {
    WiderThanTall(f32),
    TallerThanWide(f32),
}

impl Overlay {
    pub fn try_include(
        &self,
        pixmap: &mut Pixmap,
        opt: &Options,
        dimensions: Dimensions,
    ) -> Result<(), anyhow::Error> {
        let logo_tree = Tree::from_data(self.bytes, &opt)?;

        let logo_scale = (dimensions.width as f32 / logo_tree.size.width() as f32)
            .min(dimensions.height as f32 / logo_tree.size.height() as f32);

        let x_offset = self
            .h_placement
            .get_x(dimensions.width as f32, logo_tree.size.width() * logo_scale);
        let y_offset = self.v_placement.get_y(
            dimensions.height as f32,
            logo_tree.size.height() * logo_scale,
        );
        let transform =
            Transform::from_scale(logo_scale, logo_scale).post_translate(x_offset, y_offset);

        resvg::Tree::render(
            &resvg::Tree::from_usvg(&logo_tree),
            transform,
            &mut pixmap.as_mut(),
        );
        Ok(())
    }
}
