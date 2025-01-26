use gtk::prelude::*;
use gtk::{
    glib, Application, ApplicationWindow, Box, Button, Entry, FileChooserDialog, Orientation,
    Picture, ResponseType,
};
use rexiv2::Metadata;
use serde_json::Value;
use std::env;
use std::fs;
use std::path::Path;

const APP_ID: &str = "org.gtk_rs.Exif_Rust";

fn main() -> glib::ExitCode {
    let app = Application::builder().application_id(APP_ID).build();
    app.connect_activate(build_ui);
    app.run()
}

fn build_ui(app: &Application) {
    let picture = Picture::new();
    picture.set_hexpand(true);
    picture.set_vexpand(true);

    let file_button = Button::with_label("Select Image");
    let search_entry = Entry::builder().placeholder_text("Search keys...").build();
    let value_entry = Entry::builder().placeholder_text("Enter value...").build();
    let save_button = Button::with_label("Save");

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

                                let file = std::fs::File::open(path_str).unwrap();
                                let mut bufreader = std::io::BufReader::new(&file);
                                let exifreader = exif::Reader::new();
                                let exif =
                                    exifreader.read_from_container(&mut bufreader).expect("xd");
                                for f in exif.fields() {
                                    println!(
                                        "{} {} {}",
                                        f.tag,
                                        f.ifd_num,
                                        f.display_value().with_unit(&exif)
                                    );
                                }

                                let metadata = Metadata::new_from_path(path_str).unwrap();
                                println!("{}", metadata.get_exif_tags().unwrap()[0]);
                                metadata
                                    .set_tag_string("Exif.Image.Software", "dik")
                                    .unwrap();
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

    let json_keys: Vec<String> = if Path::new("tags.json").exists() {
        let file_content = fs::read_to_string("tags.json").unwrap_or_default();
        let json: Value = serde_json::from_str(&file_content).unwrap_or_default();

        json.as_array()
            .map(|arr| {
                arr.iter()
                    .filter_map(|item| item.as_object())
                    .filter_map(|obj| obj.get("tag"))
                    .filter_map(|tag| tag.as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default()

        // for (key) in json.as_array().iter().map(f) {
        //     println!("{}", serde_json::to_string_pretty(&key).unwrap());
        // }

        // println!("{}", serde_json::to_string_pretty(&json).unwrap());

        // for (key, value) in json["tag"].as_object().unwrap() {
        //     println!("{}", value);
        // }
        // println!("Please call {} at the number", json[0]["tag"]);
        //vec![]
    } else {
        vec![]
    };

    let completion = gtk::EntryCompletion::new();
    let list_store = gtk::ListStore::new(&[glib::Type::STRING]);

    for key in &json_keys {
        list_store.set(&list_store.append(), &[(0, &key)]);
    }

    completion.set_model(Some(&list_store));
    completion.set_text_column(0);
    search_entry.set_completion(Some(&completion));

    // Connect the save button
    save_button.connect_clicked({
        let search_entry = search_entry.clone();
        let value_entry = value_entry.clone();
        move |_| {
            let key = search_entry.text().to_string();
            let value = value_entry.text().to_string();

            if key.is_empty() || value.is_empty() {
                println!("Key or value cannot be empty");
                return;
            }

            println!("Saving key: {} with value: {}", key, value);
            // Here you can add code to save the metadata to the selected file
        }
    });

    let input_box = Box::new(Orientation::Vertical, 6);
    input_box.append(&search_entry);
    input_box.append(&value_entry);
    input_box.append(&save_button);
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
