pub use crate::prelude::*;

use anyhow::anyhow;
use bevy::render::texture::CompressedImageFormats;
use chrono::NaiveDate;
use resvg::usvg::{fontdb, Tree, TreeParsing, TreeTextToPath};
use serde::{Deserialize, Serialize};

pub struct NewsPlugin;

impl Plugin for NewsPlugin {
    fn build(&self, app: &mut App) {
        app.init_tracked_resource::<NewsResource>()
            .register_async_event::<NewsItem>()
            .add_systems(PostStartup, check_loaded_news)
            .add_systems(PostStartup, get_latest_news.after(check_loaded_news))
            .add_systems(Update, update_news_items);
    }
}

#[derive(Debug, PartialEq, Resource, Serialize, Deserialize, Default, Clone, MavericContext)]
pub struct NewsResource {
    pub latest: Option<NewsItem>,
    pub is_read: bool,
}

impl TrackableResource for NewsResource {
    const KEY: &'static str = "news";
}

fn check_loaded_news(
    mut news: ResMut<NewsResource>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    let Some(item) = &news.latest else {
        info!("No news loaded");
        return;
    };

    if item.expired() {
        info!("Loaded news is expired");
        news.latest = None;
    } else {
        match create_image_bytes(item, asset_server.as_ref(), &mut images) {
            Ok(()) => {
                info!("Created image bytes for loaded news");
            }
            Err(err) => {
                error!("{err}");
                news.latest = None;
            }
        }
    }
}

fn get_latest_news(writer: AsyncEventWriter<NewsItem>) {
    info!("Getting latest news");
    spawn_and_run(get_latest_news_async(writer));
}

async fn get_latest_news_async(writer: AsyncEventWriter<NewsItem>) {
    let url = "https://steks.net/news.yaml".to_string();

    let res = reqwest::get(url).await;
    let text = match res {
        Ok(response) => response.text().await,
        Err(err) => {
            error!("{err}");
            return;
        }
    };

    let text = match text {
        Ok(text) => text,
        Err(err) => {
            error!("{err}");
            return;
        }
    };

    let item: Result<NewsItem, serde_yaml::Error> = serde_yaml::from_str(&text);

    let item = match item {
        Ok(item) => item,
        Err(err) => {
            error!("{err}");
            return;
        }
    };

    match writer.send(item) {
        Ok(()) => {
            info!("Latest news sent");
        }
        Err(err) => error!("{err}"),
    }
}

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Event, Clone)]
pub struct NewsItem {
    pub title: String,
    pub svg: String,
    pub android_link: String,
    pub ios_link: String,
    pub default_link: String,
    pub date: NaiveDate,
    pub expiry_date: NaiveDate,
}

impl NewsItem {
    pub fn expired(&self) -> bool {
        let today = chrono::offset::Utc::now().date_naive();
        today > self.expiry_date
    }
}

fn update_news_items(
    mut events: EventReader<NewsItem>,
    mut news: ResMut<NewsResource>,
    asset_server: Res<AssetServer>,
    mut images: ResMut<Assets<Image>>,
) {
    'events: for item in events.read() {
        info!("Checking news item");
        if let Some(previous) = &news.as_ref().latest {
            if previous.date >= item.date {
                info!("Latest news is no newer than stored news");
                continue 'events;
            }
        } else if item.expired() {
            info!("Latest news is expired");
            continue 'events;
        }

        match create_image_bytes(item, asset_server.as_ref(), &mut images) {
            Ok(()) => {
                news.latest = Some(item.clone());
                news.is_read = false;
            }
            Err(err) => {
                error!("{err}");
            }
        }
    }
}

fn create_image_bytes(
    item: &NewsItem,
    asset_server: &AssetServer,
    images: &mut ResMut<Assets<Image>>,
) -> Result<(), anyhow::Error> {
    let image_bytes = try_draw_image(&item.svg)?;

    let image = Image::from_buffer(
        &image_bytes,
        bevy::render::texture::ImageType::Extension("png"),
        CompressedImageFormats::empty(),
        true,
        bevy::render::texture::ImageSampler::Default,
        bevy::render::render_asset::RenderAssetUsages::all()
    )?;

    let handle: Handle<Image> = asset_server.get_handle(NEWS_IMAGE_HANDLE).ok_or(anyhow!("Could not get news image handle"))?;

    let im = images.get_or_insert_with(handle, Image::default);
    *im = image;
    Ok(())
}

pub fn try_draw_image(svg: &str) -> Result<Vec<u8>, anyhow::Error> {
    let opt: resvg::usvg::Options = Default::default();

    let mut tree = Tree::from_data(svg.as_bytes(), &opt)?;
    let width = tree.size.width();
    let height = tree.size.height();

    let mut font_database: fontdb::Database = fontdb::Database::new();
    let font_data = include_bytes!(r#"../../fonts/FiraMono-Medium.ttf"#).to_vec();

    font_database.load_font_data(font_data);

    tree.convert_text(&font_database);

    let scale = NEWS_IMAGE_WIDTH_F32 / width;

    //info!("bbox {width} {height} scale {scale}");
    let mut pixmap = resvg::tiny_skia::Pixmap::new(
        (width * scale).floor() as u32,
        (height * scale).floor() as u32,
    )
    .ok_or(anyhow::anyhow!("Could not create pixmap"))?;

    resvg::Tree::render(
        &resvg::Tree::from_usvg(&tree),
        resvg::tiny_skia::Transform::from_scale(scale / 1.0, scale / 1.0),
        &mut pixmap.as_mut(),
    );

    Ok(pixmap.encode_png()?)
}

const NEWS_IMAGE_WIDTH_U32: u32 = 256;
const NEWS_IMAGE_WIDTH_F32: f32 = NEWS_IMAGE_WIDTH_U32 as f32;
const NEWS_IMAGE_HANDLE: &str = "news-image.png";

#[derive(Debug, PartialEq)]
pub struct NewsNode;

impl MavericNode for NewsNode {
    type Context = ();

    fn set_components(commands: SetComponentCommands<Self, Self::Context>) {
        commands
            .ignore_node()
            .ignore_context()
            .insert(NodeBundle {
                style: Style {
                    position_type: PositionType::Absolute,

                    top: Val::Px(50.0),
                    width: Val::Px(NEWS_IMAGE_WIDTH_F32 + (UI_BORDER_WIDTH * 2.0)),
                    margin: UiRect {
                        left: Val::Auto,
                        right: Val::Auto,
                        top: Val::Px(0.0),
                        bottom: Val::Px(0.0),
                    },
                    bottom: Val::Auto,
                    justify_content: JustifyContent::Center,
                    flex_direction: FlexDirection::Column,
                    border: UiRect::all(Val::Px(UI_BORDER_WIDTH)),
                    ..Default::default()
                },
                border_color: BorderColor(Color::BLACK),

                z_index: ZIndex::Global(15),
                ..Default::default()
            })
            .finish()
    }

    fn set_children<R: MavericRoot>(commands: SetChildrenCommands<Self, Self::Context, R>) {
        commands
            .ignore_node()
            .unordered_children_with_context(|context, commands| {
                let node = image_button_node(
                    IconButton::FollowNewsLink,
                    NEWS_IMAGE_HANDLE,
                    NewsButtonStyle,
                    NewsImageStyle,
                );

                commands.add_child("news_image", node, context)
            });
    }
}

#[derive(Debug, Clone, PartialEq)]
pub struct NewsImageStyle;

impl IntoBundle for NewsImageStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {
        Style {
            width: Val::Px(NEWS_IMAGE_WIDTH_F32),
            margin: UiRect::all(Val::Auto),

            ..Default::default()
        }
    }
}

#[derive(Debug, PartialEq, Clone)]
pub struct NewsButtonStyle;

impl IntoBundle for NewsButtonStyle {
    type B = Style;

    fn into_bundle(self) -> Self::B {
        Style {
            width: Val::Px(NEWS_IMAGE_WIDTH_F32),
            ..Default::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::global_ui::NewsItem;

    #[test]
    pub fn go() {
        const NEWS_EXAMPLE: &str = include_str!(r#"./../news.yaml"#);
        let item: Result<NewsItem, serde_yaml::Error> = serde_yaml::from_str(NEWS_EXAMPLE);

        let item: NewsItem = item.expect("Should be able to deserialize latest news");

        let _: Vec<u8> =
            crate::global_ui::try_draw_image(&item.svg).expect("Should be able to draw image");
    }
}
