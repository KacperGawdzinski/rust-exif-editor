use gtk::prelude::*;
use gtk::{
    glib, Align, Application, ApplicationWindow, Box, Button, Entry, FileChooserDialog, Label,
    ListBox, ListBoxRow, Orientation, Picture, ResponseType,
};
use rexiv2::Metadata;
use serde_json::Value;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::{Arc, Mutex};

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

    let file_button = Button::with_label("Select Image Directory");
    let left_button = Button::with_label("<");
    let right_button = Button::with_label(">");

    let search_entry = Entry::builder().placeholder_text("Search keys...").build();
    let value_entry = Entry::builder().placeholder_text("Enter value...").build();
    let save_button = Button::with_label("Save");
    let list_box = ListBox::new();

    let path_str = Arc::new(Mutex::new(String::new()));
    let image_paths = Arc::new(Mutex::new(Vec::<String>::new()));
    let current_index = Arc::new(Mutex::new(0));
    let path_str = Arc::new(Mutex::new(String::new()));

    file_button.connect_clicked({
        let picture = picture.clone();
        let path_str: Arc<Mutex<String>> = path_str.clone();
        let image_paths = image_paths.clone();
        let current_index = current_index.clone();
        let list_box = list_box.clone();

        move |_| {
            let file_dialog = FileChooserDialog::new(
                Some("Select an Image"),
                None::<&ApplicationWindow>,
                gtk::FileChooserAction::SelectFolder,
                &[("Close", gtk::ResponseType::Close)],
            );

            file_dialog.add_buttons(&[
                ("Cancel", ResponseType::Cancel),
                ("Open", ResponseType::Accept),
            ]);

            file_dialog.connect_response({
                let picture = picture.clone();
                let path_str = path_str.clone();
                let image_paths = image_paths.clone();
                let current_index = current_index.clone();
                let list_box = list_box.clone();

                move |dialog, response| {
                    if response == ResponseType::Accept {
                        if let Some(folder_path) = dialog.file().and_then(|f| f.path()) {
                            let mut paths = Vec::new();

                            for entry in fs::read_dir(folder_path).unwrap() {
                                let entry = entry.unwrap();
                                let path = entry.path();
                                if path.is_file()
                                    && path.extension().map_or(false, |ext| {
                                        matches!(
                                            ext.to_str().unwrap_or(""),
                                            "jpg" | "jpeg" | "png" | "bmp"
                                        )
                                    })
                                {
                                    paths.push(path);
                                }
                            }

                            *image_paths.lock().unwrap() = paths
                                .iter()
                                .map(|p| p.to_str().unwrap().to_string())
                                .collect();
                            *current_index.lock().unwrap() = 0;

                            if let Some(first_image) = image_paths.lock().unwrap().first() {
                                let pic = gio::File::for_path(first_image);
                                picture.set_file(Some(&pic));

                                let mut child = list_box.first_child();
                                while let Some(widget) = child {
                                    child = widget.next_sibling();
                                    list_box.remove(&widget);
                                }

                                if let Ok(file) = std::fs::File::open(first_image) {
                                    let mut path_str = path_str.lock().unwrap();
                                    path_str.replace_range(.., first_image);

                                    let mut bufreader = std::io::BufReader::new(&file);
                                    let exifreader = exif::Reader::new();
                                    if let Ok(exif) = exifreader.read_from_container(&mut bufreader)
                                    {
                                        for f in exif.fields() {
                                            let row = ListBoxRow::new();
                                            let row_box = Box::new(Orientation::Horizontal, 6);

                                            let key_label = Label::new(Some(&f.tag.to_string()));
                                            key_label.set_halign(Align::Start);

                                            let value_label = Label::new(Some(
                                                &f.display_value()
                                                    .with_unit(&exif)
                                                    .to_string()
                                                    .chars()
                                                    .take(50)
                                                    .collect::<String>(),
                                            ));
                                            value_label.set_halign(Align::End);

                                            row_box.append(&key_label);
                                            row_box.append(&value_label);
                                            row.set_child(Some(&row_box));

                                            list_box.append(&row);
                                        }
                                    }
                                }
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
                    // .map(|full_tag| full_tag.split('.').nth(2).unwrap_or(&full_tag).to_string())
                    .collect()
            })
            .unwrap_or_default()
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

    save_button.connect_clicked({
        let search_entry = search_entry.clone();
        let value_entry = value_entry.clone();
        let list_box = list_box.clone();
        let path_str = path_str.clone();

        move |_| {
            let key = search_entry.text().to_string();
            let value = value_entry.text().to_string();

            if key.is_empty() || value.is_empty() {
                println!("Key or value cannot be empty");
                return;
            }

            let mut data = HashMap::new();
            data.insert("Key1".to_string(), "Value1".to_string());
            data.insert("Key2".to_string(), "Value2".to_string());
            data.insert("Key3".to_string(), "Value3".to_string());

            for (key, value) in data {
                let row = ListBoxRow::new();
                let row_box = Box::new(Orientation::Horizontal, 6);

                let key_label = Label::new(Some(&key));
                key_label.set_halign(Align::Start);

                let value_label = Label::new(Some(&value));
                value_label.set_halign(Align::End);

                row_box.append(&key_label);
                row_box.append(&value_label);
                row.set_child(Some(&row_box));

                list_box.append(&row);
            }

            let mut path_str = path_str.lock().unwrap();

            // println!("{} {} {}", key, value, &*path_str);
            let metadata = Metadata::new_from_path(&*path_str).unwrap();
            metadata.set_tag_string(&key, &value).unwrap();
            metadata.save_to_file(&*path_str).unwrap();
        }
    });

    left_button.connect_clicked({
        let picture = picture.clone();
        let image_paths = image_paths.clone();
        let current_index = current_index.clone();
        let path_str: Arc<Mutex<String>> = path_str.clone();
        let list_box = list_box.clone();

        move |_| {
            let mut index = current_index.lock().unwrap();
            let images = image_paths.lock().unwrap();
            if !images.is_empty() {
                *index = if *index == 0 {
                    images.len() - 1
                } else {
                    *index - 1
                };

                if let Some(image) = images.get(*index) {
                    let pic = gio::File::for_path(image);
                    picture.set_file(Some(&pic));

                    let pic = gio::File::for_path(image);
                    picture.set_file(Some(&pic));

                    let mut child = list_box.first_child();
                    while let Some(widget) = child {
                        child = widget.next_sibling();
                        list_box.remove(&widget);
                    }

                    if let Ok(file) = std::fs::File::open(image) {
                        let mut path_str = path_str.lock().unwrap();
                        path_str.replace_range(.., image);

                        let mut bufreader = std::io::BufReader::new(&file);
                        let exifreader = exif::Reader::new();
                        if let Ok(exif) = exifreader.read_from_container(&mut bufreader) {
                            for f in exif.fields() {
                                let row = ListBoxRow::new();
                                let row_box = Box::new(Orientation::Horizontal, 6);

                                let key_label = Label::new(Some(&f.tag.to_string()));
                                key_label.set_halign(Align::Start);

                                let value_label = Label::new(Some(
                                    &f.display_value()
                                        .with_unit(&exif)
                                        .to_string()
                                        .chars()
                                        .take(50)
                                        .collect::<String>(),
                                ));
                                value_label.set_halign(Align::End);

                                row_box.append(&key_label);
                                row_box.append(&value_label);
                                row.set_child(Some(&row_box));

                                list_box.append(&row);
                            }
                        }
                    }
                }
            }
        }
    });

    right_button.connect_clicked({
        let picture = picture.clone();
        let image_paths = image_paths.clone();
        let current_index = current_index.clone();
        let list_box = list_box.clone();

        move |_| {
            let mut index = current_index.lock().unwrap();
            let images = image_paths.lock().unwrap();
            if !images.is_empty() {
                *index = (*index + 1) % images.len();

                if let Some(image) = images.get(*index) {
                    let pic = gio::File::for_path(image);
                    picture.set_file(Some(&pic));

                    let pic = gio::File::for_path(image);
                    picture.set_file(Some(&pic));

                    let mut child = list_box.first_child();
                    while let Some(widget) = child {
                        child = widget.next_sibling();
                        list_box.remove(&widget);
                    }

                    if let Ok(file) = std::fs::File::open(image) {
                        let mut path_str = path_str.lock().unwrap();
                        path_str.replace_range(.., image);

                        let mut bufreader = std::io::BufReader::new(&file);
                        let exifreader = exif::Reader::new();
                        if let Ok(exif) = exifreader.read_from_container(&mut bufreader) {
                            for f in exif.fields() {
                                let row = ListBoxRow::new();
                                let row_box = Box::new(Orientation::Horizontal, 6);

                                let key_label = Label::new(Some(&f.tag.to_string()));
                                key_label.set_halign(Align::Start);

                                let value_label = Label::new(Some(
                                    &f.display_value()
                                        .with_unit(&exif)
                                        .to_string()
                                        .chars()
                                        .take(50)
                                        .collect::<String>(),
                                ));
                                value_label.set_halign(Align::End);

                                row_box.append(&key_label);
                                row_box.append(&value_label);
                                row.set_child(Some(&row_box));

                                list_box.append(&row);
                            }
                        }
                    }
                }
            }
        }
    });

    let input_box = Box::new(Orientation::Vertical, 6);
    input_box.append(&search_entry);
    input_box.append(&value_entry);
    input_box.append(&save_button);
    input_box.append(&file_button);
    input_box.append(&list_box);

    input_box.set_width_request(400);
    input_box.set_hexpand(false);

    let navigation_box = Box::new(Orientation::Horizontal, 12);
    navigation_box.append(&left_button);
    navigation_box.append(&right_button);

    let main_box = Box::new(Orientation::Horizontal, 12);
    main_box.set_margin_top(12);
    main_box.set_margin_bottom(12);
    main_box.set_margin_start(12);
    main_box.set_margin_end(12);

    main_box.append(&picture);
    main_box.append(&navigation_box);
    main_box.append(&input_box);

    let settings = gtk::Settings::default().unwrap();
    settings.set_gtk_application_prefer_dark_theme(true);

    let window = ApplicationWindow::builder()
        .application(app)
        .title("EXIF Rust tool")
        .child(&main_box)
        .default_width(1200)
        .default_height(800)
        .resizable(true)
        .build();

    window.present();
}
