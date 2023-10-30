export function new_a() {
  return document.createElement("a");
}

export function new_body() {
  return document.createElement("body");
}

export function new_br() {
  return document.createElement("br");
}

export function new_button() {
  return document.createElement("button");
}

export function new_caption() {
    return document.createElement("caption");
}

export function new_del() {
    return document.createElement("del");
}

export function new_div() {
    return document.createElement("div");
}

export function new_form() {
  return document.createElement("form");
}

export function new_food_options_collection() {
    return new_select_with_food_opts().options;
}

export function new_head() {
  return document.createElement("head");
}

export function new_heading() {
    return document.createElement("h1");
}

export function new_hr() {
    return document.createElement("hr");
}

export function new_html() {
  return document.createElement("html");
}

export function new_input() {
    return document.createElement("input");
}

export function new_ins() {
    return document.createElement("ins");
}

export function new_menu() {
    return document.createElement("menu");
}

export function new_menuitem() {
    return document.createElement("menuitem");
}

export function new_meta() {
    return document.createElement("meta");
}

export function new_meter() {
    return document.createElement("meter");
}

export function new_olist() {
    return document.createElement("ol");
}

export function new_optgroup() {
    return document.createElement("optgroup");
}

export function new_output() {
    return document.createElement("output");
}

export function new_paragraph() {
    return document.createElement("p");
}

export function new_param() {
    return document.createElement("param");
}

export function new_pre() {
    return document.createElement("pre");
}

export function new_progress() {
    return document.createElement("progress");
}

export function new_quote() {
    return document.createElement("q");
}

export function new_script() {
  return document.createElement("script");
}

export function new_select_with_food_opts() {
  let select = document.createElement("select");
  let opts = ["tomato", "potato", "orange", "apple"];

  for(let i = 0; i < opts.length; i++) {
      let opt = document.createElement("option");
      opt.value = opts[i];
      opt.text = opts[i];
      select.appendChild(opt);
  }
  return select;
}

export function new_slot() {
    return document.createElement("slot");
}

export function new_span() {
  return document.createElement("span");
}

export function new_style() {
  return document.createElement("style");
}

export function new_table() {
    return document.createElement("table");
}

export function new_tfoot() {
    return document.createElement("tfoot");
}

export function new_thead() {
    return document.createElement("thead");
}

export function new_title() {
  return document.createElement("title");
}

export function new_xpath_result() {
    let xmlDoc = new DOMParser().parseFromString("<root><value>tomato</value></root>", "application/xml");
    let xpathResult = xmlDoc.evaluate("/root//value", xmlDoc, null, XPathResult.ANY_TYPE, null);
    return xpathResult;
}
