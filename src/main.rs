use arboard::GetExtLinux;
use input::event::pointer::{ButtonState};
use input::event::PointerEvent;
use input::{Event, Libinput, LibinputInterface};
use libc::{O_RDONLY, O_RDWR, O_WRONLY};

use std::fs::{File, OpenOptions};
use std::os::unix::{fs::OpenOptionsExt, io::OwnedFd};
use std::path::Path;
use std::sync::{OnceLock, Mutex};
use std::time::Duration;

use tokio::{self, select};
use std::sync::mpsc::{self, Receiver};
use eframe::egui;

mod api;



//static TRANS_RESULT: OnceLock<Mutex<Option<String>>> = OnceLock::new(); 

struct Interface;

impl LibinputInterface for Interface {
    fn open_restricted(&mut self, path: &Path, flags: i32) -> Result<OwnedFd, i32> {
        OpenOptions::new()
            .custom_flags(flags)
            .read((flags & O_RDONLY != 0) | (flags & O_RDWR != 0))
            .write((flags & O_WRONLY != 0) | (flags & O_RDWR != 0))
            .open(path)
            .map(|file| file.into())
            .map_err(|err| err.raw_os_error().unwrap())
    }
    fn close_restricted(&mut self, fd: OwnedFd) {
        drop(File::from(fd));
    }
}


struct App {
    res_cx: Receiver<String>,
    cur: String,
    show: bool,
    hide_cx: Receiver<()>
}

impl eframe::App for App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        //let res = TRANS_RESULT.get().unwrap().lock().unwrap();
        let res = self.res_cx.try_recv();
        if res.is_ok() {
            self.cur = res.unwrap();
            self.show = true;
        }
        if self.hide_cx.try_recv().is_ok() {
            self.show = false;
        }
        if self.show {
            egui::Window::new("translate result").show(ctx, |ui| {
                ui.label(&self.cur);
            });
        }
        ctx.send_viewport_cmd(egui::ViewportCommand::MousePassthrough(true));
        //ctx.send_viewport_cmd(egui::ViewportCommand::Visible(false));

        ctx.request_repaint();
    }
}
//test text: In hindsight, we can tell you: it’s more challenging than it seems. Users attempting to travel 5 years back with guix time-machine are (or were) unavoidably going to hit bumps on the road—a real problem because that’s one of the use cases Guix aims to support well, in particular in a reproducible research context.

fn main() {
    let (event_tx, event_cx) = mpsc::channel();
    let (res_tx, res_cx) = mpsc::channel();
    let (hide_tx, hide_cx) = mpsc::channel();
    //let _ = TRANS_RESULT.set(Mutex::new(None));
    std::thread::spawn(move || {
        let mut input = Libinput::new_with_udev(Interface);
        input.udev_assign_seat("seat0").unwrap();
        loop {
            input.dispatch().unwrap();
            let btn_ev = input.find(|event| {
                if let Event::Pointer(PointerEvent::Button(e)) = event {
                    if e.button() == 272 && e.button_state() == ButtonState::Released {
                        return true;
                    }
                }
                return false;
            });
            if btn_ev.is_some() {
                event_tx.send(()).unwrap();
            }
        }
    });

    std::thread::spawn(move || {
        let mut clipboard = arboard::Clipboard::new().unwrap();
        let rt = tokio::runtime::Builder::new_multi_thread().enable_all().build().unwrap();

        let (reset_tx, mut reset_cx) = tokio::sync::mpsc::channel(1);
        
        std::thread::spawn(move ||{
            let rt = tokio::runtime::Builder::new_current_thread().enable_all().build().unwrap();
            rt.block_on(async {
                loop {
                    select! {
                        _ = tokio::time::sleep(Duration::from_secs(3)) => {
                            hide_tx.send(()).unwrap();
                        }
                        _ = reset_cx.recv() => {
                            continue;
                        }
                    }
                }

            })

        });
        let api = api::TransRequest::from_config();
        rt.block_on(async {
            loop {
                let _ = event_cx.recv().unwrap();
                
                let text = clipboard.get().clipboard(arboard::LinuxClipboardKind::Primary).text().unwrap();
                //println!("fan yi qing qiu {}", text);
                
                let res = api.request(text.as_str()).await;

                reset_tx.send(()).await.unwrap();
                //println!("fan yi xiang ying {}", res);
                //res_tx.send(res).unwrap();
                //let mut setter = TRANS_RESULT.get().unwrap().lock().unwrap();
                //*setter = Some(res);
                res_tx.send(res).unwrap();

            }
        });
    });

    let mut options = eframe::NativeOptions::default();
    options.viewport.mouse_passthrough = Some(true);
    options.viewport.decorations = Some(false);
    let _ = eframe::run_native(
        "My egui App",
        options,
        Box::new(|cc| {
            let mut fonts = egui::FontDefinitions::default();
            fonts.font_data.insert("wqy".to_owned(), egui::FontData::from_static(include_bytes!("/home/cc1/.local/share/fonts/wenquanyi/wqy-microhei/wqy-microhei.ttc")));
            fonts.families.get_mut(&egui::FontFamily::Proportional).unwrap().insert(0, "wqy".to_owned());
            fonts.families.get_mut(&egui::FontFamily::Monospace).unwrap().push("wqy".to_owned());
            cc.egui_ctx.set_fonts(fonts);
            // This gives us image support:
            egui_extras::install_image_loaders(&cc.egui_ctx);

            Box::new(App{
                res_cx,
                cur: "".to_string(),
                show: false,
                hide_cx
            })
        }),
    );


}