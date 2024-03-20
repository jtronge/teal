use gtk4::prelude::*;
use gtk4::subclass::widget::WidgetClassExt;
use gtk4::{Application, ApplicationWindow, GLArea, DrawingArea, Widget};
use gtk4::cairo;
use gtk4::ShortcutsShortcut;
use glib::signal;

pub fn gui<F>(f: F)
where
    F: FnMut(&mut Context, Event) -> bool + 'static,
{
    let app = Application::builder()
        .application_id("org.teal.Teal")
        .build();
    let mut ctx = Context;
    app.connect_activate(move |app| {
        let window = ApplicationWindow::builder()
            .application(app)
            .title("Teal")
            .build();
        let drawing_area = DrawingArea::new();
        drawing_area.set_draw_func(move |area, ctx, a, b| {
            println!("ctx: {:?}", ctx);
            println!("drawing...");
            let (x0, y0, x1, y1) = ctx.clip_extents().unwrap();
            let format = cairo::Format::Rgb24;
            let width = (x1 / 2.0) as i32;
            let height = (y1 / 2.0) as i32;
            let stride = format.stride_for_width(width.try_into().unwrap()).unwrap();
            println!("stride: {}", stride);
            let data = vec![0; (height * stride).try_into().unwrap()];
            let surface = cairo::ImageSurface::create_for_data(data, cairo::Format::Rgb24, width, height, stride).unwrap();
            ctx.set_source_surface(surface, 0.0, 0.0);
            ctx.paint().unwrap();
        });
        window.set_child(Some(&drawing_area));
        window.present();
    });
    app.run();
}

#[derive(Debug)]
pub enum Key {
    ASCII(char),
}

#[derive(Debug)]
pub enum Event {
    KeyPress(Key),
}

pub struct Context;
