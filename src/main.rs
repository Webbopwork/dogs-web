use gtk::{gdk, glib, prelude::*};

mod backend;

fn main() -> glib::ExitCode {
    let application = gtk::Application::builder()
        .application_id("space.uoxide.dogs-web.text-browser-v0-0-1")
        .build();

    application.connect_startup(|app| {
        let provider = gtk::CssProvider::new();
        provider.load_from_string(include_str!("style.css"));

        gtk::style_context_add_provider_for_display(
            &gdk::Display::default().expect("Could not connect to a display."),
            &provider,
            gtk::STYLE_PROVIDER_PRIORITY_APPLICATION
        );

        build_ui(app);
    });
    application.run()
}

fn build_ui(application: &gtk::Application) {
    let window = gtk::ApplicationWindow::builder()
        .application(application)
        .title("DOGS Web Browser")
        .default_height(720)
        .default_width(1280)
        .build();

    let vbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Vertical)
        .build();

    let hbox = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .valign(gtk::Align::Start)
        .margin_top(20)
        .margin_bottom(20)
        .margin_start(20)
        .margin_end(20)
        .spacing(5)
        .build();

    let ip_input = gtk::Text::builder()
        .placeholder_text("Address and port")
        .css_name("ip-input")
        .build();

    let path_input = gtk::Text::builder()
        .placeholder_text("Path")
        .css_name("path-input")
        .hexpand(true)
        .build();

    let picture_frame = gtk::Picture::builder()
        .visible(false)
        .can_shrink(true)
        .vexpand(true)
        .hexpand(true)
        .build();

    let hbox_content = gtk::Box::builder()
        .orientation(gtk::Orientation::Horizontal)
        .build();

    let content_window = gtk::ScrolledWindow::builder()
        .vexpand(true)
        .hscrollbar_policy(gtk::PolicyType::Automatic)
        .css_name("content-window")
        .margin_bottom(20)
        .margin_start(20)
        .margin_end(20)
        .build();
    
    let content_label = gtk::Label::builder()
        .label("Hello there! Welcome to the 0.0.2 release of the internet browser using the DOGS communication protocol.\n\nThis current client is based on version 0.0.3 of the web protocol.\n\nFeel free to insert a compatible adress in the first textbox over this, you don't always need the path.\n\nThen send the request using the cleaverly named \"Send\" button!\n\nGreen borders represents a matching hash, red borders reprsent a hash not matching.\n\nThe first textbox displays the status of Hash1 with these colours, the second textbox is Hash2.\n\nA blurple colour on the first field means that the address doesn't work. Blurple on the second means other internal error.")
        .wrap_mode(gtk::pango::WrapMode::Word)
        .wrap(true)
        .hexpand(true)
        .css_name("content-label")
        .build();

    hbox_content.append(&content_label);
    hbox_content.append(&picture_frame);

    content_window.set_child(Some(&hbox_content));

    let send_button = gtk::Button::builder()
        .label("Send")
        .css_name("send-button")
        .build();

    hbox.append(&ip_input);
    hbox.append(&path_input);
    hbox.append(&send_button);

    vbox.append(&hbox);

    vbox.append(&content_window);

    window.set_child(Some(&vbox));

    send_button.connect_clicked(move |button| {
        println!("Send button pressed");
        backend::communication_start(&ip_input, &path_input, &content_label, &button, &picture_frame);
    });

    application.connect_activate(move |_| {
        window.present();
    });
}