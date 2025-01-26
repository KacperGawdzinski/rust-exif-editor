use gtk::prelude::*;
use gtk::{
    glib, Application, ApplicationWindow, Box, Button, Entry, FileChooserDialog, Orientation,
    Picture, ResponseType,
};
use exif::{Field, In, Tag, Value};
use exif::experimental::Writer;
use rexiv2::Metadata;

const APP_ID: &str = "org.gtk_rs.Exif_Rust";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let button = Button::builder()
        .label("Press me!")
        .margin_top(12)
        .margin_bottom(12)
        .margin_start(12)
        .margin_end(12)
        .build();

    button.connect_clicked(|button| {
        button.set_label("Hello World!");
    });

    let picture = Picture::new();
    picture.set_hexpand(true);
    picture.set_vexpand(true);

    let file_button = Button::with_label("Select Image");
    file_button.connect_clicked({
        let picture = picture.clone();
        move |_| {
            let file_dialog = FileChooserDialog::new(
                Some("Select an Image"),
                None::<&ApplicationWindow>,
                gtk::FileChooserAction::Open,
                &[("Close", gtk::ResponseType::Accept)],
            );

            file_dialog.add_buttons(&[
                ("Cancel", ResponseType::Cancel),
                ("Open", ResponseType::Accept),
            ]);

            file_dialog.connect_response({
                let picture = picture.clone();
                move |dialog, response| {
                    if response == ResponseType::Accept {
                        if let Some(file_path) = dialog.file().and_then(|f| f.path()) {
                            if let Some(path_str) = file_path.to_str() {
                                let pic = gio::File::for_path(path_str);
                                picture.set_file(Some(&pic));

                                // let file = std::fs::File::open(path_str).unwrap();
                                // let mut bufreader = std::io::BufReader::new(&file);
                                // let exifreader = exif::Reader::new();
                                // let exif =
                                //     exifreader.read_from_container(&mut bufreader).expect("xd");
                                // for f in exif.fields() {
                                //     println!(
                                //         "{} {} {}",
                                //         f.tag,
                                //         f.ifd_num,
                                //         f.display_value().with_unit(&exif)
                                //     );
                                // }
                                 
                                // let image_desc = Field {
                                //     tag: Tag::SceneType,
                                //     ifd_num: In::PRIMARY,
                                //     value: Value::Ascii(vec![b"RATATATA".to_vec()]),
                                // };
                                // let mut writer = Writer::new();
                                // let mut buf = std::io::Cursor::new(Vec::new());
                                // writer.push_field(&image_desc);
                                // writer.write(&mut buf, false);

                                let mut metadata = Metadata::new_from_path(path_str).unwrap();
                                println!("{}", metadata.get_exif_tags().unwrap()[0]);
                                metadata.set_tag_string("Exif.Image.Artist", "John Doe").unwrap();
                                metadata.save_to_file(path_str).unwrap();
                            }
                        }
                    }
                    dialog.close();
                }
            });

            file_dialog.show();
        }
    });

    let input_box = Box::new(Orientation::Vertical, 6);
    for _ in 0..10 {
        let entry = Entry::builder().margin_start(12).margin_end(12).build();
        input_box.append(&entry);
    }

    input_box.append(&button);
    input_box.append(&file_button);

    let main_box = Box::new(Orientation::Horizontal, 12);
    main_box.set_margin_top(12);
    main_box.set_margin_bottom(12);
    main_box.set_margin_start(12);
    main_box.set_margin_end(12);
    main_box.append(&picture);
    main_box.append(&input_box);

    let settings = gtk::Settings::default().unwrap();
    settings.set_gtk_application_prefer_dark_theme(true);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("EXIF Rust tool")
        .child(&main_box)
        .default_width(800)
        .default_height(800)
        .resizable(true)
        .build();

    window.present();
}
