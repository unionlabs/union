use serde::Serialize;

pub struct Dumper {
    directory_path: String,
}

impl Dumper {
    pub fn new(directory_path: String) -> Self {
        Dumper { directory_path }
    }

    pub fn dump<T: Serialize>(&self, name: String, value: &T) {
        let content = serde_json::to_string_pretty(value).unwrap();
        std::fs::write(format!("{}/{}.json", self.directory_path, name), content).unwrap();
    }
}
