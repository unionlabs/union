fn good1() -> String {
    wasm_bindgen::link_to!(inline_js = "console.log('Hello world!');")
}

fn good2() -> String {
    wasm_bindgen::link_to!(raw_module = "./foo.js")
}

fn bad1() -> String {
    wasm_bindgen::link_to!(module = "package/foo.js")
}

fn bad2() -> String {
    wasm_bindgen::link_to!(raw_module = "package/foo.js")
}

fn bad3() -> String {
    wasm_bindgen::link_to!(module = "/src/not-found.js")
}

fn bad4() -> String {
    wasm_bindgen::link_to!()
}

fn bad5() -> String {
    wasm_bindgen::link_to!(
        inline_js = "console.log('Hello world!');",
        js_namespace = foo
    )
}

fn main() {}
