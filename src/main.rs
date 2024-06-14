use input::event::pointer::ButtonState;
use input::event::PointerEvent;
use input::{Event, Libinput, LibinputInterface};
use libc::{O_RDONLY, O_RDWR, O_WRONLY};

use std::fs::{File, OpenOptions};
use std::os::unix::{fs::OpenOptionsExt, io::OwnedFd};
use std::path::Path;
use std::time::Duration;

use std::sync::mpsc;
use tokio::{self, select};

mod api;

use gtk::prelude::*;
use gtk::{glib, Application, ApplicationWindow};

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

//test text: In hindsight, we can tell you: it’s more challenging than it seems. Users attempting to travel 5 years back with guix time-machine are (or were) unavoidably going to hit bumps on the road—a real problem because that’s one of the use cases Guix aims to support well, in particular in a reproducible research context.

fn main() {
    let app = Application::builder().application_id("org.slpopt").build();
    app.connect_activate(build_ui);
    app.run();
}

fn build_ui(app: &Application) {
    let (event_tx, event_cx) = mpsc::channel();
    let (res_tx, mut res_cx) = tokio::sync::mpsc::channel(1);
    let (hide_tx, mut hide_cx) = tokio::sync::mpsc::channel(1);
    let (reset_tx, mut reset_cx) = tokio::sync::mpsc::channel(1);

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
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        rt.block_on(async {
            loop {
                select! {
                    _ = tokio::time::sleep(Duration::from_secs(3)) => {
                        hide_tx.send(()).await.unwrap();
                    }
                    _ = reset_cx.recv() => {
                        continue;
                    }
                }
            }
        })
    });
    std::thread::spawn(move || {
        //let mut clipboard = arboard::Clipboard::new().unwrap();
        let rt = tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()
            .unwrap();

        let api = api::TransRequest::from_config();
        rt.block_on(async {
            let mut last_text = selection::get_text();
            loop {
                let _ = event_cx.recv().unwrap();
                //println!("mouse left up");
                std::thread::sleep(Duration::from_millis(100));
                let text = selection::get_text();
                //println!("select text: {}", text);
                if text == last_text {
                    continue;
                } else {
                    last_text = text.clone();
                }
                //let text = clipboard.get().clipboard(arboard::LinuxClipboardKind::Primary).text().unwrap();
                //println!("translate request: {}", text);

                let res = api.request(text.as_str()).await;

                reset_tx.send(()).await.unwrap();
                //println!("fan yi xiang ying {}", res);

                res_tx.send(res).await.unwrap();
            }
        });
    });

    let label = gtk::Label::new(None);
    label.set_wrap(true);
    let window = ApplicationWindow::builder()
        .application(app)
        .title("slpopt")
        .default_width(300)
        .default_height(100)
        .decorated(false)
        .resizable(false)
        .child(&label)
        .build();

    window.present();

    window.set_visible(false);

    glib::spawn_future_local(async move {
        loop {
            select! {
                _ = hide_cx.recv() => {
                    window.set_visible(false);
                    //window.destroy();
                }
                res = res_cx.recv() => {
                    //window.present();
                    window.set_visible(true);
                    label.set_text(res.unwrap().as_str());
                }
            }
        }
    });
}
