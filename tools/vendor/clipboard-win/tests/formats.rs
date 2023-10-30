use clipboard_win::raw::{register_format, format_name, format_name_big};

#[test]
fn custom_format_smol() {
    const NAME: &str = "SMOL";

    let format = register_format(NAME).expect("To create format").get();

    let name = format_name(format).expect("To get name");

    assert_eq!(NAME, name.as_str());
}

#[test]
fn custom_format_big() {
    const NAME: &str = "ahdkajhfdsakjfhhdsakjgfdsakjgfdsakjghrdskjghfdskjghrdskjghfdkjghfds;kjghfd;kjgfdsjgfdskjgbfdkjgfdgkjfdsahgkjfdghkjfdgkjfdgfdkjgbfdkjgsakjdhsakjdhs";

    let format = match register_format(NAME) {
        Some(format) => format.get(),
        None => {
            panic!("Failed to register format: {}", std::io::Error::last_os_error());
        },
    };

    let name = format_name_big(format).expect("To get name");

    assert_eq!(NAME, name.as_str());
}
