use image;
use ksni;

use crate::config;

#[derive(Debug)]
pub struct SlpoptTray {
    pub target_language: usize,
    pub vendor: usize,
}

impl ksni::Tray for SlpoptTray {
    fn icon_pixmap(&self) -> Vec<ksni::Icon> {
        let img_png = image::open("slpopt_tray_icon.png").unwrap();
        let mut img = img_png.to_rgba8();
        for image::Rgba(pixel) in img.pixels_mut() {
            *pixel = u32::from_be_bytes(*pixel).rotate_right(8).to_be_bytes();
        }
        vec![ksni::Icon {
            width: img_png.width() as i32,
            height: img_png.height() as i32,
            data: img.into_raw(),
        }]
    }

    //fn icon_name(&self) -> String {
    //    "slpopt".into()
    //}
    fn title(&self) -> String {
        "slpopt".into()
    }
    // **NOTE**: On some system trays, [`Tray::id`] is a required property to avoid unexpected behaviors
    fn id(&self) -> String {
        env!("CARGO_PKG_NAME").into()
    }
    fn menu(&self) -> Vec<ksni::MenuItem<Self>> {
        use ksni::menu::*;
        let mut vendor_vec = Vec::new();
        for i in config::VENDOR {
            vendor_vec.push(RadioItem {
                label: i.into(),
                ..Default::default()
            });
        }
        let mut language_vec = Vec::new();
        for i in config::get_config().language.iter() {
            language_vec.push(RadioItem {
                label: i.into(),
                ..Default::default()
            });
        }
        vec![
            RadioGroup {
                selected: self.target_language,
                select: Box::new(|this: &mut Self, current| {
                    this.target_language = current;
                }),
                options: language_vec,
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            RadioGroup {
                selected: self.vendor,
                select: Box::new(|this: &mut Self, current| {
                    this.vendor = current;
                }),
                options: vendor_vec,
                ..Default::default()
            }
            .into(),
            MenuItem::Separator,
            StandardItem {
                label: "Exit".into(),
                icon_name: "application-exit".into(),
                activate: Box::new(|_| std::process::exit(0)),
                ..Default::default()
            }
            .into(),
        ]
    }
}
