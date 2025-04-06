use iced::Color;
use iced::Theme as IcedTheme;
use iced::alignment::{Horizontal, Vertical};
use iced::border::Radius;
use iced::mouse::Button;
use iced::widget::button::{Catalog, Status as BtnStatus};
use iced::widget::text;
use iced::widget::{button, container};
use iced::*;
use iced::{Background, Border, Length};
use serde::{Deserialize, Deserializer};
use std::result::Result;

use super::Event;
pub type Hex = u32;

#[derive(Debug, Clone, Deserialize)]
#[serde(rename = "snake_case")]
#[serde(rename_all = "snake_case")]
pub struct Theme {
    name: String,
    background: Hex,
    account_panel: AccountPanel,
    contact_panel: ContactPanel,
    message_panel: MessagePanel,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AccountPanel {
    #[serde(deserialize_with = "de_from_str")]
    pub text_color: Color,

    #[serde(deserialize_with = "de_from_str")]
    pub btn_color: Color,

    #[serde(deserialize_with = "de_from_str")]
    pub btn_hover_color: Color,

    #[serde(deserialize_with = "de_from_str")]
    pub btn_selected_color: Color,
}

fn de_from_str<'de, D>(deserializer: D) -> Result<Color, D::Error>
where
    D: Deserializer<'de>,
{
    let hex = i32::deserialize(deserializer)?;
    Ok(color!(hex))
}

fn deserialize_into_color(a: i32) {}

#[derive(Debug, Clone, Deserialize)]
pub struct ContactPanel {}

#[derive(Debug, Clone, Deserialize)]
pub struct MessagePanel {}

pub type ContainerStyle = container::Style;

/// returns width, height, padding, and style
pub fn account_panel(
    theme: &IcedTheme,
) -> (
    Length,
    Length,
    Padding,
    impl Fn(&IcedTheme) -> ContainerStyle,
) {
    let style = |theme: &IcedTheme| ContainerStyle {
        text_color: None,
        background: None,
        border: Border {
            color: Color::from_rgb8(0, 255, 0),
            width: 1.0,
            radius: Radius::new(1),
        },
        shadow: iced::Shadow::default(),
    };

    (50.into(), 50.into(), 10.into(), style)
}

pub fn account_btn_size() -> (Length, Length) {
    (50.into(), 50.into())
}

pub fn background() -> () {}

use crate::ui::widget::sss::Style as BtnStyle;
// pub fn account_btn(
//     theme: &IcedTheme,
//     status: BtnStatus,
//     active: bool
// ) ->  BtnStyle {
//
//     if active {
//         BtnStyle {
//             background: Background::Color(Color::from_rgb8(
//                 100, 100, 0,
//             )),
//             border_width: 1.0,
//             border_color: Color::default(),
//             text_color: None,
//
//             // ..Default::default()
//         }
//     } else {
//
//         // BtnStyle:
//         // button::primary(theme, status)
//     }
// }

fn hex_to_iced_color(hex: u32) -> Color {
    let r = hex as u8;
    let g = (hex >> 8) as u8;
    let b = (hex >> 16) as u8;
    let a = (hex >> 24) as u8;
    let a = f32::from(a) / 255.0;
    Color::from_rgba8(r, g, b, a)
}
