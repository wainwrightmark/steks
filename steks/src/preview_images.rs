use crate::prelude::*;
use bevy::render::texture::CompressedImageFormats;

use maveric::prelude::*;
use steks_common::images::prelude::{Dimensions, OverlayChooser};
use strum::EnumIs;

pub struct PreviewImagePlugin;

impl Plugin for PreviewImagePlugin {
    fn build(&self, app: &mut App) {
        app.add_systems(Startup, set_up_preview_image)
            .add_systems(Update, update_preview_images);
    }
}

const PREVIEW_IMAGE_SIZE_U32: u32 = 256;
const PREVIEW_IMAGE_SIZE_F32: f32 = PREVIEW_IMAGE_SIZE_U32 as f32;
pub const PREVIEW_IMAGE_ASSET_PATH: &str = "images/preview.png";

#[derive(Debug, Clone, Copy, PartialEq, Eq, EnumIs)]
pub enum PreviewImage {
    PB,
    WR,
}

fn set_up_preview_image(asset_server: Res<AssetServer>) {
    let handle: Handle<Image> = asset_server.load(PREVIEW_IMAGE_ASSET_PATH);
    std::mem::forget(handle);
}

fn update_preview_images(
    mut images: ResMut<Assets<Image>>,
    ui_state: Res<GlobalUiState>,
    pbs: Res<PersonalBests>,
    wrs: Res<WorldRecords>,
    current_level: Res<CurrentLevel>,
) {
    if !ui_state.is_changed() && !current_level.is_changed() {
        return;
    }

    let GlobalUiState::MenuClosed(GameUIState::Preview(preview)) = ui_state.as_ref() else {
        return;
    };

    let LevelCompletion::Complete { score_info } = current_level.completion else {
        return;
    };

    let handle = images.get_handle(PREVIEW_IMAGE_ASSET_PATH);

    let Some(im) = images.get_mut(&handle) else {
        return;
    };

    let mut clear = false;

    match preview {
        PreviewImage::PB => {
            if let Some(pb) = pbs.map.get(&score_info.hash) {
                match game_to_image(pb.image_blob.as_slice()) {
                    Ok(image) => {
                        *im = image;
                    }
                    Err(err) => error!("{err}"),
                }
            } else {
                clear = true;
            }
        }
        PreviewImage::WR => {
            if let Some(wr) = wrs.map.get(&score_info.hash) {
                match game_to_image(wr.image_blob.as_slice()) {
                    Ok(image) => {
                        *im = image;
                    }
                    Err(err) => error!("{err}"),
                }
            } else {
                clear = true;
            }
        }
    }

    if clear {
        for pixel in im.data.chunks_exact_mut(4) {
            pixel[0] = 200;
            pixel[1] = 200;
            pixel[2] = 200;
            pixel[3] = 255;
        }
    }
}

fn game_to_image(data: &[u8]) -> Result<Image, anyhow::Error> {
    let image_bytes = steks_common::images::drawing::try_draw_image(
        data,
        &OverlayChooser::no_overlay(),
        Dimensions {
            width: PREVIEW_IMAGE_SIZE_U32,
            height: PREVIEW_IMAGE_SIZE_U32,
        },
    )?;

    Ok(Image::from_buffer(
        &image_bytes,
        bevy::render::texture::ImageType::Extension("png"),
        CompressedImageFormats::empty(),
        true,
    )?)
}

#[derive(Debug, Clone, PartialEq)]
pub struct PreviewImageStyle;

impl IntoBundle for PreviewImageStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {
        Style {
            width: Val::Px(PREVIEW_IMAGE_SIZE_F32 - 1.0),
            height: Val::Px(PREVIEW_IMAGE_SIZE_F32 - 1.0),
            margin: UiRect::all(Val::Auto),

            ..Default::default()
        }
    }
}