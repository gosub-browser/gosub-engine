use anyhow::Result;
use gosub_engine::html5::parser::document::{Document, DocumentBuilder};
use gosub_engine::{
    bytes::{CharIterator, Confidence, Encoding},
    html5::parser::Html5Parser,
};
use ::std::process::exit;

use mozjs::rooted;
use mozjs::rust::SIMPLE_GLOBAL_CLASS;
use mozjs::{jsapi::*, rust::JSEngine, rust::RealmOptions, rust::Runtime};
use mozjs::jsval::UndefinedValue;
use ::std::ptr;

fn bail(message: &str) -> ! {
    println!("{}", message);
    exit(1);
}

fn main() -> Result<()> {
    let url = ::std::env::args()
        .nth(1)
        .unwrap_or_else(|| bail("Usage: gosub-parser <url>"));

    let html = if url.starts_with("http://") || url.starts_with("https://") {
        // Fetch the html from the url
        let response = ureq::get(&url).call()?;
        if response.status() != 200 {
            bail(&format!(
                "Could not get url. Status code {}",
                response.status()
            ));
        }
        response.into_string()?
    } else {
        // Get html from the file
        ::std::fs::read_to_string(&url)?
    };

    let mut chars = CharIterator::new();
    chars.read_from_str(&html, Some(Encoding::UTF8));
    chars.set_confidence(Confidence::Certain);

    // If the encoding confidence is not Confidence::Certain, we should detect the encoding.
    if !chars.is_certain_encoding() {
        chars.detect_encoding()
    }

    let document = DocumentBuilder::new_document();
    let parse_errors = Html5Parser::parse_document(&mut chars, Document::clone(&document), None)?;

    println!("Generated tree: \n\n {}", document);

    for e in parse_errors {
        println!("Parse Error: {}", e.message)
    }

    let engine = JSEngine::init().expect("failed to initalize JS engine");
    let runtime = Runtime::new(engine.handle());
    assert!(!runtime.cx().is_null(), "failed to create JSContext");

    run(runtime);

    Ok(())
}


fn run(rt: Runtime) {
    let cx = rt.cx();
    let options = RealmOptions::default();
    rooted!(in(cx) let global = unsafe {
        JS_NewGlobalObject(cx, &SIMPLE_GLOBAL_CLASS, ptr::null_mut(),
                           OnNewGlobalHookOption::FireOnNewGlobalHook,
                           &*options)
    });

    let filename: &'static str = "inline.js";
    let lineno: u32 = 1;

    rooted!(in(rt.cx()) let mut rval = UndefinedValue());

    let source: &'static str = "document.write(\"hello world from spidermonkey\"); 40 + 2";
    let res = rt.evaluate_script(global.handle(), source, filename, lineno, rval.handle_mut());

    if res.is_ok() {
        println!("{}", rval.get().to_int32());
    }
}