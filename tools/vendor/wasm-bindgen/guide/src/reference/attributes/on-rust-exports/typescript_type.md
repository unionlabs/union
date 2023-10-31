# typescript_type

The `typescript_type` allows us to use typescript declarations in `typescript_custom_section` as arguments for rust functions! For example:

```rust
#[wasm_bindgen(typescript_custom_section)]
const ITEXT_STYLE: &'static str = r#"
interface ITextStyle {
    bold: boolean;
    italic: boolean;
    size: number;
}
"#;

#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "ITextStyle")]
    pub type ITextStyle;
}

#[wasm_bindgen]
#[derive(Default)]
pub struct TextStyle {
    pub bold: bool,
    pub italic: bool,
    pub size: i32,
}

#[wasm_bindgen]
impl TextStyle {
    #[wasm_bindgen(constructor)]
    pub fn new(i: ITextStyle) -> TextStyle {
        let _js_value: JsValue = i.into();
        // parse JsValue
        TextStyle::default()
    }

    pub fn optional_new(_i: Option<ITextStyle>) -> TextStyle {
        // parse JsValue
        TextStyle::default()
    }
}
```

We can write our `typescript` code like: 

```ts
import { ITextStyle, TextStyle } from "./my_awesome_module";

const style: TextStyle = new TextStyle({
  bold: true,
  italic: true,
  size: 42,
});

const optional_style: TextStyle = TextStyle.optional_new();
```
