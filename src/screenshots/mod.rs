use std::{fs, ops::Neg};

use anyhow::anyhow;
use bevy::prelude::*;
use bevy_prototype_lyon::prelude::{tess::geom::traits::Transformation, *};
use resvg::usvg::{self, NodeExt};

use crate::*;

pub struct ScreenshotPlugin;

impl Plugin for ScreenshotPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(SavedSvg::default())
            .add_event::<SaveSVGEvent>()
            .add_event::<DownloadPngEvent>()
            .add_system_to_stage(CoreStage::Last, save_svg)
            .add_system(download_svg);
    }
}

pub struct SaveSVGEvent {
    pub title: String,
}

pub struct DownloadPngEvent;

#[derive(Resource, Default)]
pub struct SavedSvg(Option<SvgFile>);

pub struct SvgFile {
    pub title: String,
    pub svg: String,
}

fn download_svg(mut events: EventReader<DownloadPngEvent>, saves: Res<SavedSvg>) {
    for _event in events.iter() {
        if let Some(svg) = &saves.0 {
            match string_to_png(&svg.svg) {
                Ok(vec) => {
                    let filename = svg.title.clone() + ".png";
                    info!("downloading {filename}");
                    #[cfg(target_arch = "wasm32")]
                    {
                        crate::wasm::download::download_bytes(filename.into(), vec);
                    }
                    #[cfg(not(target_arch = "wasm32"))]
                    {
                        save_file(filename.into(), vec).expect("Could not save file");
                    }
                    println!("{}", svg.svg)
                }
                Err(err) => {
                    error!("{}", err)
                }
            }
        } else {
            warn!("No Svg to save")
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn save_file(file_name: std::path::PathBuf, bytes: Vec<u8>) -> anyhow::Result<()> {
    fs::write(file_name, bytes)?;

    Ok(())
}

fn save_svg(
    mut events: EventReader<SaveSVGEvent>,
    query: Query<
        (&Transform, &Path, &DrawMode),
        (With<Draggable>, Without<Wall>, Without<Padlock>),
    >,
    mut saves: ResMut<SavedSvg>,
) {
    for event in events.iter() {
        let svg = create_svg(query.iter());
        *saves = SavedSvg(Some(SvgFile {
            title: event.title.clone(),
            svg,
        }))
    }
}

fn string_to_png(str: &str) -> Result<Vec<u8>, anyhow::Error> {
    //println!("{}", str);
    let opt = usvg::Options::default();
    //info!(str);
    let tree = usvg::Tree::from_str(str, &opt)?;
    //info!("Tree Size {:?}", tree.size);
    //info!("Viewbox {:?}", tree.view_box);
    //info!("ViewBox Size {:?}", tree.view_box.rect.size());
    let bounding_box = tree.root.calculate_bbox().unwrap();

    let pixmap_size = bounding_box.to_rect().unwrap().size().to_screen_size(); // tree.size.to_screen_size();
                                                                               //info!("Pixmap size {:?}", pixmap_size);
    let mut pixmap = resvg::tiny_skia::Pixmap::new(pixmap_size.width(), pixmap_size.height())
        .ok_or(anyhow!("Could not create pixmap"))?;

        pixmap.fill(resvg::tiny_skia::Color::from_rgba(BACKGROUND_COLOR.r(), BACKGROUND_COLOR.g(), BACKGROUND_COLOR.b(), BACKGROUND_COLOR.a()).unwrap());
    resvg::render(
        &tree,
        usvg::FitTo::Original,
        resvg::tiny_skia::Transform::from_translate(
            bounding_box.x().neg() as f32,
            bounding_box.y().neg() as f32,
        ),
        pixmap.as_mut(),
    )
    .ok_or(anyhow!("Could not render svg"))?;

    let vec = pixmap.encode_png()?;
    Ok(vec)
}

pub fn create_svg<'a, I: Iterator<Item = (&'a Transform, &'a Path, &'a DrawMode)>>(
    iterator: I,
) -> String {
    let mut str: String = "".to_owned();

    let left = WINDOW_WIDTH * 0.5;
    let top = WINDOW_HEIGHT * 0.5;

    let global_transform = Transform::from_translation(Vec3 {
        x: left,
        y: top,
        z: 0.0,
    });
    let global_transform = global_transform.with_scale(Vec3 {
        x: 1.0,
        y: -1.0,
        z: 1.0,
    });
    let global_transform: TransformWrapper = (&global_transform).into();

    // let mut min_x: f32 = WINDOW_WIDTH;
    // let mut min_y: f32 = WINDOW_HEIGHT;
    // let mut max_x: f32 = 0.;
    // let mut max_y: f32 = 0.;

    str.push('\n');
    for (transform, path, draw_mode) in iterator {
        let tw: TransformWrapper = transform.into();
        let path = path.0.clone().transformed(&tw);
        let path = path.transformed(&global_transform);

        // for event in path.iter(){
        //     // let (p1, p2) =
        //     // match event {
        //     //     tess::path::Event::Begin { at } => (at,at),
        //     //     tess::path::Event::Line { from, to} => (from, to),
        //     //     tess::path::Event::Quadratic { from, ctrl: _, to } => (from, to),
        //     //     tess::path::Event::Cubic { from, ctrl1:_, ctrl2:_, to } => (from, to),
        //     //     tess::path::Event::End { last, first, close:_ } => (last, first),
        //     // };

        //     // for p in [p1,p2]{
        //     //     min_x = min_x.min(p.x);
        //     //     max_x = max_x.max(p.x);
        //     //     min_y = min_y.min(p.y);
        //     //     max_y = max_y.max(p.y);
        //     // }
        // }

        str.push('\n');
        let path_d = format!("{:?}", path);
        let path_style = get_path_style(draw_mode);

        str.push_str(format!(r#"<path {path_style} d={path_d} />"#).as_str());
        str.push('\n');
        str.push('\n');
    }

    // let width = max_x - min_x;
    // let height = max_y - min_y;

    format!(
        r#"<svg
        viewbox = "0 0 {WINDOW_WIDTH} {WINDOW_HEIGHT}"
        xmlns="http://www.w3.org/2000/svg" fill="{}">
        {str}
        </svg>"#, color_to_rgba(color::BACKGROUND_COLOR)
    )
}

fn get_path_style(draw_mode: &DrawMode) -> String {
    match draw_mode {
        DrawMode::Fill(fill_mode) => get_fill_style(fill_mode),
        DrawMode::Stroke(stroke_mode) => get_stroke_style(stroke_mode),
        DrawMode::Outlined {
            fill_mode,
            outline_mode,
        } => format!(
            "{} {}",
            get_fill_style(fill_mode),
            get_stroke_style(outline_mode)
        ),
    }
}

fn get_fill_style(fill_mode: &FillMode) -> String {
    format!(r#"fill = "{}""#, color_to_rgba(fill_mode.color))
}

fn get_stroke_style(stroke_mode: &StrokeMode) -> String {
    format!(r#"stroke = "{}""#, color_to_rgba(stroke_mode.color))
}

fn color_to_rgba(color: Color) -> String {
    let [mut r, mut g, mut b, mut a] = color.as_rgba_f32();
    r *= 255.0;
    g *= 255.0;
    b *= 255.0;
    a *= 255.0;
    format!(
        "#{:02X}{:02X}{:02X}{:02X}",
        r as u8, g as u8, b as u8, a as u8
    )
}

impl Transformation<f32> for TransformWrapper {
    fn transform_point(&self, p: tess::geom::Point<f32>) -> tess::geom::Point<f32> {
        let matrix = self.0.compute_matrix();
        let vec2: Vec2 = Vec2 { x: p.x, y: p.y };
        let vec2 = matrix.transform_point3(vec2.extend(0.0)).truncate();

        tess::geom::Point::<f32>::new(vec2.x, vec2.y)
    }

    fn transform_vector(&self, v: tess::geom::Vector<f32>) -> tess::geom::Vector<f32> {
        let matrix = self.0.compute_matrix();
        let vec2: Vec2 = Vec2 { x: v.x, y: v.y };
        let vec2 = matrix.transform_point3(vec2.extend(0.0)).truncate();

        tess::geom::Vector::<f32>::new(vec2.x, vec2.y)
    }
}

struct TransformWrapper(Transform);

impl From<&Transform> for TransformWrapper {
    fn from(value: &Transform) -> Self {
        Self(*value)
    }
}
