use dogs;
use dogs::check::CheckableDog;
use std::net::SocketAddr;
use std::io::{Write, self};
use std::str::FromStr;
use gtk::{prelude::EntryBufferExtManual, prelude::WidgetExt};
use gtk::gio::{prelude::IOStreamExt, prelude::OutputStreamExtManual, self};

pub fn communication_start(ip_input: &gtk::Text, path_input: &gtk::Text, content_label: &gtk::Label, send_button: &gtk::Button, picture_frame: &gtk::Picture) {
    let client_addr = SocketAddr::from(([0, 0, 0, 0], dogs::Dog::CLIENT_PORT));

    let client = dogs::ConnectedDog::new(client_addr).unwrap();

    let mut address: String = ip_input.buffer().text().to_string().trim().to_string();

    let path: String = path_input.buffer().text().to_string().replace("\n", "");

    if address.len() != 0 {
        if address.find(":") == None {
            address = format!("{}:0", address);
        }
        match process(&client, &address, &path, &ip_input, &path_input, &content_label, &send_button, &picture_frame) {
            Ok(_) => println!("\nProcess over!"),
            Err(e) => {
                eprintln!("Got error: {}\nEnding process...", e);
                other_sys_err(&path_input);
            }
        }
    } else {
        eprintln!("Address is empty");
        addr_sys_err(&ip_input);
    }
}

fn process(dog: &dogs::ConnectedDog, address_string: &String, path: &String, ip_input: &gtk::Text, path_input: &gtk::Text, content_label: &gtk::Label, send_button: &gtk::Button, picture_frame: &gtk::Picture) -> io::Result<()> {
    // Create the server address.
    let mut server_addr = match SocketAddr::from_str(address_string) {
        Ok(addr) => addr,
        Err(e) => {
            eprintln!("Got error: {}\nEnding process...", e);
            addr_sys_err(&ip_input);
            return Ok(());
        }
    };

    if server_addr.port() == 0 {
        server_addr.set_port(dogs::Dog::SERVER_PORT);
    }

    println!("Connecting to server...");
    dog.connect(server_addr)?;
    println!("Connected to server!\nSending request..");

    send_request(dog, path)?;

    let (data_first, code_first) = listen_for_response(dog, &ip_input, &path_input, &send_button)?;

    let (content_type, data) = 
    if code_first.opt1 { //Then it's only one packet
        if code_first.opt5 { //Then you are using types
            separate_type(data_first)
        } else {
            (None, data_first)
        }
    } else {
        let mut all_data: Vec<Vec<u8>> = vec![remove_type(data_first.clone())];
        loop {
            let (data, code) = listen_for_response(dog, &ip_input, &path_input, &send_button)?;
            all_data.push(data);
            if code.opt1 { //Then server has sent all data
                break;
            }
        }
        
        (
            if code_first.opt5 {
                get_type(data_first)
            } else {
                None
            },
            all_data.concat()
        )
    };

    match content_type {
        Some(the_type) => {
            println!("Current content type is: '{}'\nRecieved all data from server.", the_type);
            handle_type(the_type, data, &content_label, &path_input, &picture_frame)
        },
        None => {
            println!("Current content type is: None");
            handle_utf8_type(data, &content_label, &path_input, &picture_frame)
        }
    }

    Ok(())
}

fn send_request(dog: &dogs::ConnectedDog, path: &String) -> io::Result<usize> {
    // Currently the handshake/bark data means nothing, that's why it's set to be empty. In the future it will have meaning tho.
    // Send a "path" or identifying data as additional data.
    dog.identify_with_data(dogs::BarkCode::empty(), &path.clone().into_bytes())
}

fn listen_for_response(dog: &dogs::ConnectedDog, ip_input: &gtk::Text, path_input: &gtk::Text, _send_button: &gtk::Button) -> io::Result<(Vec<u8>, dogs::BarkCode)> {
    // Recieves the BarkCode of the sender, with a peek as to not consume the packet.
    let (_size, code) = dog.bark_peek_listen()?;

    // Gets all data sent in the packet(NOT including BarkCode) as well as 2 hashes by the sender, consuming it, we can be sure that the address is the same as before because we didn't consume it before.
    let (recieved_data, hash1, hash2, data_hash) = dog.get_checker_duo()?;

    println!("Hash checks begin...\n");
    let is_hash1_same = hash1 == data_hash;
    let is_hash2_same = hash2 == data_hash;
    let is_either_hashes_same = is_hash1_same || is_hash2_same;
    let is_both_hashes_same = is_hash1_same && is_hash2_same;

    println!("Hash 1: {}\nHash 2: {}\nHash 1 or 2: {}\nHash 1 and 2: {}\n", is_hash1_same, is_hash2_same, is_either_hashes_same, is_both_hashes_same);

    if is_hash1_same {
        ip_input.add_css_class("ok");
        ip_input.remove_css_class("err");
        ip_input.remove_css_class("sys-err");
    } else {
        ip_input.add_css_class("err");
        ip_input.remove_css_class("ok");
        ip_input.remove_css_class("sys-err");
    }

    if is_hash2_same {
        path_input.add_css_class("ok");
        path_input.remove_css_class("err");
        path_input.remove_css_class("sys-err");
    } else {
        path_input.add_css_class("err");
        path_input.remove_css_class("ok");
        path_input.remove_css_class("sys-err");
    }

    Ok((recieved_data, code))
}

fn separate_type(data: Vec<u8>) -> (Option<String>, Vec<u8>) {
    (get_type(data.clone()), remove_type(data))
}

fn get_type(data: Vec<u8>) -> Option<String> {
    Some(String::from_utf8_lossy(&data[..4]).to_string())
}

fn remove_type(data: Vec<u8>) -> Vec<u8> {
    data[4..].to_owned()
}

fn handle_type(content_type: String, data: Vec<u8>, content_label: &gtk::Label, path_input: &gtk::Text, picture_frame: &gtk::Picture) {
    match content_type.as_str() {
        "txt " | "utf8" => handle_utf8_type(data, &content_label, &path_input, &picture_frame),
        "img " | "webp" | "jpeg" | "png " => handle_img_type(data, &content_label, &path_input, &picture_frame, &content_type),
        _ => {
            eprintln!("Type set by server is not identified.");
            other_sys_err(&path_input)
        }
    }
}

fn handle_utf8_type(data: Vec<u8>, content_label: &gtk::Label, path_input: &gtk::Text, picture_frame: &gtk::Picture) {
    match str::from_utf8(&data) {
        Ok(text) => {
            println!("Recieved utf-8:\n\n{}", text);
            picture_frame.set_visible(false);
            content_label.set_visible(true);
            content_label.set_text(text);
        },
        Err(e) => {
            eprintln!("Failed to convert data to utf-8, got error: {}", e);
            other_sys_err(&path_input);
        }
    }
}

fn handle_img_type(data: Vec<u8>, content_label: &gtk::Label, path_input: &gtk::Text, picture_frame: &gtk::Picture, content_type: &String) {
    content_label.set_visible(false);
    picture_frame.set_visible(true);

    let tmp_path = format!("dogs-web-image-file-XXXXXX.{}", content_type.replace(" ", ""));

    println!("Temporary file path: {}", tmp_path);

    match gio::File::new_tmp(Some(tmp_path)) {
        Ok((tmp_file, file_stream)) => {
            let mut output_stream_write = file_stream.output_stream().into_write();
            match output_stream_write.write_all(&data) {
                Ok(_) => {
                    match output_stream_write.flush() {
                        Ok(_) => {
                            picture_frame.set_file(Some(&tmp_file));
                            println!("Put picture in pitcure_frame");
                        },
                        Err(e) => {
                            eprintln!("Failed to flush temporary file, got error: {}", e);
                            other_sys_err(&path_input);
                        }
                    }
                },
                Err(e) => {
                    eprintln!("Failed to write to temporary file, got error: {}", e);
                    other_sys_err(&path_input);
                }
            }
        },
        Err(e) => {
            eprintln!("Failed to create temporary file, got error: {}", e);
            other_sys_err(&path_input);
        }
    }
}

fn addr_sys_err(ip_input: &gtk::Text) {
    ip_input.add_css_class("sys-err");
    ip_input.remove_css_class("ok");
    ip_input.remove_css_class("err");
}

fn other_sys_err(path_input: &gtk::Text) {
    path_input.add_css_class("sys-err");
    path_input.remove_css_class("ok");
    path_input.remove_css_class("err");
}